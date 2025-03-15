use std::net::TcpListener;

use log::*;

mod server;

const SRBNFS_SERVER_PORT: i32 = 7848;

fn main() {
    env_logger::init();

    info!("Welcome to SRBNFS - Stupid Ring Buffer Network Filesystem");

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

        std::thread::spawn(move || {
            let mut client = server::Client {
                stream: client.unwrap(),
            };

            client.handle();
        });
    }
}
