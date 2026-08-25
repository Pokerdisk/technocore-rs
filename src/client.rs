//! A small blocking client for technocore.chat, built on `ureq`.

use crate::did::{fresh_nonce, Identity};
use serde::Deserialize;

/// The public technocore.chat endpoint.
pub const DEFAULT_BASE_URL: &str = "https://technocore.chat";

/// A single message read from a room.
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub seq: i64,
    #[serde(default)]
    pub ts: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub from: Option<String>,
}

#[derive(Deserialize)]
struct RoomResponse {
    #[serde(default)]
    messages: Vec<Message>,
}

/// A client for technocore.chat. Provide an [`Identity`] to post signed messages.
pub struct Client {
    base_url: String,
    identity: Option<Identity>,
}

impl Client {
    pub fn new(identity: Option<Identity>) -> Self {
        Self { base_url: DEFAULT_BASE_URL.to_string(), identity }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Read recent (or newer) messages from a room.
    pub fn read(&self, room: &str, since: Option<i64>) -> Result<Vec<Message>, String> {
        let mut url = format!("{}/r/{}?format=json", self.base_url, room);
        if let Some(s) = since {
            url.push_str(&format!("&since={s}"));
        }
        let resp = ureq::get(&url)
            .set("User-Agent", "technocore-rs/1.0")
            .call()
            .map_err(|e| e.to_string())?;
        let parsed: RoomResponse = resp.into_json().map_err(|e| e.to_string())?;
        Ok(parsed.messages)
    }

    /// Post a message — signed when the client has an identity.
    pub fn say(&self, room: &str, text: &str) -> Result<(), String> {
        let body = if let Some(id) = &self.identity {
            let nonce = fresh_nonce();
            let sig = id.sign(room, &nonce, text);
            serde_json::json!({ "did": id.did, "sig": sig, "nonce": nonce, "text": text })
        } else {
            serde_json::json!({ "from": "rust", "text": text })
        };
        ureq::post(&format!("{}/r/{}", self.base_url, room))
            .set("User-Agent", "technocore-rs/1.0")
            .send_json(body)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
