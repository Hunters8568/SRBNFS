use log::{error, trace};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PacketType {
    Handshake,
    RootConfiguration,
    RelayFile,
    InjectFile,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtocolInfo {
    pub version: String,
    pub github_url: String,
}

impl ProtocolInfo {
    pub fn new() -> Self {
        Self {
            version: "2.0".to_string(),
            github_url: "https://github.com/Hunters8568/SRBNFS".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub packet_type: PacketType,
    pub protocol_info: Option<ProtocolInfo>,
    pub data: Option<Value>,
}

impl Packet {
    pub fn new(packet_type: PacketType, include_protoinfo: bool) -> Self {
        Self {
            packet_type,
            protocol_info: if include_protoinfo {
                Some(ProtocolInfo::new())
            } else {
                None
            },
            data: None,
        }
    }

    pub fn send(self, stream: &mut std::net::TcpStream) {
        match stream.write(format!("{}\n", serde_json::to_string(&self).unwrap()).as_bytes()) {
            Ok(_) => {
                trace!("Transported packet: {:?}", self);
            }
            Err(err) => {
                error!("Failed to transport packet: {:#?} {}", self, err);
            }
        }
    }
}
