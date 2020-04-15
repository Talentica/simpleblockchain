use libp2p::core::{multiaddr::Protocol, Multiaddr, PeerId};
use std::collections::hash_map::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

pub struct PeerData {
    pub id: PeerId,
    pub last_seen: u128,
    pub multi_addr: Multiaddr,
}

impl PeerData {
    pub fn new(peer_id: PeerId, ts: u128, addr: Multiaddr) -> Self {
        PeerData {
            id: peer_id,
            last_seen: ts,
            multi_addr: addr,
        }
    }

    pub fn get_network_addr(&self) -> Result<IpAddr, String> {
        let components: Vec<_> = self.multi_addr.iter().collect();
        match (components[0]) {
            Protocol::Ip4(addr) => return Ok(IpAddr::V4(addr)),
            Protocol::Ip6(addr) => return Ok(IpAddr::V6(addr)),
            _ => Err(String::from("Invalid address")),
        }
    }
}

pub struct GlobalData {
    pub peers: HashMap<String, PeerData>,
}

impl GlobalData {
    pub fn new() -> Self {
        GlobalData {
            peers: HashMap::new(),
        }
    }
}

lazy_static! {
    pub static ref GLOBALDATA: Arc<Mutex<GlobalData>> = Arc::new(Mutex::new(GlobalData::new()));
}
// let time_stamp = SystemTime::now()
// .duration_since(SystemTime::UNIX_EPOCH)
// .unwrap()
// .as_micros();
