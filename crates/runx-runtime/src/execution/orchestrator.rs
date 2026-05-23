//! Canonical local orchestration entrypoint.
//!
//! CLI commands and TypeScript wrappers should enter local skill, graph, and
//! harness execution through this module instead of calling narrower execution
//! helpers directly.

use std::collections::BTreeMap;
use std::path::PathBuf;

use runx_contracts::{ClosureDisposition, Receipt, JsonValue};
use thiserror::Error;

use super::harness::{HarnessReplayError, HarnessReplayOutput};
#[cfg(feature = "cli-tool")]
use super::runner::GraphRun;
use super::skill_run::{SkillRunError, execute_skill_run};

#[derive(Clone, Debug, PartialEq)]
pub struct SkillRunRequest {
    pub skill_path: PathBuf,
    pub receipt_dir: Option<PathBuf>,
    pub run_id: Option<String>,
    pub answers_path: Option<PathBuf>,
    pub inputs: BTreeMap<String, JsonValue>,
    pub env: BTreeMap<String, String>,
    pub cwd: PathBuf,
    /// Optional one-shot, per-run local credential supplied at invocation.
    ///
    /// When present, the runtime derives a `CredentialDelivery` from it for this
    /// single run. The secret value is never persisted and is redacted from
    /// captured output, receipts, and metadata through the existing delivery
    /// channel. `None` keeps the current no-credential behavior.
    pub local_credential: Option<LocalCredentialDescriptor>,
}

/// Structured per-run credential provision request.
///
/// This is the local, no-network establishment surface for the OSS CLI: the
/// caller supplies the non-secret binding fields plus the raw secret value, and
/// the runtime turns it into a `CredentialDelivery` through the existing opaque
/// `MaterialResolver`. No secret state is persisted; the descriptor lives only
/// for the duration of a single run.
#[derive(Clone, PartialEq, Eq)]
pub struct LocalCredentialDescriptor {
    /// Provider the credential authenticates against (for example `github`).
    pub provider: String,
    /// Authentication mode label carried on the delivery profile/envelope.
    pub auth_mode: String,
    /// Environment variable the secret is delivered into for the skill process.
    pub env_var: String,
    /// Opaque reference identifying the in-memory material for this run.
    pub material_ref: String,
    /// Scopes recorded on the credential envelope.
    pub scopes: Vec<String>,
    /// The raw secret value supplied for this run only.
    pub secret: String,
}

// Manual Debug so the raw secret never reaches logs, panics, or any Debug of an
// enclosing type (e.g. SkillRunRequest).
impl std::fmt::Debug for LocalCredentialDescriptor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalCredentialDescriptor")
            .field("provider", &self.provider)
            .field("auth_mode", &self.auth_mode)
            .field("env_var", &self.env_var)
            .field("material_ref", &self.material_ref)
            .field("scopes", &self.scopes)
            .field("secret", &"[redacted]")
            .finish()
    }
}

/// What a skill needs from a connected grant at run time: the provider, the
/// scopes required, and the environment variable the resolved access token is
/// delivered into.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunCredentialRequest {
    pub provider: String,
    pub scopes: Vec<String>,
    pub env_var: String,
}

/// A resolved, in-memory credential for a single run. The secret is never
/// persisted; it lives only for the duration of this run, exactly like
/// [`LocalCredentialDescriptor`].
#[derive(Clone)]
pub struct ResolvedRunCredential {
    pub auth_mode: String,
    pub material_ref: String,
    pub scopes: Vec<String>,
    pub secret: String,
}

// Manual Debug so the minted token never reaches logs or panics.
impl std::fmt::Debug for ResolvedRunCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ResolvedRunCredential")
            .field("auth_mode", &self.auth_mode)
            .field("material_ref", &self.material_ref)
            .field("scopes", &self.scopes)
            .field("secret", &"[redacted]")
            .finish()
    }
}

/// Injected resolver that turns a connected-grant requirement into a freshly
/// minted secret at run time. The OSS build never installs one (no network);
/// the hosted distribution provides a minting implementation that brokers a
/// short-lived token. Returning `Ok(None)` means "no matching grant" and leaves
/// the run uncredentialed, identical to today's behavior.
pub trait RunCredentialResolver {
    fn resolve(&self, request: &RunCredentialRequest) -> Result<Option<ResolvedRunCredential>, String>;
}

