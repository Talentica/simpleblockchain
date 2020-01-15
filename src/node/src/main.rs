use libp2p::PeerId;
use p2plib::simpleswarm::SimpleSwarm;
use utils::configreader;
use utils::configreader::Configuration;

fn main() {
    let config: &Configuration = &configreader::GLOBAL_CONFIG;
    let peer_id = PeerId::from_public_key(config.node.public.clone());
    println!("peer id = {:?}", peer_id);
    SimpleSwarm::process(peer_id, config);
    // let transport = libp2p::build_development_transport(libp2p::identity::Keypair::Ed25519(
}
