use std::{ffi::c_void, ptr::null};

use esp_idf_svc::sys::{
    esp, esp_efuse_block_t_EFUSE_BLK_KEY1, esp_efuse_desc_t, esp_efuse_get_field_size,
    esp_efuse_key_block_unused, esp_efuse_read_field_blob, esp_fill_random,
};
pub use k256::{
    ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey},
    elliptic_curve::{rand_core::RngCore, sec1::ToEncodedPoint},
};
pub use k256::{PublicKey, SecretKey};
use log::{debug, warn};
use sha3::{Digest, Keccak256};

pub static ZERO_ARRAY_32: [u8; 32] = [0; 32];

pub fn random_key() -> [u8; 32] {
    unsafe {
        let mut buf = [0u8; 32];
        esp_fill_random(buf.as_mut_ptr() as *mut c_void, 32);
        buf
    }
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
        return Some(buf);
    }
}
pub trait KeyArr {
    fn to_secret_key(&self) -> SecretKey;
}

impl KeyArr for [u8; 32] {
    fn to_secret_key(&self) -> SecretKey {
        SecretKey::from_slice(self).unwrap()
    }
}

pub trait SecretKeyExt {
    fn to_verifying_key(&self) -> VerifyingKey;
    fn to_signing_key(&self) -> SigningKey;
    fn to_nostr_pubkey(&self) -> String;
}

impl SecretKeyExt for SecretKey {
    fn to_verifying_key(&self) -> VerifyingKey {
        self.public_key().into()
    }
    fn to_signing_key(&self) -> SigningKey {
        self.into()
    }
    fn to_nostr_pubkey(&self) -> String {
        let pubkey = self.public_key();
        let pubkey_bytes = pubkey.to_encoded_point(true);
        let pubkey_bytes = pubkey_bytes.as_bytes();
        hex::encode(&pubkey_bytes[1..])
    }
}

pub trait VerifyingKeyExt {
    fn to_eth_address_bytes(&self) -> [u8; 20];
    fn to_eth_address_str(&self) -> String;
    fn to_nostr_pubkey_bytes(&self) -> [u8; 32];
    fn to_nostr_pubkey_str(&self) -> String;
}

impl VerifyingKeyExt for VerifyingKey {
    fn to_eth_address_bytes(&self) -> [u8; 20] {
        let key = self.to_encoded_point(false);
        let key = key.as_bytes();
        let mut hasher = Keccak256::default();
        hasher.update(&key[1..]);
        let hash: [u8; 32] = hasher.finalize().into();
        let addr = &hash[12..32];
        addr.try_into().unwrap()
    }
    fn to_eth_address_str(&self) -> String {
        let b = self.to_eth_address_bytes();
        format!("0x{}", hex::encode(b))
    }
    fn to_nostr_pubkey_bytes(&self) -> [u8; 32] {
        let pubkey = self.to_encoded_point(true);
        let pubkey_bytes = pubkey.as_bytes();
        let pubkey_bytes = &pubkey_bytes[1..];
        pubkey_bytes.try_into().unwrap()
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

lazy_static::lazy_static! {
    pub static ref SECRET_KEY: SecretKey = get_device_secret_key();
    pub static ref SIGNER: SigningKey = SECRET_KEY.clone().into();
    pub static ref SIGNER_MOVE: SigningKey = SECRET_KEY.clone().into();
    pub static ref VERIFYING_KEY: VerifyingKey = SECRET_KEY.clone().to_verifying_key();
    pub static ref VERIFYING_KEY_MOVE: VerifyingKey = SECRET_KEY.clone().to_verifying_key();
    pub static ref ETH_ADDRESS_BYTES: [u8; 20] = VERIFYING_KEY.to_eth_address_bytes();
    pub static ref ETH_ADDRESS_STRING: String = VERIFYING_KEY.to_eth_address_str();
    pub static ref ETH_ADDRESS_BYTES_VEC: Vec<u8> = ETH_ADDRESS_BYTES.to_vec();
    pub static ref NOSTR_PUBKEY_BYTES: [u8; 32] = VERIFYING_KEY.to_nostr_pubkey_bytes();
    pub static ref NOSTR_PUBKEY_STRING: String = VERIFYING_KEY.to_nostr_pubkey_str();
    pub static ref NOSTR_PUBKEY_BYTES_VEC: Vec<u8> = NOSTR_PUBKEY_BYTES.to_vec();
}
