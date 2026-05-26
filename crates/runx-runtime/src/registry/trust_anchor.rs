use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use ring::signature::{ED25519, UnparsedPublicKey};

use super::types::RegistrySignedManifest;

pub const REGISTRY_SIGNED_MANIFEST_SCHEMA: &str = "runx.registry.signed_manifest.v1";
pub const RUNX_REGISTRY_MANIFEST_TRUST_KEY_ENV: &str = "RUNX_REGISTRY_MANIFEST_TRUST_KEY_BASE64";
pub const RUNX_REGISTRY_MANIFEST_TRUST_KEY_ID_ENV: &str = "RUNX_REGISTRY_MANIFEST_TRUST_KEY_ID";

const RUNX_REGISTRY_MANIFEST_KEY_ID: &str = "runx-registry-ed25519-v1";
const RUNX_REGISTRY_MANIFEST_PUBLIC_KEY_BASE64: &str =
    "vacyj4d6LKwcrUK66mdH/BWHRy9haaDRQOtJEH+vOaY=";
const REGISTRY_MANIFEST_SIGNATURE_BASE64_PREFIX: &str = "base64:";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustedRegistryManifestKey {
    pub key_id: String,
    pub public_key: Vec<u8>,
}

impl TrustedRegistryManifestKey {
    pub fn from_base64(key_id: String, public_key: &str) -> Result<Self, RegistryManifestKeyError> {
        let public_key = decode_base64(public_key).map_err(|_| RegistryManifestKeyError)?;
        if public_key.len() != 32 {
            return Err(RegistryManifestKeyError);
        }
        Ok(Self { key_id, public_key })
    }

    #[must_use]
    pub fn public_key_base64(&self) -> String {
        STANDARD.encode(&self.public_key)
    }
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("registry manifest key is invalid")]
pub struct RegistryManifestKeyError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryManifestVerificationFailure {
    UnsupportedSchema,
    UnsupportedAlgorithm,
    MalformedPayload,
    UnknownKey,
    MalformedKey,
    MalformedSignature,
    SignatureMismatch,
}

pub fn verify_registry_signed_manifest(
    manifest: &RegistrySignedManifest,
    trusted_keys: &[TrustedRegistryManifestKey],
) -> Result<(), RegistryManifestVerificationFailure> {
    if manifest.schema != REGISTRY_SIGNED_MANIFEST_SCHEMA {
        return Err(RegistryManifestVerificationFailure::UnsupportedSchema);
    }
    if manifest.signature.alg != "ed25519" {
        return Err(RegistryManifestVerificationFailure::UnsupportedAlgorithm);
    }
    validate_registry_manifest_payload_terms(manifest)?;
    let key = trusted_keys
        .iter()
        .find(|key| key.key_id == manifest.signer.key_id)
        .ok_or(RegistryManifestVerificationFailure::UnknownKey)?;
    if key.public_key.len() != 32 {
        return Err(RegistryManifestVerificationFailure::MalformedKey);
    }
    let signature = decode_signature(&manifest.signature.value)?;
    if signature.len() != 64 {
        return Err(RegistryManifestVerificationFailure::MalformedSignature);
    }
    let payload = registry_manifest_payload(
        &manifest.skill_id,
        &manifest.version,
        &manifest.digest,
        manifest.profile_digest.as_deref(),
        &manifest.signer.id,
        &manifest.signer.key_id,
    );
    UnparsedPublicKey::new(&ED25519, &key.public_key)
        .verify(payload.as_bytes(), &signature)
        .map_err(|_| RegistryManifestVerificationFailure::SignatureMismatch)
}

pub fn default_trusted_registry_manifest_keys()
-> Result<Vec<TrustedRegistryManifestKey>, RegistryManifestKeyError> {
    Ok(vec![TrustedRegistryManifestKey::from_base64(
        RUNX_REGISTRY_MANIFEST_KEY_ID.to_owned(),
        RUNX_REGISTRY_MANIFEST_PUBLIC_KEY_BASE64,
    )?])
}

fn registry_manifest_payload(
    skill_id: &str,
    version: &str,
    digest: &str,
    profile_digest: Option<&str>,
    signer_id: &str,
    key_id: &str,
) -> String {
    format!(
        "{REGISTRY_SIGNED_MANIFEST_SCHEMA}\nskill_id={skill_id}\nversion={version}\ndigest={digest}\nprofile_digest={}\nsigner_id={signer_id}\nkey_id={key_id}\n",
        profile_digest.unwrap_or("")
    )
}

fn validate_registry_manifest_payload_terms(
    manifest: &RegistrySignedManifest,
) -> Result<(), RegistryManifestVerificationFailure> {
    validate_registry_manifest_payload_term(&manifest.skill_id)?;
    validate_registry_manifest_payload_term(&manifest.version)?;
    validate_registry_manifest_payload_term(&manifest.digest)?;
    if let Some(profile_digest) = &manifest.profile_digest {
        validate_registry_manifest_payload_term(profile_digest)?;
    }
    validate_registry_manifest_payload_term(&manifest.signer.id)?;
    validate_registry_manifest_payload_term(&manifest.signer.key_id)
}

fn validate_registry_manifest_payload_term(
    value: &str,
) -> Result<(), RegistryManifestVerificationFailure> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| matches!(byte, b'\n' | b'\r' | b'=' | 0))
    {
        return Err(RegistryManifestVerificationFailure::MalformedPayload);
    }
    Ok(())
}

fn decode_signature(value: &str) -> Result<Vec<u8>, RegistryManifestVerificationFailure> {
    let Some(encoded) = value.strip_prefix(REGISTRY_MANIFEST_SIGNATURE_BASE64_PREFIX) else {
        return Err(RegistryManifestVerificationFailure::MalformedSignature);
    };
    decode_base64(encoded).map_err(|_| RegistryManifestVerificationFailure::MalformedSignature)
}

fn decode_base64(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| STANDARD.decode(value))
}
