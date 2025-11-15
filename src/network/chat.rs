use crate::network::EventLoop;
pub enum ChatCommand {}
impl EventLoop {
    async fn handle_chat_command(&mut self, command: ChatCommand) {}
}
