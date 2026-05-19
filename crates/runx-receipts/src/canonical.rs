use runx_contracts::HarnessReceipt;
use sha2::{Digest, Sha256};

use crate::ReceiptError;

#[must_use]
pub fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", hex_lower(&digest))
}

pub fn canonical_receipt_json(receipt: &HarnessReceipt) -> Result<String, ReceiptError> {
    let value = serde_json::to_value(receipt).map_err(|source| ReceiptError::Serialization {
        message: source.to_string(),
    })?;
    serde_json::to_string(&value).map_err(|source| ReceiptError::Serialization {
        message: source.to_string(),
    })
}

pub fn canonical_receipt_digest(receipt: &HarnessReceipt) -> Result<String, ReceiptError> {
    canonical_receipt_json(receipt).map(|json| sha256_prefixed(json.as_bytes()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::sha256_prefixed;

    #[test]
    fn sha256_prefixes_digest() {
        assert_eq!(
            sha256_prefixed(b"runx"),
            "sha256:8186b7035bea2f66ebe27c1f5cf7de4e94ef935e259a2f3160352adffc752f28"
        );
    }
}
