use libp2p::PeerId;
use serde::{Deserialize, Serialize};

use crate::network::{Client, EventLoop};
#[derive(Debug, Serialize, Deserialize)]
pub enum FriendRequest {
    RequestName,
    VerifyName { name: String },
    AddFriend,
    AcceptFriend { decision: bool },
}
#[derive(Debug, Serialize, Deserialize)]
pub enum FriendResponse {
    RequestName { name: String },
    VerifyName(Option<String>),
    AddFriendAck,
    AcceptFriendAck,
}
pub enum FriendCommand {
    RequestName { peer: PeerId },
    VerifyName { name: String, peer: PeerId },
    AddFriend { peer: PeerId },
    AcceptFriend { peer: PeerId, decision: bool },
}
impl EventLoop {
    pub async fn handle_friend_command(&mut self, command: FriendCommand) {
        match command {
            FriendCommand::RequestName { peer } => self
                .swarm
                .behaviour_mut()
                .friends
                .send_request(&peer, FriendRequest::RequestName),
            FriendCommand::VerifyName { peer, name } => self
                .swarm
                .behaviour_mut()
                .friends
                .send_request(&peer, FriendRequest::VerifyName { name }),
            FriendCommand::AddFriend { peer } => self
                .swarm
                .behaviour_mut()
                .friends
                .send_request(&peer, FriendRequest::AddFriend),
            FriendCommand::AcceptFriend { peer, decision } => self
                .swarm
                .behaviour_mut()
                .friends
                .send_request(&peer, FriendRequest::AcceptFriend { decision }),
        };
    }
}
impl Client {
    async fn request_name(&mut self, peer: PeerId) {
        self.command_sender
            .send(super::Command::FriendCommand(FriendCommand::RequestName {
                peer,
            }))
            .await
            .expect("to send request");
    }
    async fn verify_name(&mut self, peer: PeerId, name: String) {
        self.command_sender
            .send(super::Command::FriendCommand(FriendCommand::VerifyName {
                name,
                peer,
            }))
            .await
            .expect("to send request");
    }
    async fn send_friend_request(&mut self, peer: PeerId) {
        self.command_sender
            .send(super::Command::FriendCommand(FriendCommand::AddFriend {
                peer,
            }))
            .await
            .expect("to send request");
    }
    async fn accept_friend_req(&mut self, peer: PeerId) {
        self.command_sender
            .send(super::Command::FriendCommand(FriendCommand::AcceptFriend {
                peer,
                decision: true,
            }))
            .await
            .expect("to send request");
    }
    async fn deny_friend_req(&mut self, peer: PeerId) {
        self.command_sender
            .send(super::Command::FriendCommand(FriendCommand::AcceptFriend {
                peer,
                decision: false,
            }))
            .await
            .expect("to send request");
    }
}

// Name exchange -- Will occur when there is no name linked to PubKey
// What is your name?
// My name is: xxxx
// acknowledged

// Name verification -- Will reoccur based on ttl values
// Is your name still xxxx?
// yes / no

// Friend request
// I wanna be ur friend
// request acknowledged

// AcceptFriendRequest
// I want / dont want to be ur friend
// acknowledged
