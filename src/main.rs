use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

use clap::Command;
use clap::arg;
use log::*;
use ringbuffer::RingBuffer;
use server::packet::PacketType;
use server::packet::ServerMode;

mod ringbuffer;
mod server;

const SRBNFS_SERVER_PORT: i32 = 7848;

fn main() {
    env_logger::init();

    let cmd = Command::new("srbnfs")
        .about("Stupid Ring Buffer Network Filesystem")
        .subcommand_required(true)
        .subcommand(
            Command::new("rootserver")
                .about("Boot in root server mode, loading the ring buffer from disk"),
        )
        .subcommand(
            Command::new("relayserver")
                .about("Boot in relay server mode, will wait for root server connections")
                .arg(arg!(<PORT_NUM> "The port to bind on address 0.0.0.0").required(true)),
        )
        .subcommand(
            Command::new("injectfile")
                .about("Inject a file into the ring at ROOTSERVER")
                .arg(arg!(<FILE> "File to inject").required(true))
                .arg(
                    arg!(<ROOT_SERVER_ADDR> "Root server address to inject files at")
                        .required(true),
                ),
        );

    let matches = cmd.get_matches();

    let mut enable_root_server = false;
    let mut enable_relay_server = false;
    let mut relay_server_port = 0;

    trace!("Default: root server enabled: {}", enable_root_server);
    trace!("Default: relay server enabled: {}", enable_relay_server);
    trace!("Default: relay server port: {}", relay_server_port);

    match matches.subcommand() {
        Some(("rootserver", _)) => {
            info!("Root server mode enabled!");
            enable_root_server = true;
        }
        Some(("relayserver", sub)) => {
            info!("Relay server mode enabled!");
            relay_server_port = sub
                .get_one::<String>("PORT_NUM")
                .expect("Port number required")
                .parse::<u64>()
                .expect("Port number failed to parse");

            enable_relay_server = true;
        }
        Some(("injectfile", sub)) => {
            let server = sub
                .get_one::<String>("ROOT_SERVER_ADDR")
                .expect("Failed to get root server address");

            let file_path = sub
                .get_one::<String>("FILE")
                .expect("Failed to get file to inject");

            info!("Injecting file at: {} from ./{}", server, file_path);

            let file_content = std::fs::read_to_string(file_path).expect("Failed to reader");

            let mut server = TcpStream::connect(server).expect("Failed to connect to root server");

            let mut packet = crate::server::packet::Packet::new();
            packet.packet_type = PacketType::InjectFileIntoRing;
            packet
                .params
                .insert("FileName".to_string(), file_path.to_string());

            packet.params.insert(
                "FileEncoded".to_string(),
                STANDARD.encode(file_content.as_bytes()),
            );

            packet.send_packet(&mut server);

            std::process::exit(0);
        }
        _ => unreachable!(),
    }

    info!("Welcome to SRBNFS - Stupid Ring Buffer Network Filesystem");

    let next_ip: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    if enable_relay_server {
        let server = server::Server {
            listener: Arc::new(Mutex::new(
                TcpListener::bind(format!("0.0.0.0:{}", relay_server_port))
                    .expect("Failed to find to local address"),
            )),
            clients: Arc::new(Mutex::new(vec![])),
        };

        info!(
            "SRBNFS server (RELAY) started on address 0.0.0.0:{}",
            relay_server_port
        );

        let listen = server.listener.lock().unwrap();

        for client in listen.incoming() {
            debug!("Client connected to relay");
            let value = next_ip.clone();

            std::thread::spawn(move || {
                let mut client = server::Client {
                    stream: client.unwrap(),
                    ring_buffer: RingBuffer::new(vec![]),
                    next_ip: value,
                    op_mode: ServerMode::Unknown,
                };

                client.handle_relay();
            });
        }
    }

    if enable_root_server {
        let ring_cfg_content =
            std::fs::read_to_string("cfg/ring.txt").expect("Failed to read ring config");

        let mut ring_split: Vec<&str> = ring_cfg_content.split("\n").collect();
        ring_split.pop();

        debug!("Loaded ring buffer: {:#?}", ring_split);

        info!("Connecting servers in ring");
        let mut ringbuf = RingBuffer::new(ring_split.iter().map(|x| x.to_string()).collect());

        let _ = ringbuf.next(); // Skip the root server

        for index in 1..ringbuf.len() {
            let relay_ip = ringbuf.at(index);
            let next_ip = ringbuf.next();

            debug!(
                "Relay server #{}: Server IP {}, next is {}",
                index, relay_ip, next_ip
            );

            let mut stream =
                TcpStream::connect(relay_ip).expect("Failed to connect to remote relay device");

            let mut next_info_packet = server::packet::Packet::new();
            next_info_packet.packet_type = server::packet::PacketType::RootServerConfigure;

            next_info_packet
                .params
                .insert(String::from("NextIPAddr"), next_ip);

            next_info_packet.send_packet(&mut stream);

            stream
                .shutdown(std::net::Shutdown::Both)
                .expect("Failed to shutdown server");
        }

        debug!("Starting remote SRBNFS server...");

        let listener = TcpListener::bind(format!("0.0.0.0:{}", SRBNFS_SERVER_PORT))
            .expect("Failed to find to local address");

        let server: Arc<Mutex<server::Server>> = Arc::new(Mutex::new(server::Server {
            listener: Arc::new(Mutex::new(listener.try_clone().unwrap())),
            clients: Arc::new(Mutex::new(vec![])),
        }));

        info!(
            "SRBNFS server started on address 0.0.0.0:{}",
            SRBNFS_SERVER_PORT
        );

        for client in listener.incoming() {
            debug!("Client connected");
            let mut new_ring = vec![];

            for ip in ring_split.to_vec() {
                new_ring.push(ip.to_string());
            }

            let weak = Arc::downgrade(&server);

            std::thread::spawn(move || {
                weak.upgrade()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .clients
                    .lock()
                    .unwrap()
                    .push(client.unwrap());

                let size = weak
                    .upgrade()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .clients
                    .lock()
                    .unwrap()
                    .len();

                let mut client = server::Client {
                    stream: weak
                        .upgrade()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .clients
                        .lock()
                        .unwrap()[size - 1]
                        .try_clone()
                        .unwrap(),
                    ring_buffer: RingBuffer::new(new_ring.to_vec()),
                    next_ip: Arc::new(Mutex::new(String::new())),
                    op_mode: ServerMode::Unknown,
                };

                client.handle_rootserver(move |packet| {
                    debug!("Got relay packet to send: {:#?}", packet);
                    let binding = weak.upgrade().unwrap();
                    let binding = binding.lock().unwrap();
                    let clients = binding.clients.lock().unwrap();

                    for client in clients.iter() {
                        let mut socket = client.try_clone().unwrap();
                        packet.send_packet(&mut socket);
                    }
                });
            });
        }
    }
}
