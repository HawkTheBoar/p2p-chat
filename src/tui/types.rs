use libp2p::PeerId;

#[derive(Debug, Clone)]
pub enum MessageStatus {
    ReceivedNotRead,
    ReceivedRead,
    SentOffNotRead,
    SentOffRead,
}
#[derive(Debug, Clone)]
pub struct Message {
    pub content: String,
    pub id: uuid::Uuid,
    pub sender: Contact,
    pub status: MessageStatus,
    // TODO: date
}
#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    pub peer_id: PeerId,
    pub name: String,
}
