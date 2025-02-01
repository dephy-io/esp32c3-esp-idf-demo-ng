use crate::{net::request_text, preludes::*};
use sha2::{Digest, Sha256};
use signature::Signer;

pub const EVENT_KIND: u16 = 1573;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEventToComputeId(
    pub u8,
    pub String,
    pub u64,
    pub u16,
    pub Vec<NostrEventTag>,
    pub String,
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEvent {
    pub id: String,
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u16,
    pub tags: Vec<NostrEventTag>,
    pub content: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEventToSend {
    pub event: NostrEvent,
}

pub type NostrEventTag = Vec<String>;

impl NostrEvent {
    pub fn new(kind: u16, tags: Vec<NostrEventTag>, content: String) -> Self {
        let pubkey = NOSTR_PUBKEY_STRING.clone();
        let created_at = now_secs();

        let id_e = NostrEventToComputeId(0, pubkey, created_at, kind, tags, content);
        let id_e_json = serde_json::to_string(&id_e).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(id_e_json.as_bytes());
        let hash = hasher.finalize();
        let id = hex::encode(&hash);

        let sigst = SIGNER_MOVE.sign(id_e_json.as_bytes()); // SHA256 hash is done within the sign function
        let sig = sigst.to_bytes();
        let sig = hex::encode(sig);

        Self {
            id,
            pubkey: id_e.1,
            created_at,
            kind,
            tags: id_e.4,
            content: id_e.5,
            sig,
        }
    }
}

pub async fn send_new_event(event: NostrEvent) -> Result<()> {
    let event = NostrEventToSend { event };
    let event_json = serde_json::to_string(&event)?;
    let (status, body) = request_text(
        DEPHY_ENDPOINT_HTTP,
        Some(esp_idf_svc::http::Method::Post),
        None,
        Some(event_json.as_bytes()),
    )?;

    if status == 200 {
        info!("Event sent");
    } else {
        warn!("Failed to send event: {}", body);
    }
    Ok(())
}

pub fn create_random_event() -> NostrEvent {
    NostrEvent::new(
        EVENT_KIND,
        vec![
            vec!["s".to_string(), "0".to_string()],
            vec![
                "p".to_string(),
                "6f7bb11c04d792784c9dfcb4246e9afc0d6a7eae549531c2fce51adf09b2887e".to_string(),
            ],
        ],
        hex::encode(random_key()),
    )
}
