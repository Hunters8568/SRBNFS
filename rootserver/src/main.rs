use log::{debug, info};
use server::RootServer;

mod server;

fn main() {
    pretty_env_logger::init();

    info!("srbnfs running on protocol v2 in ROOTSERVER mode");

    info!("Loading ring from ./cfg/ring.txt");

    let ring_cfg_content =
        std::fs::read_to_string("cfg/ring.txt").expect("Failed to read ring config");

    let mut ring_split: Vec<&str> = ring_cfg_content.split("\n").collect();
    ring_split.pop();

    let mut ringbuf =
        shared::ringbuffer::RingBuffer::new(ring_split.iter().map(|x| x.to_string()).collect());

    debug!("Loaded ring: {:#?}", ring_split);

    ringbuf.next(); // Skip root server

    let mut root_server = RootServer::new(7848, ringbuf).expect("Failed to bind");

    root_server.mainloop().expect("RootServer logic failure");
}
