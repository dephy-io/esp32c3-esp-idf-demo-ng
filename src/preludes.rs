pub use crate::build_env::*;
pub use crate::crypto::*;
pub use anyhow::{anyhow, Result};
pub use esp_idf_svc::sys::{esp, EspError};
pub use log::*;
pub use serde::{Deserialize, Serialize};
pub use serde_json::{json, Value};
pub use std::sync::Arc;
pub use std::time::{Duration, SystemTime};

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
