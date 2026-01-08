use libp2p::PeerId;
#[derive(Debug, Clone)]
pub struct Message {
    pub content: String,
    pub id: uuid::Uuid,
    pub sender: Contact,
    // date
}
#[derive(Debug, Clone)]
pub struct Contact {
    pub peer_id: PeerId,
    pub name: String,
}
