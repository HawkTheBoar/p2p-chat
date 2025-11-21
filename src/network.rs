use futures::StreamExt;
use libp2p::{
    PeerId, StreamProtocol, Swarm,
    identity::{
        Keypair,
        ed25519::{self, PublicKey},
    },
    mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use crate::network::{
    chat::{ChatCommand, DirectMessageRequest, DirectMessageResponse, Message, MessageResponse},
    friends::FriendCommand,
    signable::Signable,
};

pub mod chat;
pub mod friends;
pub mod signable;

pub enum Command {
    ChatCommand(ChatCommand),
    FriendCommand(FriendCommand),
}
pub(crate) async fn new(
    identities: Arc<Mutex<HashMap<PeerId, PublicKey>>>,
) -> (EventLoop, Client, mpsc::Receiver<Event>) {
    // TODO: Confiugre properly & handle errors
    // Dont generate identities on every run, create a store
    let id = Keypair::generate_ed25519();
    let pk = id.public();
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .unwrap()
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
            let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "1.0".to_string(),
                pk,
            ));
            Ok(Behaviour {
                mdns,
                direct_message,
                identify,
            })
        })
        .unwrap()
        .build();
    // Listen on all interfaces and whatever port the OS assigns
    swarm
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
        .unwrap();
    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .unwrap();
    let (command_tx, command_rx) = mpsc::channel(100);
    let (event_tx, event_rx) = mpsc::channel(100);
    let client = Client {
        command_sender: command_tx,
        id,
    };
    let event_loop = EventLoop::new(swarm, command_rx, event_tx, identities);
    (event_loop, client, event_rx)
}
#[derive(Debug)]
pub(crate) enum Event {
    InboundMessage { message: Message },
    OutboundMessageReceived { message_id: i32 },
    OutboundMessageInvalidSignature { message_id: i32 },
}
#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: mdns::tokio::Behaviour,
    direct_message:
        libp2p::request_response::cbor::Behaviour<DirectMessageRequest, DirectMessageResponse>,
    identify: libp2p::identify::Behaviour,
}
pub struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_rx: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
    identities: Arc<Mutex<HashMap<PeerId, PublicKey>>>,
}
#[derive(Clone)]
pub(crate) struct Client {
    pub command_sender: mpsc::Sender<Command>,
    id: Keypair,
}
impl EventLoop {
    fn new(
        swarm: Swarm<Behaviour>,
        command_rx: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<Event>,
        identities: Arc<Mutex<HashMap<PeerId, PublicKey>>>,
    ) -> Self {
        EventLoop {
            swarm,
            command_rx,
            event_sender,
            identities,
        }
    }
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                Some(command) = self.command_rx.recv() => {
                    match command {
                        Command::ChatCommand(chat) => self.handle_chat_command(chat).await,
                        Command::FriendCommand(friend) => self.handle_friend_command(friend).await,
                    }
                },
            }
        }
    }
    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Identify(
                libp2p::identify::Event::Received {
                    connection_id,
                    peer_id,
                    info,
                },
            )) => {
                let pk = info.public_key;
                if let Ok(ed) = pk.try_into_ed25519() {
                    self.identities.lock().await.insert(peer_id, ed);
                } else {
                    println!("Identified peer public key is not ed25519");
                };
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("{peer_id} peer connected!")
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _multiaddr) in list {}
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Local node is listening on {address}");
            }

            SwarmEvent::Behaviour(BehaviourEvent::DirectMessage(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request { request, .. } => {
                    // TODO: remove this unwrap
                    let sender: PeerId = request.0.sender.to_owned().parse().unwrap();
                    let verified = match self.identities.lock().await.get(&sender) {
                        Some(pk) => request.0.verify(pk),
                        None => todo!(),
                    };
                    if !verified {
                        println!("Message failed to verify");
                        return;
                    }
                    // if message is valid, send
                    self.event_sender
                        .send(Event::InboundMessage { message: request.0 })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                request_response::Message::Response { response, .. } => match response {
                    DirectMessageResponse(MessageResponse::ACK { message_id }) => {
                        self.event_sender
                            .send(Event::OutboundMessageReceived { message_id })
                            .await
                            .expect("Event receiver not to be dropped.");
                    }
                    DirectMessageResponse(MessageResponse::InvalidSignature { message_id }) => {
                        self.event_sender
                            .send(Event::OutboundMessageInvalidSignature { message_id })
                            .await
                            .expect("Event receiver not to be dropped");
                    }
                },
            },
            _ => {}
        }
    }
}
