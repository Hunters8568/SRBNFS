use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::net::TcpStream;

use clap::{Command, arg};
use serde_json::json;

fn main() {
    let cmd = Command::new("srbnfs")
        .about("Interact with srbnfs root servers")
        .author("Hunter s + antiLimited [https://github.com/Hunters8568/SRBNFS]")
        .subcommand(
            Command::new("injectfile")
                .about("Inject a file into the network file system at the given root server")
                .arg(arg!(<FILE> "File to inject").required(true))
                .arg(arg!(<ROOT_SERVER_ADDRESS> "Address of the root server").required(true)),
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("injectfile", sub)) => {
            let server = sub
                .get_one::<String>("ROOT_SERVER_ADDRESS")
                .expect("Failed to get root server address");

            let file_path = sub
                .get_one::<String>("FILE")
                .expect("Failed to get file to inject");

            println!("Injecting file at: {} from ./{}", server, file_path);

            let file_content = std::fs::read_to_string(file_path).expect("Failed to reader");
            let mut stream =
                TcpStream::connect(server).expect("Failed to connect to remote server");

            let mut packet = protocol::Packet::new(protocol::PacketType::InjectFile, true);

            packet.data = Some(json!({
                "FileName": file_path,
                "FileContent": STANDARD.encode(file_content.as_bytes())
            }));

            packet.send(&mut stream);
        }
        _ => unreachable!(),
    };
}
