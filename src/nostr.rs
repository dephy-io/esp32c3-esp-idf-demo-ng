use crate::{net::request_text, preludes::*};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEventToSend {
    pub event: NostrEvent,
}

pub type NostrEventTag = Vec<String>;

impl NostrEvent {
    pub fn new(kind: u16, tags: Vec<NostrEventTag>, content: String) -> Self {
        let pubkey = NOSTR_PUBKEY_STRING.clone();
        let created_at = now_secs();

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

pub async fn send_new_event() -> Result<()> {
    let event = NostrEventToSend {
        event: NostrEvent::new(
            EVENT_KIND,
            vec![vec!["s".to_string(), "asdjaksdghjkas".to_string()]],
            "Hello, world!".to_string(),
        ),
    };
    info!("event: {:?}", event);
    let event_json = serde_json::to_string(&event)?;
    let (status, body) = request_text(
        DEPHY_ENDPOINT_HTTP,
        Some(esp_idf_svc::http::Method::Post),
        None,
        Some(event_json.as_bytes()),
    )?;
    info!("status: {}", status);
    info!("body: {}", body);

    if status == 200 {
        info!("event sent");
    } else {
        warn!("Failed to send event: {}", body);
    }
    Ok(())
}
