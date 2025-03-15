use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMode {
    Relay,
    Root,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketType {
    Handshake,
    Intentions,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    pub date_time: u64,
    pub params: HashMap<String, String>,
    pub packet_type: PacketType,
}

impl Packet {
    pub fn new() -> Self {
        Self {
            date_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
            params: HashMap::new(),
            packet_type: PacketType::Handshake,
        }
    }

    pub fn send_packet(&self, stream: &mut std::net::TcpStream) {
        match stream.write(format!("{}\n", serde_json::to_string(self).unwrap()).as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed during network I/O transport: {}", err);
            }
        }
    }
}
