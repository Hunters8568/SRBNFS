use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, error, info, trace, warn};

pub struct RelayServer {
    listener: Arc<TcpListener>,
}

impl RelayServer {
    pub fn new(port: u16) -> Result<Self, std::io::Error> {
        info!("RelayServer bind to: {}", port);

        Ok(Self {
            listener: Arc::new(TcpListener::bind(format!("0.0.0.0:{}", port))?),
        })
    }

    pub fn spawn_client(stream_arc: Arc<Mutex<TcpStream>>, next_ip: Arc<Mutex<String>>) {
        trace!("Started client thread");

        loop {
            let stream = stream_arc.lock().unwrap().try_clone().unwrap();
            let mut bufreader: BufReader<TcpStream> = BufReader::new(stream);

            let mut line = String::new();

            if bufreader.read_line(&mut line).is_err() {
                error!("Socket I/O failure, disconnecting client");
                break;
            }

            if line.is_empty() {
                warn!("Got empty packet, assuming client disconnect??");
                break;
            }

            let packet: protocol::Packet = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    error!("Packet parsing failed: {}", e);

                    continue;
                }
            };

            trace!("Parsed incoming packet: {:#?}", packet);

            let data = match packet.data {
                Some(ref e) => e,
                None => {
                    warn!("Packet has no payload, ignoring");
                    continue;
                }
            };

            match packet.packet_type {
                protocol::PacketType::Handshake => {
                    debug!("Got handshake from root server");
                }
                protocol::PacketType::RootConfiguration => {
                    let relay_ip_raw = data.get("NextRelayAddress");

                    if relay_ip_raw.is_none() {
                        warn!("Root server sent malformed packet, missing NextRelayAddress");
                        continue;
                    }

                    let relay_ip = relay_ip_raw.unwrap().as_str().unwrap();

                    info!(
                        "Got root configuration from server, next ip address is: {}",
                        relay_ip
                    );

                    if !next_ip.lock().unwrap().is_empty() {
                        error!("Refusing to overwrite old IP address");
                    }

                    *next_ip.lock().unwrap() = relay_ip.to_string();
                }
                protocol::PacketType::RelayFile => {
                    debug!("Relaying file to next server: {}", next_ip.lock().unwrap());

                    let mut stream =
                        match TcpStream::connect(next_ip.lock().unwrap().to_string().trim()) {
                            Ok(v) => {
                                trace!("Connected to next relay");
                                v
                            }
                            Err(err) => {
                                error!("Failed to connect to next relay, file ends here. {}", err);
                                break;
                            }
                        };

                    std::thread::sleep(Duration::from_secs(2));

                    packet.send(&mut stream);
                }
                protocol::PacketType::InjectFile => {
                    error!("Injecting files can only be done at the root server level!");
                }
            }
        }
    }

    pub fn mainloop(&mut self) -> Result<(), std::io::Error> {
        let next_ip = Arc::new(Mutex::new(String::new()));
        loop {
            let (stream, addr) = self.listener.accept()?;

            info!("Client connected with address: {:#?}", addr);

            let stream_cloned = Arc::new(Mutex::new(stream));
            let next_ip_cloned = next_ip.clone();

            std::thread::spawn(move || {
                RelayServer::spawn_client(stream_cloned, next_ip_cloned);
            });
        }
    }
}
