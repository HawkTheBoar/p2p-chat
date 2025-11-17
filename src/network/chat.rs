use crate::network::Command;
use crate::network::{Client, EventLoop};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectMessageRequest(pub Message);
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectMessageResponse(pub MessageResponse);

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub sender: String,
    pub content: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum MessageResponse {
    ACK { message_id: i32 },
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
    pub async fn send_message(&mut self, peer: PeerId, message: Message) {
        self.command_sender
            .send(Command::ChatCommand(ChatCommand::SendMessage {
                receiver: peer,
                message,
            }))
            .await
            .expect("To send message");
    }
}
