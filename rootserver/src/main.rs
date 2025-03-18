use std::net::TcpStream;

use log::{debug, info};
use serde_json::json;
use server::RootServer;

mod server;

fn main() {
    pretty_env_logger::init();

    info!("srbnfs running on protocol v2 in ROOTSERVER mode");

    info!("Loading ring from ./cfg/ring.txt");

    let ring_cfg_content =
        std::fs::read_to_string("cfg/ring.txt").expect("Failed to read ring config");

    let mut ring_split: Vec<&str> = ring_cfg_content.split("\n").collect();
    ring_split.pop();

    let mut ringbuf =
        shared::ringbuffer::RingBuffer::new(ring_split.iter().map(|x| x.to_string()).collect());

    debug!("Loaded ring: {:#?}", ring_split);

    info!("Configuring relay servers...");

    ringbuf.next(); // Skip root server

    for index in 1..ringbuf.len() {
        let relay_ip = ringbuf.at(index);
        let next_ip = ringbuf.next();

        debug!(
            "Relay server #{}: Server IP {}, next is {}",
            index, relay_ip, next_ip
        );

        let mut stream =
            TcpStream::connect(relay_ip).expect("Failed to connect to remote relay device");

        let mut next_info_packet =
            protocol::Packet::new(protocol::PacketType::RootConfiguration, true);

        next_info_packet.data = Some(json!({
            "NextRelayAddress": next_ip
        }));

        next_info_packet.send(&mut stream);

        stream
            .shutdown(std::net::Shutdown::Both)
            .expect("Failed to shutdown server");
    }

    let mut root_server = RootServer::new(7848, ringbuf).expect("Failed to bind");

    root_server.mainloop().expect("RootServer logic failure");
}
