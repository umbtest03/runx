// rust-style-allow: large-file because the kernel JSON bridge keeps its
// externally callable operation registry in one dispatch module.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use runx_contracts::{AuthorityTerm, JsonValue, json_string_field};
use serde::Deserialize;

use crate::policy::{
    BuildAuthorityProofOptions, CredentialBindingRequest, GraphScopeAdmissionRequest,
    LocalAdmissionGrant, LocalAdmissionOptions, LocalAdmissionSkill, LocalScopeAdmissionOptions,
    PublicCommentOpportunityRequest, PublicPullRequestCandidateRequest, PublicWorkPolicy,
    RetryAdmissionRequest, SandboxAdmissionOptions, SandboxDeclaration, admit_graph_step_scopes,
    admit_local_skill, admit_retry_policy, admit_sandbox, build_authority_proof_metadata,
    build_local_scope_admission, evaluate_public_comment_opportunity,
    evaluate_public_pull_request_candidate, is_payment_authority_subset,
    normalize_public_work_policy, normalize_sandbox_declaration, sandbox_requires_approval,
    validate_credential_binding,
};
use crate::state_machine::{
    FanoutBranchResult, FanoutGroupPolicy, SequentialGraphEvent, SequentialGraphState,
    SequentialGraphStepDefinition, SingleStepEvent, SingleStepState, create_sequential_graph_state,
    create_single_step_state, evaluate_fanout_sync, fanout_sync_decision_key,
    plan_sequential_graph_transition, transition_sequential_graph, transition_single_step,
};

const MAX_KERNEL_EVAL_DOCUMENT_BYTES: usize = 1024 * 1024;
const MAX_KERNEL_EVAL_JSON_DEPTH: usize = 64;
const MAX_KERNEL_EVAL_JSON_NODES: usize = 20_000;
const MAX_KERNEL_EVAL_ARRAY_ITEMS: usize = 4_096;
const MAX_KERNEL_EVAL_OBJECT_FIELDS: usize = 512;
const MAX_KERNEL_EVAL_OBJECT_KEY_BYTES: usize = 1024;
const MAX_KERNEL_EVAL_STRING_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KernelEvalOutput {
    Output { value: JsonValue },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KernelEvalError {
    InvalidDocument(String),
    InvalidInput(String),
    SerializeOutput(String),
}

impl KernelEvalError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidDocument(_) => "invalid_document",
            Self::InvalidInput(_) => "invalid_input",
            Self::SerializeOutput(_) => "serialize_output",
        }
    }
}

impl fmt::Display for KernelEvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocument(message)
            | Self::InvalidInput(message)
            | Self::SerializeOutput(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for KernelEvalError {}

pub fn evaluate_kernel_document_str(source: &str) -> Result<KernelEvalOutput, KernelEvalError> {
    validate_kernel_source_size(source)?;
    let document = serde_json::from_str::<JsonValue>(source)
        .map_err(|error| KernelEvalError::InvalidDocument(error.to_string()))?;
    validate_kernel_document_shape(&document)?;
    if let Some(kind) = kernel_document_kind(&document)
        && !is_supported_kernel_kind(kind)
    {
        return Err(KernelEvalError::InvalidInput(format!(
            "unsupported kernel input kind '{kind}'"
        )));
    }
    let input = serde_json::from_str::<KernelDocument>(source)
        .map_err(|error| KernelEvalError::InvalidInput(error.to_string()))?;
    Ok(KernelEvalOutput::Output {
        value: evaluate_kernel_input(input)?,
    })
}

fn validate_kernel_source_size(source: &str) -> Result<(), KernelEvalError> {
    if source.len() > MAX_KERNEL_EVAL_DOCUMENT_BYTES {
        return Err(KernelEvalError::InvalidInput(format!(
            "kernel eval input exceeds {MAX_KERNEL_EVAL_DOCUMENT_BYTES} bytes"
        )));
    }
    Ok(())
}

fn validate_kernel_document_shape(document: &JsonValue) -> Result<(), KernelEvalError> {
    let mut node_count = 0usize;
    let mut pending = vec![(document, 1usize)];

    while let Some((value, depth)) = pending.pop() {
        node_count += 1;
        if node_count > MAX_KERNEL_EVAL_JSON_NODES {
            return Err(KernelEvalError::InvalidInput(format!(
                "kernel eval input exceeds {MAX_KERNEL_EVAL_JSON_NODES} JSON nodes"
            )));
        }
        if depth > MAX_KERNEL_EVAL_JSON_DEPTH {
            return Err(KernelEvalError::InvalidInput(format!(
                "kernel eval input exceeds JSON depth {MAX_KERNEL_EVAL_JSON_DEPTH}"
            )));
        }

        match value {
            JsonValue::Array(values) => {
                if values.len() > MAX_KERNEL_EVAL_ARRAY_ITEMS {
                    return Err(KernelEvalError::InvalidInput(format!(
                        "kernel eval input array exceeds {MAX_KERNEL_EVAL_ARRAY_ITEMS} items"
                    )));
                }
                for child in values {
                    pending.push((child, depth + 1));
                }
            }
            JsonValue::Object(fields) => {
                if fields.len() > MAX_KERNEL_EVAL_OBJECT_FIELDS {
                    return Err(KernelEvalError::InvalidInput(format!(
                        "kernel eval input object exceeds {MAX_KERNEL_EVAL_OBJECT_FIELDS} fields"
                    )));
                }
                for (key, child) in fields {
                    if key.len() > MAX_KERNEL_EVAL_OBJECT_KEY_BYTES {
                        return Err(KernelEvalError::InvalidInput(format!(
                            "kernel eval input object key exceeds {MAX_KERNEL_EVAL_OBJECT_KEY_BYTES} bytes"
                        )));
                    }
                    pending.push((child, depth + 1));
                }
            }
            JsonValue::String(value) => {
                if value.len() > MAX_KERNEL_EVAL_STRING_BYTES {
                    return Err(KernelEvalError::InvalidInput(format!(
                        "kernel eval input string exceeds {MAX_KERNEL_EVAL_STRING_BYTES} bytes"
                    )));
                }
            }
            JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) => {}
        }
    }

    Ok(())
}

