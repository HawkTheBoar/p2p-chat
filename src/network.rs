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
            Ok(Behaviour {
                mdns,
                direct_message,
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
    let event_loop = EventLoop::new(swarm, command_rx, event_tx);
    (event_loop, client, event_rx)
}
#[derive(Debug)]
pub(crate) enum Event {
    InboundMessage { message: Message, sender: PublicKey },
    OutboundMessageReceived { message_id: i32 },
    OutboundMessageInvalidSignature { message_id: i32 },
}
#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: mdns::tokio::Behaviour,
    direct_message:
        libp2p::request_response::cbor::Behaviour<DirectMessageRequest, DirectMessageResponse>,
}
pub struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_rx: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
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
    ) -> Self {
        EventLoop {
            swarm,
            command_rx,
            event_sender,
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
                    let (message, sender) = request.0.verify().expect("to be verified");
                    // if message is valid, send
                    self.event_sender
                        .send(Event::InboundMessage { message, sender })
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
