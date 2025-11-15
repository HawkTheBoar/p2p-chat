use crate::network::EventLoop;
pub enum FriendCommand {}
impl EventLoop {
    async fn handle_friend_command(&mut self, command: FriendCommand) {}
}
