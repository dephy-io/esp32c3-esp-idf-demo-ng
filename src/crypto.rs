use crate::preludes::*;
use esp_idf_svc::sys::{
    esp, esp_efuse_block_t_EFUSE_BLK_KEY1, esp_efuse_desc_t, esp_efuse_get_field_size,
    esp_efuse_key_block_unused, esp_efuse_read_field_blob, esp_fill_random,
};
pub use k256::{
    elliptic_curve::{rand_core::RngCore, sec1::ToEncodedPoint},
    schnorr::{Signature, SigningKey, VerifyingKey},
};
pub use k256::{PublicKey, SecretKey};
use std::{ffi::c_void, ptr::null};

pub static ZERO_ARRAY_32: [u8; 32] = [0; 32];

pub fn random_key() -> [u8; 32] {
    unsafe {
        let mut buf = [0u8; 32];
        esp_fill_random(buf.as_mut_ptr() as *mut c_void, 32);
        buf
    }
}

pub fn read_debug_force_privkey() -> Option<[u8; 32]> {
    if DEBUG_FORCE_PRIVKEY.is_empty() {
        return None;
    }
    if DEBUG_FORCE_PRIVKEY.len() != 64 {
        warn!("DEBUG_FORCE_PRIVKEY is not 64 bytes long, ignoring.");
        return None;
    }
    let mut key = [0u8; 32];
    if let Err(e) = hex::decode_to_slice(DEBUG_FORCE_PRIVKEY, &mut key) {
        warn!("Failed to decode DEBUG_FORCE_PRIVKEY: {}", e);
        return None;
    }
    warn!("using key from DEBUG_FORCE_PRIVKEY");
    Some(key)
}

pub fn read_key_from_efuse() -> Option<[u8; 32]> {
    unsafe {
        if esp_efuse_key_block_unused(esp_efuse_block_t_EFUSE_BLK_KEY1) {
            warn!("esp_efuse_block_t_EFUSE_BLK_KEY1 not used.");
            return None;
        }
        let mut desc1 = esp_efuse_desc_t::default();
        desc1.set_efuse_block(esp_efuse_block_t_EFUSE_BLK_KEY1);
        desc1.bit_start = 0;
        desc1.bit_count = 256;

        let mut desc1 = [&desc1 as *const esp_efuse_desc_t, null()];
        let desc1 = desc1.as_mut_ptr();

        let size1 = esp_efuse_get_field_size(desc1);
        debug!("size1: {}", size1);
        if size1 == 0 {
            return None;
        }
        let mut buf = [0u8; 32];
        esp!(esp_efuse_read_field_blob(
            desc1,
            buf.as_mut_ptr() as *mut c_void,
            256
        ))
        .unwrap();
        info!("Got key from efuse");
        return Some(buf);
    }
}
pub trait KeyArr {
    fn to_secret_key(&self) -> SecretKey;
    fn to_signing_key(&self) -> SigningKey;
}

impl KeyArr for [u8; 32] {
    fn to_secret_key(&self) -> SecretKey {
        SecretKey::from_slice(self).unwrap()
    }
    fn to_signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(self).unwrap()
    }
}

pub trait SigningKeyExt {
    fn to_verifying_key(&self) -> VerifyingKey;
}

impl SigningKeyExt for SigningKey {
    fn to_verifying_key(&self) -> VerifyingKey {
        self.verifying_key().clone()
    }
}

pub trait VerifyingKeyExt {
    fn to_nostr_pubkey_bytes(&self) -> [u8; 32];
    fn to_nostr_pubkey_str(&self) -> String;
}

impl VerifyingKeyExt for VerifyingKey {
    fn to_nostr_pubkey_bytes(&self) -> [u8; 32] {
        let bytes = self.to_bytes();
        bytes.try_into().unwrap()
    }
    fn to_nostr_pubkey_str(&self) -> String {
        let b = self.to_nostr_pubkey_bytes();
        hex::encode(b)
    }
}

pub fn get_device_secret_key() -> SecretKey {
    let key = read_key_from_efuse();
    if key.is_none() {
        warn!("No key found in efuse, using random key.");
    }
    let key = key.unwrap_or(random_key());
    key.to_secret_key()
}

pub fn get_device_signing_key() -> SigningKey {
    let mut key = read_debug_force_privkey();
    if key.is_none() {
        key = read_key_from_efuse();
    }
    if key.is_none() {
        warn!("No key found in efuse, using random key.");
    }
    let key = key.unwrap_or(random_key());
    key.to_signing_key()
}

lazy_static::lazy_static! {
    pub static ref SIGNER: SigningKey = get_device_signing_key();
    pub static ref SIGNER_MOVE: SigningKey = SIGNER.clone();
    pub static ref VERIFYING_KEY: VerifyingKey = SIGNER.to_verifying_key();
    pub static ref VERIFYING_KEY_MOVE: VerifyingKey = VERIFYING_KEY.clone();
    pub static ref NOSTR_PUBKEY_BYTES: [u8; 32] = VERIFYING_KEY.to_nostr_pubkey_bytes();
    pub static ref NOSTR_PUBKEY_STRING: String = VERIFYING_KEY.to_nostr_pubkey_str();
    pub static ref NOSTR_PUBKEY_BYTES_VEC: Vec<u8> = NOSTR_PUBKEY_BYTES.to_vec();
}
