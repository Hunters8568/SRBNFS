use flume::{Receiver, Sender};
use log::{debug, error, info, trace, warn};
use protocol::Packet;
use serde_json::json;
use shared::ringbuffer::RingBuffer;
use std::io::BufRead;
use std::os::fd::AsRawFd;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub enum Event {
    ClientConnected(TcpStream),
    ClientDisconnected(TcpStream),
    InjectFile(String, String),
    RelayPacket(Packet),
}

pub struct RootServerClient {
    stream: Arc<Mutex<TcpStream>>,
}

pub struct RootServer {
    listener: Arc<TcpListener>,
    sender: Arc<Sender<Event>>,
    rec: Arc<Receiver<Event>>,
    client_list: Arc<Mutex<Vec<RootServerClient>>>,
    relay_server_ring: shared::ringbuffer::RingBuffer,
}

impl RootServer {
    pub fn new(
        port: u16,
        relay_server_ring: shared::ringbuffer::RingBuffer,
    ) -> Result<Self, std::io::Error> {
        info!("RootServer bind to: {}", port);

        let (tx, rx) = flume::unbounded();

        Ok(Self {
            listener: Arc::new(TcpListener::bind(format!("0.0.0.0:{}", port))?),
            sender: Arc::new(tx),
            rec: Arc::new(rx),
            client_list: Arc::new(Mutex::new(vec![])),
            relay_server_ring,
        })
    }

    fn spawn_client(sender: Arc<Sender<Event>>, client: RootServerClient) {
        trace!("Started client thread");

        let mut stream = client.stream.lock().unwrap().try_clone().unwrap();

        let mut handshake_packet = Packet::new(protocol::PacketType::Handshake, true);
        handshake_packet.data = Some(json!({
            "ProgramName": "srbnfs_root_server"
        }));

        handshake_packet.send(&mut stream);

        // Stream used when shutting down this client
        let shutdown_stream = stream.try_clone().unwrap();

        let mut bufreader: BufReader<TcpStream> = BufReader::new(stream);

        loop {
            let mut line = String::new();

            if bufreader.read_line(&mut line).is_err() {
                error!("Socket I/O failure, disconnecting client");
            }

            // Cleanup packet

            line = line.trim().to_string();

            if line.is_empty() {
                warn!("Client sent blank packet, assuming disconnection??");

                sender
                    .send(Event::ClientDisconnected(shutdown_stream))
                    .expect("Failed to send disconnect event to dispatch");
                break;
            };

            trace!("Unparsed paylod: {}", line);

            let packet: protocol::Packet = match serde_json::from_str(&line) {
                Ok(p) => p,
                Err(err) => {
                    error!("Failed to parse packet: {}", err);
                    continue;
                }
            };

            trace!("Parsed payload: {:#?}", packet);

            let data = match packet.data {
                Some(ref e) => e,
                None => {
                    warn!("Packet has no payload, ignoring");
                    continue;
                }
            };

            match packet.packet_type {
                protocol::PacketType::Handshake => {
                    error!("Root server got handshake, this should not happen, ignoring");
                }
                protocol::PacketType::RootConfiguration => {
                    error!("Root server got configuration packet! Ignoring");
                }
                protocol::PacketType::RelayFile => {
                    sender
                        .send(Event::RelayPacket(packet))
                        .expect("Failed to send relay packet");
                }
                protocol::PacketType::InjectFile => {
                    let file = data.get("FileName");
                    let file_content_base64 = data.get("FileContent");

                    if file.is_none() || file_content_base64.is_none() {
                        warn!("InjectFile packet is missing FileName or FileContent!");
                        continue;
                    }

                    // Inject the file into our ring
                    sender
                        .send(Event::InjectFile(
                            file.unwrap().to_string(),
                            file_content_base64.unwrap().to_string(),
                        ))
                        .expect("Failed to send InjectFile message");
                }
            }
        }
    }

