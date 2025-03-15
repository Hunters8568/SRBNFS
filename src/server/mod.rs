use log::*;
use std::io::BufRead;

pub struct Server {
    pub listener: std::net::TcpListener,
}
pub struct Client {
    pub stream: std::net::TcpStream,
}

impl Client {
    pub fn handle(&mut self) {
        debug!("Client handler start!");

        let mut stream = std::io::BufReader::new(&self.stream);

        loop {
            let mut line_buffer = String::new();

            if stream.read_line(&mut line_buffer).is_err() {
                error!("Failed to read line from BufReader, closing client connection!");
                break;
            }

            debug!("{}", line_buffer);
        }
    }
}
