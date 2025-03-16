mod server;

use clap::{Command, arg};
use log::info;
use server::RelayServer;

fn main() {
    pretty_env_logger::init();

    let cmd = Command::new("relayserver")
        .about("Run the srbnfs relay server")
        .arg(arg!(<PORT> "Port to run the relay server on").required(true));

    let matches = cmd.get_matches();

    let port_num = matches
        .get_one::<String>("PORT")
        .expect("Could not get port number")
        .parse::<u16>()
        .expect("Port must be a u16");

    info!("srbnfs running on protocol v2 in RELAY mode");

    let mut relayserver = RelayServer::new(port_num).expect("Failed to create RelayServer");

    relayserver
        .mainloop()
        .expect("RelayServer mainloop failure!");
}
