pub mod packet;

use log::*;
use packet::ServerMode;
use std::io::BufRead;

use crate::ringbuffer::RingBuffer;

pub struct Server {
    pub listener: std::net::TcpListener,
}
pub struct Client {
    pub stream: std::net::TcpStream,
    pub ring_buffer: RingBuffer,
    pub next_ip: Option<String>,
    pub op_mode: ServerMode,
}

impl Client {
    pub fn handle_relay(&mut self) {
        let mut stream = std::io::BufReader::new(&self.stream);

        let mut welcome_packet = packet::Packet::new();
        welcome_packet
            .params
            .insert("ProgramName".to_string(), "SRBNFS-Relay".to_string());

        let stream1 = &mut self.stream.try_clone().expect("Failed to clone TCPStream");
        welcome_packet.send_packet(stream1);

        loop {
            let mut line_buffer = String::new();

            if stream.read_line(&mut line_buffer).is_err() {
                error!("Failed to read line from BufReader, closing client connection!");
                break;
            }

            if line_buffer.is_empty() {
                break;
            }

            let packet: packet::Packet = serde_json::from_str(&line_buffer)
                .expect("Failed to parse JSON packet from client");

            debug!("{:#?}", packet);

            match packet.packet_type {
                packet::PacketType::Handshake => {
                    debug!("Client handshake complete");
                }
                packet::PacketType::Intentions => {
                    let mode: ServerMode = match packet.params["Intention"].as_str() {
                        "Relay" => ServerMode::Relay,
                        "Root" => ServerMode::Root,
                        _ => ServerMode::Unknown,
                    };

                    self.op_mode = mode;
                    debug!("Connected client is operating as a: {:?}", self.op_mode);
                }
                packet::PacketType::RootServerConfigure => {
                    let next_ip = packet.params["NextIPAddr"].clone();
                    debug!(
                        "Root server configure: Setting next address to: {}",
                        next_ip
                    );

                    self.next_ip = Some(next_ip);
                }
            }
        }
    }

    pub fn handle_rootserver(&mut self) {
        debug!("Client handler start!");

        let mut stream = std::io::BufReader::new(&self.stream);

        let mut welcome_packet = packet::Packet::new();
        welcome_packet
            .params
            .insert("ProgramName".to_string(), "SRBNFS-Router".to_string());

        let stream1 = &mut self.stream.try_clone().expect("Failed to clone TCPStream");
        welcome_packet.send_packet(stream1);

        loop {
            let mut line_buffer = String::new();

            if stream.read_line(&mut line_buffer).is_err() {
                error!("Failed to read line from BufReader, closing client connection!");
                break;
            }

            if line_buffer.is_empty() {
                break;
            }

            let packet: packet::Packet = serde_json::from_str(&line_buffer)
                .expect("Failed to parse JSON packet from client");

            debug!("{:#?}", packet);
        }
    }
}