    fn mainloop_messagequeue(
        client_list: Arc<Mutex<Vec<RootServerClient>>>,
        ringbuffer: RingBuffer,
        sender: Arc<Sender<Event>>,
        rec: Arc<Receiver<Event>>,
    ) {
        debug!("Started message queue mainloop");

        for msg in rec.iter() {
            trace!("Got message: {:#?}", msg);

            match msg {
                Event::RelayPacket(packet) => {
                    let ip = ringbuffer.at(1); // 0 is us!

                    debug!("Attempting to connect to inital relay: {}", ip);

                    let mut stream = match TcpStream::connect(ip) {
                        Ok(s) => {
                            trace!("Connected to inital relay!");
                            s
                        }
                        Err(e) => {
                            error!(
                                "Failed to connect to inital relay, no clients will be informed of this file! {}",
                                e
                            );

                            break;
                        }
                    };

                    packet.send(&mut stream);
                    stream
                        .shutdown(std::net::Shutdown::Both)
                        .expect("Failed to shutdown I/O");
                }
                Event::ClientDisconnected(socket) => {
                    debug!("Client disconnected with address of: {:#?}", socket);

                    let mut index = 0;
                    let mut found = false;

                    for client in client_list.lock().unwrap().iter() {
                        let client_addr = client.stream.lock().unwrap().peer_addr();

                        if client_addr.is_err() {
                            found = true;
                            break;
                        }

                        index += 1;
                    }

                    if found {
                        client_list.lock().unwrap().remove(index);
                    } else {
                        error!("Failed to disconnect client!");
                    }
                }
                Event::InjectFile(file_path, file_content_hashed) => {
                    trace!(
                        "Injecting file {} with content size of {}",
                        file_path,
                        file_content_hashed.len()
                    );

                    // Alert the first server in the chain of this file
                    let ip = ringbuffer.at(1); // 0 is us!

                    debug!("Attempting to connect to inital relay: {}", ip);

                    let mut stream = match TcpStream::connect(ip) {
                        Ok(s) => {
                            trace!("Connected to inital relay!");
                            s
                        }
                        Err(e) => {
                            error!(
                                "Failed to connect to inital relay, no clients will be informed of this file! {}",
                                e
                            );

                            break;
                        }
                    };

                    let mut init_relay_pack = Packet::new(protocol::PacketType::RelayFile, false);
                    init_relay_pack.data = Some(json!({
                            "FileName": file_path,
                            "FileContent": file_content_hashed,
                            "StartTime": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Could not determine current time")
                    .as_secs(),
                        }));

                    init_relay_pack.send(&mut stream);

                    // Tell all connected clients to the root server about this
                    for client in client_list.lock().unwrap().iter() {
                        let mut stream = client.stream.lock().unwrap();

                        // Generate packet
                        let mut relay_packet = Packet::new(protocol::PacketType::RelayFile, false);
                        relay_packet.data = Some(json!({
                                "FileName": file_path,
                                "FileContent": file_content_hashed,
                                "StartTime": SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not determine current time")
                        .as_secs(),
                            }));

                        relay_packet.send(&mut stream);
                    }
                }
                Event::ClientConnected(stream) => {
                    debug!("Adding new client to connection list!");
                    let stream_cloned = Arc::new(Mutex::new(stream.try_clone().unwrap()));

                    let client = RootServerClient {
                        stream: Arc::new(Mutex::new(stream)),
                    };

                    client_list.lock().unwrap().push(client);

                    // Clone resources for client thread

                    let new_client = RootServerClient {
                        stream: stream_cloned,
                    };

                    let sender_cloned = sender.clone();
                    std::thread::spawn(move || RootServer::spawn_client(sender_cloned, new_client));
                }
            };
        }
    }

    pub fn mainloop(&mut self) -> Result<(), std::io::Error> {
        info!("RootServer waiting for incoming connections");

        let sender_cloned = self.sender.clone();
        let rec_cloned = self.rec.clone();
        let client_list_cloned = self.client_list.clone();
        let ringbuffer_cloned = self.relay_server_ring.clone();

        std::thread::spawn(move || {
            RootServer::mainloop_messagequeue(
                client_list_cloned,
                ringbuffer_cloned,
                sender_cloned,
                rec_cloned,
            );
        });

        loop {
            let (stream, addr) = self.listener.accept()?;

            info!("Client connected with address: {:#?}", addr);

            self.sender
                .send(Event::ClientConnected(stream.try_clone().unwrap()))
                .expect("Failed to send message");
        }
    }
}
