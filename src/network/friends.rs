use crate::network::{Client, EventLoop};
pub enum FriendCommand {}
impl EventLoop {
    pub async fn handle_friend_command(&mut self, command: FriendCommand) {}
}
impl Client {}
