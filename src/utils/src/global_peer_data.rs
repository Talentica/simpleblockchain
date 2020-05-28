use libp2p::core::{multiaddr::Protocol, Multiaddr, PeerId};
use std::collections::hash_map::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
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
        match components[0] {
            Protocol::Ip4(addr) => return Ok(IpAddr::V4(addr)),
            Protocol::Ip6(addr) => return Ok(IpAddr::V6(addr)),
            _ => return Err(String::from("Invalid address")),
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

#[cfg(test)]
mod tests_peer_data {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::net::Ipv4Addr;
    use std::thread;
    use std::time::Duration;
    // fn to test insert & PeerData new function
    #[test]
    fn test_add_global_peer_data() {
        let peer_id: String = "127.0.0.1".to_string();
        let time_stamp: u128 = 123445;
        let addr: Multiaddr = Multiaddr::empty();
        let id: PeerId = PeerId::random();
        let peer_data: PeerData = PeerData::new(id, time_stamp, addr);
        {
            GLOBALDATA.lock().unwrap().peers.insert(peer_id, peer_data);
        }
        assert_eq!(
            GLOBALDATA.lock().unwrap().peers.len(),
            1,
            "insertion error in global peer data"
        );
    }

    // fn to test PeerData getNetworkAddr function
    #[test]
    fn test_get_global_peer_data() {
        let peer_id: String = "127.0.0.1".to_string();
        let time_stamp: u128 = 123445;
        let addr: Multiaddr = Multiaddr::from(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let id: PeerId = PeerId::random();
        let peer_data: PeerData = PeerData::new(id, time_stamp, addr);
        thread::sleep(Duration::from_millis(1000));
        {
            GLOBALDATA
                .lock()
                .unwrap()
                .peers
                .insert(peer_id.clone(), peer_data);
        }
        {
            let mut lock_global_peer_data = GLOBALDATA.lock().unwrap();
            let peer_data: &mut PeerData = lock_global_peer_data.peers.get_mut(&peer_id).unwrap();
            let get_network_addr_result = peer_data.get_network_addr();
            assert!(get_network_addr_result.is_ok());
        }
    }
}
