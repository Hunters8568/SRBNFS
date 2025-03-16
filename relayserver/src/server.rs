use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
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

    pub fn spawn_client(stream_arc: Arc<TcpStream>) {
        trace!("Started client thread");

        loop {
            let stream = stream_arc.try_clone().unwrap();
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

            debug!("{}", line);
        }
    }

    pub fn mainloop(&mut self) -> Result<(), std::io::Error> {
        loop {
            let (stream, addr) = self.listener.accept()?;

            info!("Client connected with address: {:#?}", addr);

            let stream_cloned = Arc::new(stream);

            std::thread::spawn(move || {
                RelayServer::spawn_client(stream_cloned);
            });
        }
    }
}
