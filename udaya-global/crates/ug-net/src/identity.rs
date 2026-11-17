//! Peer identity: an Ed25519 identity key plus a Noise X25519 static key.
//!
//! The Ed25519 key is the stable peer identity used by the peer store, routing
//! table and consensus allowlists. The X25519 key is used only by Noise and may
//! change between restarts; the Hello signature ties the two together.

use crate::{Error, Result};
use ed25519_dalek::{Signer, SigningKey};
use std::fmt;
use ug_core::crypto::{self, Hash32, SignatureBytes};
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

/// Stable peer identity: the Ed25519 public key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId(pub Hash32);

impl PeerId {
    pub fn from_public_key(key: &Hash32) -> Self {
        Self(*key)
    }

    pub fn as_bytes(&self) -> &Hash32 {
        &self.0
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({})", hex::encode(self.0))
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

/// Local identity material.
#[derive(Clone)]
pub struct LocalIdentity {
    signing: SigningKey,
    noise_secret: StaticSecret,
}

impl LocalIdentity {
    /// Generate a fresh identity from the operating system CSPRNG.
    pub fn generate() -> Self {
        let signing = crypto::generate_key();
        let seed: [u8; 32] = rand::random();
        let noise_secret = StaticSecret::from(derive_noise_secret(&seed));
        Self {
            signing,
            noise_secret,
        }
    }

    /// Deterministic identity for tests and reproducible deployments.
    ///
    /// The Ed25519 key is derived directly from `seed`; the Noise static key is
    /// derived through a domain-separated hash so it is independent of the
    /// signing key. Production nodes should load a CSPRNG-generated seed from
    /// an operator-controlled file, never hardcode one.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            signing: SigningKey::from_bytes(seed),
            noise_secret: StaticSecret::from(derive_noise_secret(seed)),
        }
    }

    pub fn peer_id(&self) -> PeerId {
        PeerId(crypto::public_key(&self.signing))
    }

    pub fn noise_static(&self) -> [u8; 32] {
        X25519Public::from(&self.noise_secret).to_bytes()
    }

    pub(crate) fn noise_secret_bytes(&self) -> [u8; 32] {
        self.noise_secret.to_bytes()
    }

    pub(crate) fn sign_digest(&self, digest: &Hash32) -> SignatureBytes {
        self.signing.sign(digest).to_bytes()
    }

    pub fn verify_digest(peer: &PeerId, signature: &SignatureBytes, digest: &Hash32) -> bool {
        use ed25519_dalek::{Signature, VerifyingKey};
        let Ok(key) = VerifyingKey::from_bytes(&peer.0) else {
            return false;
        };
        key.verify_strict(digest, &Signature::from_bytes(signature))
            .is_ok()
    }
}

impl fmt::Debug for LocalIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LocalIdentity({})", self.peer_id())
    }
}

fn derive_noise_secret(seed: &[u8; 32]) -> [u8; 32] {
    crypto::tagged_hash("ug/p2p-noise-static", &[seed])
}

/// Validate that a peer identity is not the local identity and is not all zero.
pub fn validate_remote_peer(local: &LocalIdentity, remote: &PeerId) -> Result<()> {
    if remote.0 == [0u8; 32] {
        return Err(Error::Handshake("remote peer id is zero".into()));
    }
    if *remote == local.peer_id() {
        return Err(Error::Handshake(
            "refusing a connection to the local peer identity".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_are_stable_and_distinct() {
        let a = LocalIdentity::from_seed(&[1u8; 32]);
        let b = LocalIdentity::from_seed(&[1u8; 32]);
        let c = LocalIdentity::from_seed(&[2u8; 32]);
        assert_eq!(a.peer_id(), b.peer_id());
        assert_eq!(a.noise_static(), b.noise_static());
        assert_ne!(a.peer_id(), c.peer_id());
        assert_ne!(a.noise_static(), c.noise_static());
        assert_ne!(a.peer_id().0, a.noise_static());
    }

    #[test]
    fn signature_binds_the_identity() {
        let a = LocalIdentity::from_seed(&[1u8; 32]);
        let digest = crypto::tagged_hash("test", &[b"hello"]);
        let signature = a.sign_digest(&digest);
        assert!(LocalIdentity::verify_digest(
            &a.peer_id(),
            &signature,
            &digest
        ));
        assert!(!LocalIdentity::verify_digest(
            &a.peer_id(),
            &signature,
            &crypto::tagged_hash("test", &[b"other"])
        ));
    }

    #[test]
    fn self_connections_are_refused() {
        let a = LocalIdentity::from_seed(&[3u8; 32]);
        assert!(validate_remote_peer(&a, &a.peer_id()).is_err());
        assert!(validate_remote_peer(&a, &PeerId([0u8; 32])).is_err());
        assert!(validate_remote_peer(&a, &PeerId([9u8; 32])).is_ok());
    }

    #[test]
    fn debug_does_not_print_secret_material() {
        let text = format!("{:?}", LocalIdentity::from_seed(&[7u8; 32]));
        assert!(text.starts_with("LocalIdentity("));
        assert!(!text.contains("StaticSecret"));
    }
}
