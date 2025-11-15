use futures::StreamExt;
use libp2p::{
    Swarm, mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use serde::{Deserialize, Serialize};

pub mod chat;
pub mod friends;

#[derive(Debug, Serialize, Deserialize)]
struct DirectMessageRequest(String);
#[derive(Debug, Serialize, Deserialize)]
struct DirectMessageResponse(i8);

#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: mdns::tokio::Behaviour,
    direct_message:
        libp2p::request_response::cbor::Behaviour<DirectMessageRequest, DirectMessageResponse>,
}
pub struct EventLoop {
    swarm: Swarm<Behaviour>,
}
impl EventLoop {
    fn new(swarm: Swarm<Behaviour>) -> Self {
        EventLoop { swarm }
    }
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await
            }
        }
    }
    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {}
}
