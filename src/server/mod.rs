pub mod packet;

use log::*;
use packet::{Packet, ServerMode};
use std::{io::BufRead, net::TcpStream, sync::Arc, sync::Mutex};

use crate::ringbuffer::RingBuffer;

pub struct Server {
    pub listener: Arc<Mutex<std::net::TcpListener>>,
    pub clients: Arc<Mutex<Vec<TcpStream>>>,
}
pub struct Client {
    pub stream: std::net::TcpStream,
    pub ring_buffer: RingBuffer,
    pub next_ip: Arc<Mutex<String>>,
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

                    *self.next_ip.lock().unwrap() = next_ip;
                }
                packet::PacketType::RelayFile => {
                    let has_ip = !(*self.next_ip.lock().unwrap()).is_empty();
                    if !has_ip {
                        error!("Cannot relay file without root server pre-configuration");
                    } else {
                        let next_ip = (*self.next_ip).lock().unwrap().clone();

                        debug!("Relayed file to: {}", next_ip);

                        let mut next_server = std::net::TcpStream::connect(next_ip)
                            .expect("Failed to connect to next remote relay device");

                        std::thread::sleep(std::time::Duration::from_secs(1));
                        packet.send_packet(&mut next_server);
                    }
                }
                packet::PacketType::InjectFileIntoRing => unreachable!(),
            }
        }
    }

    pub fn handle_rootserver<F: Fn(Packet)>(&mut self, relay: F) {
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

            match packet.packet_type {
                packet::PacketType::Handshake => {
                    debug!("Client sent handshake!");
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
                    error!("Root server cannot be configured!");
                    break;
                }
                packet::PacketType::RelayFile => {
                    let next_ip_after_root = self.ring_buffer.at(1);

                    let mut next_server = std::net::TcpStream::connect(next_ip_after_root)
                        .expect("Failed to connect to remote relay device");

                    packet.send_packet(&mut next_server);

                    relay(packet);
                }
                packet::PacketType::InjectFileIntoRing => {
                    let file_name = packet.params["FileName"].clone();

                    let mut relay_packet = Packet::new();
                    relay_packet.packet_type = packet::PacketType::RelayFile;
                    relay_packet
                        .params
                        .insert("FileName".to_string(), file_name);

                    relay_packet.params.insert(
                        "FileEncoded".to_string(),
                        packet.params["FileEncoded"].to_string(),
                    );

                    let mut stream = TcpStream::connect(self.ring_buffer.at(1))
                        .expect("Failed to connect to first server in chain");

                    relay_packet.send_packet(&mut stream);
                }
            };
        }
    }
}