/// Extract a connected-credential requirement from a runner's `auth` block.
///
/// Auto-injection is opt-in per skill: the skill must name the delivery env var
/// (`auth.env` / `auth.env_var`). `auth.type` of `env`/`none`/`local`, or a
/// missing provider/env, yields `None` so the run stays uncredentialed.
#[must_use]
pub fn connected_credential_request(auth: Option<&JsonValue>) -> Option<RunCredentialRequest> {
    let object = match auth {
        Some(JsonValue::Object(object)) => object,
        _ => return None,
    };
    if let Some(JsonValue::String(kind)) = object.get("type") {
        if matches!(kind.as_str(), "env" | "none" | "local") {
            return None;
        }
    }
    let provider = match object.get("provider") {
        Some(JsonValue::String(provider)) if !provider.is_empty() => provider.clone(),
        _ => return None,
    };
    let env_var = match object.get("env").or_else(|| object.get("env_var")) {
        Some(JsonValue::String(env_var)) if !env_var.is_empty() => env_var.clone(),
        _ => return None,
    };
    let scopes = match object.get("scopes") {
        Some(JsonValue::Array(scopes)) => scopes
            .iter()
            .filter_map(|scope| match scope {
                JsonValue::String(value) => Some(value.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    Some(RunCredentialRequest {
        provider,
        scopes,
        env_var,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphRunRequest {
    pub graph_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarnessRunRequest {
    pub fixture_path: PathBuf,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunContinuation {
    pub run_id: Option<String>,
    pub answers_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RunRequest {
    Skill(Box<SkillRunRequest>),
    Graph(GraphRunRequest),
    Harness(HarnessRunRequest),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunResult {
    pub status: RunStatus,
    pub output: JsonValue,
    pub receipt_refs: Vec<String>,
    pub child_receipt_refs: Vec<String>,
    pub pending_requests: Vec<JsonValue>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunStatus {
    NeedsAgent,
    Sealed,
    Succeeded,
    Failed,
}

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error(transparent)]
    SkillRun(#[from] SkillRunError),
    #[error(transparent)]
    Runtime(#[from] crate::RuntimeError),
    #[error(transparent)]
    Harness(#[from] HarnessReplayError),
    #[error(
        "native graph orchestration is unavailable because runx-runtime was built without the cli-tool feature"
    )]
    CliToolFeatureDisabled,
}

#[derive(Default)]
pub struct LocalOrchestrator;

impl LocalOrchestrator {
    pub fn run(&self, request: RunRequest) -> Result<RunResult, OrchestratorError> {
        match request {
            RunRequest::Skill(request) => self.run_skill(&request),
            RunRequest::Graph(request) => self.run_graph(&request),
            RunRequest::Harness(request) => self.run_harness(&request),
        }
    }

    pub fn run_skill(&self, request: &SkillRunRequest) -> Result<RunResult, OrchestratorError> {
        let output = execute_skill_run(request)?;
        Ok(skill_result(output))
    }

    /// Like [`Self::run_skill`], but with an injected credential resolver so a
    /// skill that requires a connected provider can have a token minted and
    /// delivered for this run. Passing `None` is equivalent to [`Self::run_skill`].
    pub fn run_skill_with_resolver(
        &self,
        request: &SkillRunRequest,
        resolver: Option<&dyn RunCredentialResolver>,
    ) -> Result<RunResult, OrchestratorError> {
        let output = super::skill_run::execute_skill_run_with_resolver(request, resolver)?;
        Ok(skill_result(output))
    }

    pub fn run_graph(&self, request: &GraphRunRequest) -> Result<RunResult, OrchestratorError> {
        #[cfg(feature = "cli-tool")]
        {
            graph_result(super::runner::run_graph_file(&request.graph_path)?)
        }
        #[cfg(not(feature = "cli-tool"))]
        {
            let _ = request;
            Err(OrchestratorError::CliToolFeatureDisabled)
        }
    }

    pub fn run_harness(&self, request: &HarnessRunRequest) -> Result<RunResult, OrchestratorError> {
        harness_result(super::harness::run_harness_fixture(&request.fixture_path)?)
    }
}

fn skill_result(output: JsonValue) -> RunResult {
    let status = match object_string(&output, "status") {
        Some("needs_agent") => RunStatus::NeedsAgent,
        Some("sealed") => RunStatus::Sealed,
        _ => RunStatus::Succeeded,
    };
    let receipt_refs = object_string(&output, "receipt_id")
        .map(|receipt_id| vec![receipt_id.to_owned()])
        .unwrap_or_default();
    let pending_requests = object_array(&output, "requests")
        .map(|requests| requests.to_vec())
        .unwrap_or_default();
    RunResult {
        status,
        output,
        receipt_refs,
        child_receipt_refs: Vec::new(),
        pending_requests,
        diagnostics: Vec::new(),
    }
}

#[cfg(feature = "cli-tool")]
fn graph_result(run: GraphRun) -> Result<RunResult, OrchestratorError> {
    let status = status_from_receipt(&run.receipt);
    let output = receipt_json(&run.receipt)?;
    Ok(RunResult {
        status,
        output,
        receipt_refs: vec![run.receipt.id.clone()],
        child_receipt_refs: child_receipt_refs(&run.receipt),
        pending_requests: Vec::new(),
        diagnostics: Vec::new(),
    })
}

fn harness_result(output: HarnessReplayOutput) -> Result<RunResult, OrchestratorError> {
    let status = status_from_receipt(&output.receipt);
    let value = receipt_json(&output.receipt)?;
    Ok(RunResult {
        status,
        output: value,
        receipt_refs: vec![output.receipt.id.clone()],
        child_receipt_refs: child_receipt_refs(&output.receipt),
        pending_requests: Vec::new(),
        diagnostics: Vec::new(),
    })
}

fn status_from_receipt(receipt: &Receipt) -> RunStatus {
    match receipt.seal.disposition {
        ClosureDisposition::Closed => RunStatus::Sealed,
        _ => RunStatus::Failed,
    }
}

fn receipt_json(receipt: &Receipt) -> Result<JsonValue, OrchestratorError> {
    let value = serde_json::to_value(receipt)
        .map_err(|source| crate::RuntimeError::json("serializing orchestrated receipt", source))?;
    serde_json::from_value(value)
        .map_err(|source| crate::RuntimeError::json("normalizing orchestrated receipt", source))
        .map_err(Into::into)
}

fn child_receipt_refs(receipt: &Receipt) -> Vec<String> {
    receipt
        .lineage
        .as_ref()
        .map(|lineage| {
            lineage
                .children
                .iter()
                .map(|reference| reference.uri.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn object_string<'a>(value: &'a JsonValue, key: &str) -> Option<&'a str> {
    let JsonValue::Object(object) = value else {
        return None;
    };
    let JsonValue::String(value) = object.get(key)? else {
        return None;
    };
    Some(value)
}

fn object_array<'a>(value: &'a JsonValue, key: &str) -> Option<&'a Vec<JsonValue>> {
    let JsonValue::Object(object) = value else {
        return None;
    };
    let JsonValue::Array(value) = object.get(key)? else {
        return None;
    };
    Some(value)
}

#[cfg(test)]
mod credential_request_tests {
    use super::{RunCredentialRequest, connected_credential_request};
    use runx_contracts::JsonValue;
    use std::collections::BTreeMap;

    fn auth_object(pairs: &[(&str, JsonValue)]) -> JsonValue {
        let mut object = BTreeMap::new();
        for (key, value) in pairs {
            object.insert((*key).to_owned(), value.clone());
        }
        JsonValue::Object(object)
    }

    #[test]
    fn extracts_provider_scopes_and_env() {
        let auth = auth_object(&[
            ("provider", JsonValue::String("github".into())),
            ("env", JsonValue::String("GITHUB_TOKEN".into())),
            (
                "scopes",
                JsonValue::Array(vec![JsonValue::String("repo".into())]),
            ),
        ]);
        assert_eq!(
            connected_credential_request(Some(&auth)),
            Some(RunCredentialRequest {
                provider: "github".into(),
                scopes: vec!["repo".into()],
                env_var: "GITHUB_TOKEN".into(),
            })
        );
    }

    #[test]
    fn accepts_env_var_alias() {
        let auth = auth_object(&[
            ("provider", JsonValue::String("github".into())),
            ("env_var", JsonValue::String("GH_TOKEN".into())),
        ]);
        assert_eq!(connected_credential_request(Some(&auth)).unwrap().env_var, "GH_TOKEN");
    }

    #[test]
    fn skips_when_env_missing() {
        let auth = auth_object(&[("provider", JsonValue::String("github".into()))]);
        assert!(connected_credential_request(Some(&auth)).is_none());
    }

    #[test]
    fn skips_local_env_and_none_auth_types() {
        for kind in ["env", "none", "local"] {
            let auth = auth_object(&[
                ("type", JsonValue::String(kind.into())),
                ("provider", JsonValue::String("github".into())),
                ("env", JsonValue::String("GITHUB_TOKEN".into())),
            ]);
            assert!(connected_credential_request(Some(&auth)).is_none(), "type={kind}");
        }
    }

    #[test]
    fn skips_when_no_auth() {
        assert!(connected_credential_request(None).is_none());
        assert!(connected_credential_request(Some(&JsonValue::Bool(false))).is_none());
    }
}
