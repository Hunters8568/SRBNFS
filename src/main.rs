use std::net::TcpListener;

use clap::Command;
use log::*;
use ringbuffer::RingBuffer;

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
        );

    let matches = cmd.get_matches();

    let mut enable_root_server = false;

    trace!("Default: root server enabled: {}", enable_root_server);

    match matches.subcommand() {
        Some(("rootserver", _)) => {
            info!("Root server mode enabled!");
            enable_root_server = true;
        }
        _ => unreachable!(),
    }

    info!("Welcome to SRBNFS - Stupid Ring Buffer Network Filesystem");

    if enable_root_server {
        let ring_cfg_content =
            std::fs::read_to_string("cfg/ring.txt").expect("Failed to read ring config");

        let mut ring_split: Vec<&str> = ring_cfg_content.split("\n").collect();
        ring_split.pop();

        debug!("Loaded ring buffer: {:#?}", ring_split);

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
                };

                client.handle();
            });
        }
    }
}
