use runx_contracts::tools::{JsonPayload, JsonPayloadObject};
use sha2::{Digest, Sha256};

pub(crate) fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

pub(crate) fn sha256_stable(value: &JsonPayload) -> String {
    sha256_prefixed(stable_stringify(value).as_bytes())
}

pub(crate) fn stable_stringify(value: &JsonPayload) -> String {
    match value {
        JsonPayload::Null => "null".to_owned(),
        JsonPayload::Bool(value) => value.to_string(),
        JsonPayload::Number(value) => value.to_string(),
        JsonPayload::String(value) => json_string(value),
        JsonPayload::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(stable_stringify)
                .collect::<Vec<_>>()
                .join(",")
        ),
        JsonPayload::Object(values) => stable_object(values),
    }
}

fn stable_object(values: &JsonPayloadObject) -> String {
    let entries = values
        .iter()
        .map(|(key, value)| format!("{}:{}", json_string(key), stable_stringify(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{entries}}}")
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_owned())
}
