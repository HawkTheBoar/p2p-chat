use crate::network::Command;
use crate::network::signable::Signable;
use crate::network::{Client, EventLoop};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectMessageRequest(pub Message);
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectMessageResponse(pub MessageResponse);

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub sender: String,
    pub content: String,
    pub id: Uuid,
    pub signature: Option<Vec<u8>>,
}
impl Signable for Message {
    fn sign(self, keypair: libp2p::identity::Keypair) -> Self {
        let data = serde_json::to_vec(&self.content).expect("Serialization failed");
        let signature = keypair.sign(&data).expect("Signature failed");
        Self {
            sender: self.sender,
            content: self.content,
            id: self.id,
            signature: Some(signature),
        }
    }
    fn verify(&self, public_key: &libp2p::identity::ed25519::PublicKey) -> bool {
        let data = serde_json::to_vec(&self.content).expect("Serialization failed");
        if let Some(sig) = &self.signature {
            public_key.verify(&data, sig)
        } else {
            false
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageResponse {
    ACK { message_id: i32 },
    InvalidSignature { message_id: i32 },
}
pub enum ChatCommand {
    SendMessage { receiver: PeerId, message: Message },
    ReadMessage { receiver: PeerId },
}
impl EventLoop {
    pub async fn handle_chat_command(&mut self, command: ChatCommand) {
        match command {
            ChatCommand::SendMessage { receiver, message } => {
                self.swarm
                    .behaviour_mut()
                    .direct_message
                    .send_request(&receiver, DirectMessageRequest(message));
            }
            ChatCommand::ReadMessage { receiver } => {
                todo!()
                // self.swarm
                //     .behaviour_mut()
                //     .direct_message
                //     .send_request(&receiver, DirectMessageRequest(1));
            }
        }
    }
}
impl Client {
    pub async fn send_message(&mut self, receiver: PeerId, message: String) {
        let message = Message {
            sender: self.id.public().to_peer_id().to_string(),
            content: message,
            id: uuid::Uuid::new_v4(),
            signature: None,
        };
        let message = message.sign(self.id.clone());
        self.command_sender
            .send(Command::ChatCommand(ChatCommand::SendMessage {
                receiver,
                message,
            }))
            .await
            .expect("To send message");
    }
}
