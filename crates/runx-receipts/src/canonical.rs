use std::collections::BTreeMap;

use runx_contracts::{HarnessReceipt, JsonNumber, JsonValue, sha256_prefixed};

use crate::ReceiptError;

pub fn canonical_receipt_json(receipt: &HarnessReceipt) -> Result<String, ReceiptError> {
    let value = receipt_json(receipt)?;
    canonical_json_value(&value)
}

pub fn canonical_receipt_digest(receipt: &HarnessReceipt) -> Result<String, ReceiptError> {
    canonical_receipt_json(receipt).map(|json| sha256_prefixed(json.as_bytes()))
}

pub fn canonical_receipt_body_json(receipt: &HarnessReceipt) -> Result<String, ReceiptError> {
    let mut value = receipt_json(receipt)?;
    strip_body_proof_fields(&mut value, true);
    canonical_json_value(&value)
}

pub fn canonical_receipt_body_digest(receipt: &HarnessReceipt) -> Result<String, ReceiptError> {
    canonical_receipt_body_json(receipt).map(|json| sha256_prefixed(json.as_bytes()))
}

fn receipt_json(receipt: &HarnessReceipt) -> Result<JsonValue, ReceiptError> {
    let value = serde_json::to_value(receipt).map_err(|source| ReceiptError::Serialization {
        message: source.to_string(),
    })?;
    serde_json::from_value(value).map_err(|source| ReceiptError::Serialization {
        message: source.to_string(),
    })
}

fn strip_body_proof_fields(value: &mut JsonValue, is_root: bool) {
    match value {
        JsonValue::Object(map) => {
            if is_root {
                map.remove("signature");
            }
            if let Some(JsonValue::Object(seal)) = map.get_mut("seal") {
                seal.remove("digest");
                seal.remove("verification_summary");
            }
            for child in map.values_mut() {
                strip_body_proof_fields(child, false);
            }
        }
        JsonValue::Array(items) => {
            for item in items {
                strip_body_proof_fields(item, false);
            }
        }
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {}
    }
}

fn canonical_json_value(value: &JsonValue) -> Result<String, ReceiptError> {
    match value {
        JsonValue::Null => Ok("null".to_owned()),
        JsonValue::Bool(value) => Ok(value.to_string()),
        JsonValue::Number(value) => Ok(canonical_json_number(value)),
        JsonValue::String(value) => {
            serde_json::to_string(value).map_err(|source| ReceiptError::Serialization {
                message: source.to_string(),
            })
        }
        JsonValue::Array(items) => {
            let body = items
                .iter()
                .map(canonical_json_value)
                .collect::<Result<Vec<_>, _>>()?
                .join(",");
            Ok(format!("[{body}]"))
        }
        JsonValue::Object(map) => {
            let ordered = map.iter().collect::<BTreeMap<_, _>>();
            let body = ordered
                .into_iter()
                .map(|(key, value)| {
                    let key = serde_json::to_string(key).map_err(|source| {
                        ReceiptError::Serialization {
                            message: source.to_string(),
                        }
                    })?;
                    Ok(format!("{key}:{}", canonical_json_value(value)?))
                })
                .collect::<Result<Vec<_>, ReceiptError>>()?
                .join(",");
            Ok(format!("{{{body}}}"))
        }
    }
}

fn canonical_json_number(value: &JsonNumber) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use runx_contracts::HarnessReceipt;
    use runx_contracts::JsonValue;
    use serde::Deserialize;

    use super::{
        ReceiptError, canonical_receipt_body_digest, canonical_receipt_body_json,
        canonical_receipt_digest, canonical_receipt_json, sha256_prefixed,
    };

    const SUCCESS_RECEIPT: &str =
        include_str!("../../../fixtures/contracts/harness-spine/harness-receipt-success.json");

    #[derive(Debug, Deserialize)]
    struct Fixture {
        expected: HarnessReceipt,
    }

    #[test]
    fn sha256_prefixes_digest() {
        assert_eq!(
            sha256_prefixed(b"runx"),
            "sha256:8186b7035bea2f66ebe27c1f5cf7de4e94ef935e259a2f3160352adffc752f28"
        );
    }

    #[test]
    fn canonical_receipt_json_is_stable_and_sorted() -> Result<(), ReceiptError> {
        let receipt = fixture()?;
        let first = canonical_receipt_json(&receipt)?;
        let second = canonical_receipt_json(&receipt)?;

        assert_eq!(first, second);
        assert!(first.starts_with(r#"{"created_at":"#));
        assert!(canonical_receipt_digest(&receipt)?.starts_with("sha256:"));
        Ok(())
    }

    #[test]
    fn body_commitment_excludes_signature_and_seal_derived_fields() -> Result<(), ReceiptError> {
        let mut receipt = fixture()?;
        let baseline_json = canonical_receipt_body_json(&receipt)?;
        let baseline_digest = canonical_receipt_body_digest(&receipt)?;

        receipt.signature.value = "base64:changed".to_owned();
        receipt.seal.digest = "sha256:changed".to_owned();
        if let Some(summary) = receipt.seal.verification_summary.as_mut() {
            summary.signature_valid = false;
        }
        if let Some(seal) = receipt.harness.seal.as_mut() {
            seal.digest = "sha256:also_changed".to_owned();
            if let Some(summary) = seal.verification_summary.as_mut() {
                summary.signature_valid = false;
            }
        }

        assert_eq!(canonical_receipt_body_json(&receipt)?, baseline_json);
        assert_eq!(canonical_receipt_body_digest(&receipt)?, baseline_digest);
        Ok(())
    }

    #[test]
    fn body_commitment_includes_nested_metadata_signature_keys() -> Result<(), ReceiptError> {
        let mut receipt = fixture()?;
        receipt.metadata.get_or_insert_default().insert(
            "signature".to_owned(),
            JsonValue::String("metadata-signature-1".to_owned()),
        );
        let baseline_digest = canonical_receipt_body_digest(&receipt)?;

        receipt.metadata.get_or_insert_default().insert(
            "signature".to_owned(),
            JsonValue::String("metadata-signature-2".to_owned()),
        );

        assert_ne!(canonical_receipt_body_digest(&receipt)?, baseline_digest);
        Ok(())
    }

    fn fixture() -> Result<HarnessReceipt, ReceiptError> {
        serde_json::from_str::<Fixture>(SUCCESS_RECEIPT)
            .map(|fixture| fixture.expected)
            .map_err(|source| ReceiptError::Serialization {
                message: source.to_string(),
            })
    }
}
