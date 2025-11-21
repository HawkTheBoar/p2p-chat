use libp2p::identity::{Keypair, ed25519};

pub trait Signable {
    fn sign(self, keypair: Keypair) -> Self;
    fn verify(&self, public_key: &ed25519::PublicKey) -> bool;
}
