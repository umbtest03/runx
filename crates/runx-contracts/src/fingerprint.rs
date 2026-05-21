//! Fingerprint contracts: content hashing identifiers.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::Reference;

/// Lowercase hex encoding of raw bytes.
#[must_use]
pub fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

/// SHA-256 of the input bytes as lowercase hex (no prefix).
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex_lower(&Sha256::digest(bytes))
}

/// SHA-256 of the input bytes, prefixed with the `sha256:` algorithm tag.
#[must_use]
pub fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FingerprintAlgorithm {
    Sha256,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fingerprint {
    pub algorithm: FingerprintAlgorithm,
    pub canonicalization: String,
    pub value: String,
    pub derived_from: Vec<Reference>,
}
