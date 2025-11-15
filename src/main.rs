mod network;
use futures::stream::StreamExt;
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, gossipsub, mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            let direct_message = libp2p::request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/direct-message/1"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            );
            Ok(MyBehaviour {
                mdns,
                direct_message,
            })
        })?
        .build();

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");
    let mut peers = Vec::<PeerId>::new();
    // Kick it off
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                let peer = peers.first().unwrap();
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        peers.push(peer_id);
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        let peer_index = peers.iter().position(|peer| *peer == peer_id).unwrap();
                        peers.remove(peer_index);
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                }
                _ => {}
            }
        }
    }
}
