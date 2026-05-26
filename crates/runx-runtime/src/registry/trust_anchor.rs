use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use ring::signature::{ED25519, Ed25519KeyPair, KeyPair, UnparsedPublicKey};

use super::types::{RegistryManifestSignature, RegistryManifestSigner, RegistrySignedManifest};

pub const REGISTRY_SIGNED_MANIFEST_SCHEMA: &str = "runx.registry.signed_manifest.v1";
pub const RUNX_REGISTRY_MANIFEST_SIGNING_SEED_ENV: &str =
    "RUNX_REGISTRY_MANIFEST_SIGNING_SEED_BASE64";
pub const RUNX_REGISTRY_MANIFEST_SIGNING_KEY_ID_ENV: &str = "RUNX_REGISTRY_MANIFEST_SIGNING_KEY_ID";
pub const RUNX_REGISTRY_MANIFEST_SIGNER_ID_ENV: &str = "RUNX_REGISTRY_MANIFEST_SIGNER_ID";
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryManifestSigningKey {
    signer_id: String,
    key_id: String,
    seed: [u8; 32],
}

impl RegistryManifestSigningKey {
    pub fn from_seed_base64(
        signer_id: String,
        key_id: String,
        seed: &str,
    ) -> Result<Self, RegistryManifestSigningFailure> {
        let seed = decode_base64(seed).map_err(|_| RegistryManifestSigningFailure)?;
        Self::from_seed_bytes(signer_id, key_id, &seed)
    }

    pub fn from_seed_bytes(
        signer_id: String,
        key_id: String,
        seed: &[u8],
    ) -> Result<Self, RegistryManifestSigningFailure> {
        let seed: [u8; 32] = seed
            .try_into()
            .map_err(|_error| RegistryManifestSigningFailure)?;
        Ok(Self {
            signer_id,
            key_id,
            seed,
        })
    }

    pub fn trusted_key(
        &self,
    ) -> Result<TrustedRegistryManifestKey, RegistryManifestSigningFailure> {
        let key_pair = self.key_pair()?;
        Ok(TrustedRegistryManifestKey {
            key_id: self.key_id.clone(),
            public_key: key_pair.public_key().as_ref().to_vec(),
        })
    }

    fn key_pair(&self) -> Result<Ed25519KeyPair, RegistryManifestSigningFailure> {
        Ed25519KeyPair::from_seed_unchecked(&self.seed)
            .map_err(|_error| RegistryManifestSigningFailure)
    }
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("registry manifest key is invalid")]
pub struct RegistryManifestKeyError;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("registry manifest signing key is invalid")]
pub struct RegistryManifestSigningFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryManifestVerificationFailure {
    UnsupportedSchema,
    UnsupportedAlgorithm,
    UnknownKey,
    MalformedKey,
    MalformedSignature,
    SignatureMismatch,
}

pub fn sign_registry_manifest(
    key: &RegistryManifestSigningKey,
    skill_id: &str,
    version: &str,
    digest: &str,
    profile_digest: Option<&str>,
) -> Result<RegistrySignedManifest, RegistryManifestSigningFailure> {
    let key_pair = key.key_pair()?;
    let signer = RegistryManifestSigner {
        id: key.signer_id.clone(),
        key_id: key.key_id.clone(),
    };
    let payload = registry_manifest_payload(
        skill_id,
        version,
        digest,
        profile_digest,
        &signer.id,
        &signer.key_id,
    );
    let signature = key_pair.sign(payload.as_bytes());
    Ok(RegistrySignedManifest {
        schema: REGISTRY_SIGNED_MANIFEST_SCHEMA.to_owned(),
        skill_id: skill_id.to_owned(),
        version: version.to_owned(),
        digest: digest.to_owned(),
        profile_digest: profile_digest.map(ToOwned::to_owned),
        signer,
        signature: RegistryManifestSignature {
            alg: "ed25519".to_owned(),
            value: format!(
                "{REGISTRY_MANIFEST_SIGNATURE_BASE64_PREFIX}{}",
                URL_SAFE_NO_PAD.encode(signature.as_ref())
            ),
        },
    })
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
