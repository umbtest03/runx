use std::collections::BTreeMap;
use std::fmt;

use runx_core::policy::{CredentialBindingDecision, CredentialEnvelope};
use thiserror::Error;

const REDACTED_CREDENTIAL: &str = "[redacted-credential]";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CredentialDeliveryProfile {
    provider: String,
    auth_mode: String,
    env_bindings: Vec<CredentialEnvBinding>,
}

impl CredentialDeliveryProfile {
    pub fn env_token(
        provider: impl Into<String>,
        auth_mode: impl Into<String>,
        env_var: impl Into<String>,
    ) -> Result<Self, CredentialDeliveryError> {
        let env_var = env_var.into();
        validate_env_name(&env_var)?;
        Ok(Self {
            provider: provider.into(),
            auth_mode: auth_mode.into(),
            env_bindings: vec![CredentialEnvBinding {
                role: CredentialMaterialRole::AccessToken,
                env_var,
            }],
        })
    }

    #[must_use]
    pub fn provider(&self) -> &str {
        &self.provider
    }

    #[must_use]
    pub fn auth_mode(&self) -> &str {
        &self.auth_mode
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CredentialEnvBinding {
    role: CredentialMaterialRole,
    env_var: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CredentialMaterialRole {
    AccessToken,
}

impl CredentialMaterialRole {
    const fn label(self) -> &'static str {
        match self {
            Self::AccessToken => "access_token",
        }
    }
}

pub trait MaterialResolver {
    fn resolve_material(
        &self,
        material_ref: &str,
    ) -> Result<ResolvedCredentialMaterial, CredentialDeliveryError>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryMaterialResolver {
    materials: BTreeMap<String, ResolvedCredentialMaterial>,
}

impl InMemoryMaterialResolver {
    #[must_use]
    pub fn with_material(
        material_ref: impl Into<String>,
        material: ResolvedCredentialMaterial,
    ) -> Self {
        let mut materials = BTreeMap::new();
        materials.insert(material_ref.into(), material);
        Self { materials }
    }
}

impl MaterialResolver for InMemoryMaterialResolver {
    fn resolve_material(
        &self,
        material_ref: &str,
    ) -> Result<ResolvedCredentialMaterial, CredentialDeliveryError> {
        self.materials.get(material_ref).cloned().ok_or_else(|| {
            CredentialDeliveryError::MaterialNotFound {
                material_ref: material_ref.to_owned(),
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCredentialMaterial {
    material_ref: String,
    values: BTreeMap<CredentialMaterialRole, SecretString>,
}

impl ResolvedCredentialMaterial {
    #[must_use]
    pub fn access_token(material_ref: impl Into<String>, value: impl Into<String>) -> Self {
        let mut values = BTreeMap::new();
        values.insert(
            CredentialMaterialRole::AccessToken,
            SecretString::new(value),
        );
        Self {
            material_ref: material_ref.into(),
            values,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTED_CREDENTIAL)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SecretEnv {
    values: BTreeMap<String, SecretString>,
}

impl SecretEnv {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(SecretString::expose)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values
            .iter()
            .map(|(key, value)| (key.as_str(), value.expose()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CredentialDelivery {
    secret_env: SecretEnv,
}

impl CredentialDelivery {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            secret_env: SecretEnv {
                values: BTreeMap::new(),
            },
        }
    }

    pub fn from_allowed_binding<R: MaterialResolver>(
        decision: &CredentialBindingDecision,
        credential: &CredentialEnvelope,
        profile: &CredentialDeliveryProfile,
        resolver: &R,
    ) -> Result<Self, CredentialDeliveryError> {
        require_allowed_binding(decision)?;
        if credential.provider != profile.provider {
            return Err(CredentialDeliveryError::ProviderMismatch {
                credential_provider: credential.provider.clone(),
                profile_provider: profile.provider.clone(),
            });
        }
        let material = resolver.resolve_material(&credential.material_ref)?;
        if material.material_ref != credential.material_ref {
            return Err(CredentialDeliveryError::MaterialRefMismatch {
                expected: credential.material_ref.clone(),
                actual: material.material_ref,
            });
        }
        Ok(Self {
            secret_env: apply_profile(profile, &material)?,
        })
    }

    #[must_use]
    pub fn secret_env(&self) -> &SecretEnv {
        &self.secret_env
    }

    #[must_use]
    pub fn redact_text(&self, text: impl Into<String>) -> String {
        let mut redacted = text.into();
        for value in self.secret_env.values.values() {
            let secret = value.expose();
            if !secret.is_empty() {
                redacted = redacted.replace(secret, REDACTED_CREDENTIAL);
            }
        }
        redacted
    }
}

#[derive(Debug, Error)]
pub enum CredentialDeliveryError {
    #[error("credential binding denied: {}", reasons.join("; "))]
    BindingDenied { reasons: Vec<String> },
    #[error(
        "credential provider '{credential_provider}' does not match delivery profile provider '{profile_provider}'"
    )]
    ProviderMismatch {
        credential_provider: String,
        profile_provider: String,
    },
    #[error("credential material '{material_ref}' was not found")]
    MaterialNotFound { material_ref: String },
    #[error("credential material ref mismatch: expected '{expected}', got '{actual}'")]
    MaterialRefMismatch { expected: String, actual: String },
    #[error("credential material is missing role '{role}'")]
    MissingRole { role: String },
    #[error("invalid credential delivery env var '{name}'")]
    InvalidEnvName { name: String },
}

fn require_allowed_binding(
    decision: &CredentialBindingDecision,
) -> Result<(), CredentialDeliveryError> {
    match decision {
        CredentialBindingDecision::Allow { .. } => Ok(()),
        CredentialBindingDecision::Deny { reasons } => {
            Err(CredentialDeliveryError::BindingDenied {
                reasons: reasons.clone(),
            })
        }
    }
}

fn apply_profile(
    profile: &CredentialDeliveryProfile,
    material: &ResolvedCredentialMaterial,
) -> Result<SecretEnv, CredentialDeliveryError> {
    let mut values = BTreeMap::new();
    for binding in &profile.env_bindings {
        let Some(secret) = material.values.get(&binding.role) else {
            return Err(CredentialDeliveryError::MissingRole {
                role: binding.role.label().to_owned(),
            });
        };
        values.insert(binding.env_var.clone(), secret.clone());
    }
    Ok(SecretEnv { values })
}

fn validate_env_name(name: &str) -> Result<(), CredentialDeliveryError> {
    let mut chars = name.chars();
    let valid = chars
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_uppercase())
        && chars.all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(CredentialDeliveryError::InvalidEnvName {
            name: name.to_owned(),
        })
    }
}
