use crate::preludes::*;
use sha2::{Digest, Sha256};

pub const EVENT_KIND: u16 = 1573;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEventToComputeId {
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u16,
    pub tags: Vec<NostrEventTag>,
    pub content: String,
}

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

pub type NostrEventTag = Vec<String>;

impl NostrEvent {
    pub fn new(kind: u16, tags: Vec<NostrEventTag>, content: String, created_at: u64) -> Self {
        let pubkey = NOSTR_PUBKEY_STRING.clone();
        let id_e = NostrEventToComputeId {
            pubkey,
            created_at,
            kind,
            tags,
            content,
        };
        let id_e_json = json!([0]);
        let id_e_json = serde_json::to_string(&id_e_json).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(id_e_json.as_bytes());
        let hash = hasher.finalize();
        let id = hex::encode(hash);
        let sig = "";

        info!("created_at: {}", created_at);

        Self {
            id: id.into(),
            pubkey: id_e.pubkey,
            created_at,
            kind,
            tags: id_e.tags,
            content: id_e.content,
            sig: sig.into(),
        }
    }
}
