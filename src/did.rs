//! Ed25519 `did:key` identities: create, sign, and verify.
//!
//! ```text
//! did = "did:key:z" + base58btc(0xED01 || raw_public_key_32)
//! ```
//! Messages are signed over `"{room}|{nonce}|{text}"` and transported as
//! unpadded base64url.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::time::{SystemTime, UNIX_EPOCH};

const MULTICODEC_ED25519: [u8; 2] = [0xed, 0x01];

/// An Ed25519 signing identity addressed by a `did:key`.
pub struct Identity {
    signing: SigningKey,
    /// The `did:key:z6Mk…` identifier.
    pub did: String,
}

impl Identity {
    /// Create a new random identity.
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).expect("system RNG unavailable");
        Self::from_seed(&seed)
    }

    /// Build an identity from a 32-byte seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing = SigningKey::from_bytes(seed);
        let did = encode_did(&signing.verifying_key().to_bytes());
        Self { signing, did }
    }

    /// Build an identity from a 32-byte seed encoded as hex.
    pub fn from_seed_hex(seed_hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(seed_hex.trim()).map_err(|e| e.to_string())?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| "seed must be 32 bytes".to_string())?;
        Ok(Self::from_seed(&arr))
    }

    /// The private seed as hex. Keep it secret.
    pub fn seed_hex(&self) -> String {
        hex::encode(self.signing.to_bytes())
    }

    /// Sign the canonical `room|nonce|text` payload, returning base64url.
    pub fn sign(&self, room: &str, nonce: &str, text: &str) -> String {
        let sig = self.signing.sign(format!("{room}|{nonce}|{text}").as_bytes());
        URL_SAFE_NO_PAD.encode(sig.to_bytes())
    }
}

/// A strictly increasing nanosecond nonce.
pub fn fresh_nonce() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}

/// Encode a raw 32-byte Ed25519 public key as a `did:key`.
pub fn encode_did(public_key: &[u8]) -> String {
    let mut body = MULTICODEC_ED25519.to_vec();
    body.extend_from_slice(public_key);
    format!("did:key:z{}", bs58::encode(body).into_string())
}

/// Recover the raw 32-byte Ed25519 public key from a `did:key`.
pub fn decode_did(did: &str) -> Result<[u8; 32], String> {
    let rest = did.strip_prefix("did:key:z").ok_or("not a did:key identifier")?;
    let decoded = bs58::decode(rest).into_vec().map_err(|e| e.to_string())?;
    if decoded.len() < 2 || decoded[0] != 0xed || decoded[1] != 0x01 {
        return Err("did:key is not an Ed25519 key".into());
    }
    decoded[2..].try_into().map_err(|_| "bad key length".to_string())
}

/// Verify a signed technocore message.
pub fn verify(did: &str, room: &str, nonce: &str, text: &str, signature: &str) -> bool {
    (|| -> Option<bool> {
        let pk = decode_did(did).ok()?;
        let vk = VerifyingKey::from_bytes(&pk).ok()?;
        let raw = URL_SAFE_NO_PAD.decode(signature).ok()?;
        let sig = Signature::from_slice(&raw).ok()?;
        Some(vk.verify(format!("{room}|{nonce}|{text}").as_bytes(), &sig).is_ok())
    })()
    .unwrap_or(false)
}