fn kernel_document_kind(document: &JsonValue) -> Option<&str> {
    let JsonValue::Object(fields) = document else {
        return None;
    };
    match fields.get("input") {
        Some(JsonValue::Object(input)) => json_string_field(input, "kind"),
        _ => json_string_field(fields, "kind"),
    }
}

fn is_supported_kernel_kind(kind: &str) -> bool {
    matches!(
        kind,
        "policy.admitLocalSkill"
            | "policy.admitRetryPolicy"
            | "policy.admitGraphStepScopes"
            | "policy.normalizeSandboxDeclaration"
            | "policy.sandboxRequiresApproval"
            | "policy.admitSandbox"
            | "policy.buildLocalScopeAdmission"
            | "policy.buildAuthorityProofMetadata"
            | "policy.validateCredentialBinding"
            | "policy.evaluatePublicPullRequestCandidate"
            | "policy.evaluatePublicCommentOpportunity"
            | "policy.normalizePublicWorkPolicy"
            | "policy.isPaymentAuthoritySubset"
            | "state-machine.createSingleStepState"
            | "state-machine.transitionSingleStep"
            | "state-machine.createSequentialGraphState"
            | "state-machine.planSequentialGraphTransition"
            | "state-machine.transitionSequentialGraph"
            | "state-machine.evaluateFanoutSync"
            | "state-machine.fanoutSyncDecisionKey"
    )
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum KernelDocument {
    Envelope { input: KernelInput },
    Input(KernelInput),
}

impl From<KernelDocument> for KernelInput {
    fn from(document: KernelDocument) -> Self {
        match document {
            KernelDocument::Envelope { input } | KernelDocument::Input(input) => input,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all_fields = "camelCase")]
enum KernelInput {
    #[serde(rename = "policy.admitLocalSkill")]
    AdmitLocalSkill {
        skill: Box<LocalAdmissionSkill>,
        #[serde(default)]
        options: LocalAdmissionOptions,
    },
    #[serde(rename = "policy.admitRetryPolicy")]
    AdmitRetryPolicy { request: RetryAdmissionRequest },
    #[serde(rename = "policy.admitGraphStepScopes")]
    AdmitGraphStepScopes { request: GraphScopeAdmissionRequest },
    #[serde(rename = "policy.normalizeSandboxDeclaration")]
    NormalizeSandboxDeclaration { sandbox: Option<SandboxDeclaration> },
    #[serde(rename = "policy.sandboxRequiresApproval")]
    SandboxRequiresApproval { sandbox: Option<SandboxDeclaration> },
    #[serde(rename = "policy.admitSandbox")]
    AdmitSandbox {
        sandbox: Option<SandboxDeclaration>,
        #[serde(default)]
        options: SandboxAdmissionOptions,
    },
    #[serde(rename = "policy.buildLocalScopeAdmission")]
    BuildLocalScopeAdmission {
        auth: Option<JsonValue>,
        #[serde(default)]
        grants: Vec<LocalAdmissionGrant>,
        #[serde(default)]
        options: LocalScopeAdmissionOptions,
    },
    #[serde(rename = "policy.buildAuthorityProofMetadata")]
    BuildAuthorityProofMetadata {
        options: Box<BuildAuthorityProofOptions>,
    },
    #[serde(rename = "policy.validateCredentialBinding")]
    ValidateCredentialBinding {
        request: Box<CredentialBindingRequest>,
    },
    #[serde(rename = "policy.evaluatePublicPullRequestCandidate")]
    EvaluatePublicPullRequestCandidate {
        request: PublicPullRequestCandidateRequest,
        #[serde(default)]
        policy: PublicWorkPolicy,
    },
    #[serde(rename = "policy.evaluatePublicCommentOpportunity")]
    EvaluatePublicCommentOpportunity {
        request: PublicCommentOpportunityRequest,
        #[serde(default)]
        policy: PublicWorkPolicy,
    },
    #[serde(rename = "policy.normalizePublicWorkPolicy")]
    NormalizePublicWorkPolicy {
        #[serde(default)]
        policy: PublicWorkPolicy,
    },
    #[serde(rename = "policy.isPaymentAuthoritySubset")]
    IsPaymentAuthoritySubset {
        child: Box<AuthorityTerm>,
        parent: Box<AuthorityTerm>,
    },
    #[serde(rename = "state-machine.createSingleStepState")]
    CreateSingleStepState { step_id: String },
    #[serde(rename = "state-machine.transitionSingleStep")]
    TransitionSingleStep {
        state: SingleStepState,
        event: SingleStepEvent,
    },
    #[serde(rename = "state-machine.createSequentialGraphState")]
    CreateSequentialGraphState {
        graph_id: String,
        steps: Vec<SequentialGraphStepDefinition>,
    },
    #[serde(rename = "state-machine.planSequentialGraphTransition")]
    PlanSequentialGraphTransition {
        state: SequentialGraphState,
        steps: Vec<SequentialGraphStepDefinition>,
        #[serde(default)]
        fanout_policies: BTreeMap<String, FanoutGroupPolicy>,
        resolved_fanout_gate_keys: Option<Vec<String>>,
    },
    #[serde(rename = "state-machine.transitionSequentialGraph")]
    TransitionSequentialGraph {
        state: SequentialGraphState,
        event: SequentialGraphEvent,
    },
    #[serde(rename = "state-machine.evaluateFanoutSync")]
    EvaluateFanoutSync {
        policy: FanoutGroupPolicy,
        results: Vec<FanoutBranchResult>,
        resolved_gate_keys: Option<Vec<String>>,
    },
    #[serde(rename = "state-machine.fanoutSyncDecisionKey")]
    FanoutSyncDecisionKey { decision: DecisionKeyInput },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecisionKeyInput {
    group_id: String,
    rule_fired: String,
}

fn evaluate_kernel_input(input: KernelDocument) -> Result<JsonValue, KernelEvalError> {
    let input = KernelInput::from(input);
    match input {
        KernelInput::AdmitLocalSkill { .. }
        | KernelInput::AdmitRetryPolicy { .. }
        | KernelInput::AdmitGraphStepScopes { .. }
        | KernelInput::NormalizeSandboxDeclaration { .. }
        | KernelInput::SandboxRequiresApproval { .. }
        | KernelInput::AdmitSandbox { .. }
        | KernelInput::BuildLocalScopeAdmission { .. }
        | KernelInput::BuildAuthorityProofMetadata { .. }
        | KernelInput::ValidateCredentialBinding { .. }
        | KernelInput::EvaluatePublicPullRequestCandidate { .. }
        | KernelInput::EvaluatePublicCommentOpportunity { .. }
        | KernelInput::NormalizePublicWorkPolicy { .. }
        | KernelInput::IsPaymentAuthoritySubset { .. } => evaluate_policy_input(input),
        KernelInput::CreateSingleStepState { .. }
        | KernelInput::TransitionSingleStep { .. }
        | KernelInput::CreateSequentialGraphState { .. }
        | KernelInput::PlanSequentialGraphTransition { .. }
        | KernelInput::TransitionSequentialGraph { .. }
        | KernelInput::EvaluateFanoutSync { .. }
        | KernelInput::FanoutSyncDecisionKey { .. } => evaluate_state_machine_input(input),
    }
}

fn evaluate_policy_input(input: KernelInput) -> Result<JsonValue, KernelEvalError> {
    match input {
        KernelInput::AdmitLocalSkill { skill, options } => {
            to_value(admit_local_skill(&skill, &options))
        }
        KernelInput::AdmitRetryPolicy { request } => to_value(admit_retry_policy(&request)),
        KernelInput::AdmitGraphStepScopes { request } => {
            to_value(admit_graph_step_scopes(&request))
        }
        KernelInput::NormalizeSandboxDeclaration { sandbox } => {
            to_value(normalize_sandbox_declaration(sandbox.as_ref()))
        }
        KernelInput::SandboxRequiresApproval { sandbox } => {
            to_value(sandbox_requires_approval(sandbox.as_ref()))
        }
        KernelInput::AdmitSandbox { sandbox, options } => {
            to_value(admit_sandbox(sandbox.as_ref(), &options))
        }
        KernelInput::BuildLocalScopeAdmission {
            auth,
            grants,
            options,
        } => to_value(build_local_scope_admission(
            auth.as_ref(),
            &grants,
            &options,
        )),
        KernelInput::BuildAuthorityProofMetadata { options } => {
            to_value(build_authority_proof_metadata(&options))
        }
        KernelInput::ValidateCredentialBinding { request } => {
            to_value(validate_credential_binding(&request))
        }
        KernelInput::EvaluatePublicPullRequestCandidate { request, policy } => {
            to_value(evaluate_public_pull_request_candidate(&request, &policy))
        }
        KernelInput::EvaluatePublicCommentOpportunity { request, policy } => {
            to_value(evaluate_public_comment_opportunity(&request, &policy))
        }
        KernelInput::NormalizePublicWorkPolicy { policy } => {
            to_value(normalize_public_work_policy(&policy))
        }
        KernelInput::IsPaymentAuthoritySubset { child, parent } => {
            to_value(is_payment_authority_subset(&child, &parent))
        }
        _ => unreachable!("policy dispatch only receives policy inputs"),
    }
}

fn evaluate_state_machine_input(input: KernelInput) -> Result<JsonValue, KernelEvalError> {
    match input {
        KernelInput::CreateSingleStepState { step_id } => {
            to_value(create_single_step_state(step_id))
        }
        KernelInput::TransitionSingleStep { state, event } => {
            to_value(transition_single_step(&state, &event))
        }
        KernelInput::CreateSequentialGraphState { graph_id, steps } => {
            to_value(create_sequential_graph_state(graph_id, &steps))
        }
        KernelInput::PlanSequentialGraphTransition {
            state,
            steps,
            fanout_policies,
            resolved_fanout_gate_keys,
        } => {
            let resolved = resolved_fanout_gate_keys.map(vec_to_set);
            to_value(plan_sequential_graph_transition(
                &state,
                &steps,
                &fanout_policies,
                resolved.as_ref(),
            ))
        }
        KernelInput::TransitionSequentialGraph { state, event } => {
            to_value(transition_sequential_graph(&state, &event))
        }
        KernelInput::EvaluateFanoutSync {
            policy,
            results,
            resolved_gate_keys,
        } => {
            let resolved = resolved_gate_keys.map(vec_to_set);
            to_value(evaluate_fanout_sync(&policy, &results, resolved.as_ref()))
        }
        KernelInput::FanoutSyncDecisionKey { decision } => Ok(JsonValue::String(
            fanout_sync_decision_key(&decision.group_id, &decision.rule_fired),
        )),
        _ => unreachable!("state-machine dispatch only receives state-machine inputs"),
    }
}

fn to_value(value: impl serde::Serialize) -> Result<JsonValue, KernelEvalError> {
    let source = serde_json::to_string(&value)
        .map_err(|error| KernelEvalError::SerializeOutput(error.to_string()))?;
    serde_json::from_str(&source)
        .map_err(|error| KernelEvalError::SerializeOutput(error.to_string()))
}

fn vec_to_set(values: Vec<String>) -> BTreeSet<String> {
    values.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_supported_document_under_limits() -> Result<(), KernelEvalError> {
        let output = evaluate_kernel_document_str(
            r#"{"kind":"state-machine.fanoutSyncDecisionKey","decision":{"groupId":"group-a","ruleFired":"all_succeeded"}}"#,
        )?;

        assert_eq!(
            output,
            KernelEvalOutput::Output {
                value: JsonValue::String("group-a:all_succeeded".to_owned())
            }
        );
        Ok(())
    }

    #[test]
    fn rejects_oversized_kernel_eval_source_before_parse() {
        let source = " ".repeat(MAX_KERNEL_EVAL_DOCUMENT_BYTES + 1);
        let error = assert_invalid_input(&source);

        assert_eq!(error.code(), "invalid_input");
        assert!(error.to_string().contains("kernel eval input exceeds"));
    }

    #[test]
    fn rejects_deep_kernel_eval_json_before_dispatch() {
        let source = format!(
            "{}0{}",
            "[".repeat(MAX_KERNEL_EVAL_JSON_DEPTH),
            "]".repeat(MAX_KERNEL_EVAL_JSON_DEPTH),
        );
        let error = assert_invalid_input(&source);

        assert_eq!(error.code(), "invalid_input");
        assert!(error.to_string().contains("exceeds JSON depth"));
    }

    fn assert_invalid_input(source: &str) -> KernelEvalError {
        match evaluate_kernel_document_str(source) {
            Err(error @ KernelEvalError::InvalidInput(_)) => error,
            Err(error) => error,
            Ok(output) => {
                KernelEvalError::InvalidInput(format!("expected invalid_input, got {output:?}"))
            }
        }
    }
}
