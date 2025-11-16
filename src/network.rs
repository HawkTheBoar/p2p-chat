use futures::StreamExt;
use libp2p::{
    StreamProtocol, Swarm, mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::network::{
    chat::{ChatCommand, DirectMessageRequest, DirectMessageResponse, Message, MessageResponse},
    friends::FriendCommand,
};

pub mod chat;
pub mod friends;

pub(crate) async fn new() -> (EventLoop, Client, mpsc::Receiver<Event>) {
    // TODO: Confiugre properly & handle errors
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
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
    let (chat_tx, chat_rx) = mpsc::channel(100);
    let (friend_tx, friend_rx) = mpsc::channel(100);
    let (event_tx, event_rx) = mpsc::channel(100);
    let client = Client {
        chat_sender: chat_tx,
        friend_sender: friend_tx,
    };
    let event_loop = EventLoop::new(swarm, chat_rx, friend_rx, event_tx);
    (event_loop, client, event_rx)
}
#[derive(Debug)]
pub(crate) enum Event {
    InboundMessage { message: Message },
    OutboundMessageRead { message_id: i32 },
}
#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: mdns::tokio::Behaviour,
    direct_message:
        libp2p::request_response::cbor::Behaviour<DirectMessageRequest, DirectMessageResponse>,
}
pub struct EventLoop {
    swarm: Swarm<Behaviour>,
    chat: mpsc::Receiver<ChatCommand>,
    friends: mpsc::Receiver<FriendCommand>,
    event_sender: mpsc::Sender<Event>,
}
#[derive(Clone)]
pub(crate) struct Client {
    pub chat_sender: mpsc::Sender<ChatCommand>,
    pub friend_sender: mpsc::Sender<FriendCommand>,
}
impl EventLoop {
    fn new(
        swarm: Swarm<Behaviour>,
        chat: mpsc::Receiver<ChatCommand>,
        friends: mpsc::Receiver<FriendCommand>,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        EventLoop {
            swarm,
            chat,
            friends,
            event_sender,
        }
    }
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                Some(chat_command) = self.chat.recv() => self.handle_chat_command(chat_command).await,
                Some(friend_command) = self.friends.recv() => self.handle_friend_command(friend_command).await
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
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    self.event_sender
                        .send(Event::InboundMessage { message: request.0 })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => match response {
                    DirectMessageResponse(MessageResponse::ACK { message_id }) => {
                        self.event_sender
                            .send(Event::OutboundMessageRead { message_id })
                            .await
                            .expect("Event receiver not to be dropped.");
                    }
                },
            },
            _ => {}
        }
    }
}
