use std::net::TcpListener;
use std::net::TcpStream;

use clap::Command;
use clap::arg;
use log::*;
use ringbuffer::RingBuffer;
use server::packet::ServerMode;

mod ringbuffer;
mod server;
mod state;

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
        _ => unreachable!(),
    }

    info!("Welcome to SRBNFS - Stupid Ring Buffer Network Filesystem");

    if enable_relay_server {
        let server = server::Server {
            listener: TcpListener::bind(format!("0.0.0.0:{}", relay_server_port))
                .expect("Failed to find to local address"),
        };

        info!(
            "SRBNFS server (RELAY) started on address 0.0.0.0:{}",
            relay_server_port
        );

        for client in server.listener.incoming() {
            debug!("Client connected to relay");

            std::thread::spawn(move || {
                let mut client = server::Client {
                    stream: client.unwrap(),
                    ring_buffer: RingBuffer::new(vec![]),
                    next_ip: None,
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

        let server = server::Server {
            listener: TcpListener::bind(format!("0.0.0.0:{}", SRBNFS_SERVER_PORT))
                .expect("Failed to find to local address"),
        };

        info!(
            "SRBNFS server started on address 0.0.0.0:{}",
            SRBNFS_SERVER_PORT
        );

        for client in server.listener.incoming() {
            debug!("Client connected");
            let mut new_ring = vec![];

            for ip in ring_split.to_vec() {
                new_ring.push(ip.to_string());
            }

            std::thread::spawn(move || {
                let mut client = server::Client {
                    stream: client.unwrap(),
                    ring_buffer: RingBuffer::new(new_ring.to_vec()),
                    next_ip: None,
                    op_mode: ServerMode::Unknown,
                };

                client.handle_rootserver();
            });
        }
    }
}
