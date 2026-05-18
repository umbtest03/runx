use runx_core::policy::{
    GraphScopeAdmissionRequest, LocalAdmissionOptions, LocalAdmissionSkill, RetryAdmissionRequest,
    SandboxAdmissionOptions, SandboxDeclaration, admit_graph_step_scopes, admit_local_skill,
    admit_retry_policy, admit_sandbox, normalize_sandbox_declaration, sandbox_requires_approval,
};
use serde::Deserialize;

const FIXTURES: &[(&str, &str)] = &[
    (
        "graph-scope-allows-empty-request",
        include_str!("../../../fixtures/kernel/policy/graph-scope-allows-empty-request.json"),
    ),
    (
        "graph-scope-allows-exact-match",
        include_str!("../../../fixtures/kernel/policy/graph-scope-allows-exact-match.json"),
    ),
    (
        "graph-scope-allows-wildcard-narrowing",
        include_str!("../../../fixtures/kernel/policy/graph-scope-allows-wildcard-narrowing.json"),
    ),
    (
        "graph-scope-deduplicates-requests",
        include_str!("../../../fixtures/kernel/policy/graph-scope-deduplicates-requests.json"),
    ),
    (
        "graph-scope-denies-empty-grant",
        include_str!("../../../fixtures/kernel/policy/graph-scope-denies-empty-grant.json"),
    ),
    (
        "graph-scope-denies-partial-widening",
        include_str!("../../../fixtures/kernel/policy/graph-scope-denies-partial-widening.json"),
    ),
    (
        "graph-scope-denies-prefix-wildcard-request",
        include_str!(
            "../../../fixtures/kernel/policy/graph-scope-denies-prefix-wildcard-request.json"
        ),
    ),
    (
        "graph-scope-denies-prefix-substring",
        include_str!("../../../fixtures/kernel/policy/graph-scope-denies-prefix-substring.json"),
    ),
    (
        "graph-scope-denies-widening",
        include_str!("../../../fixtures/kernel/policy/graph-scope-denies-widening.json"),
    ),
    (
        "graph-scope-omits-grant-id-when-absent",
        include_str!("../../../fixtures/kernel/policy/graph-scope-omits-grant-id-when-absent.json"),
    ),
    (
        "local-admission-allows-cli-tool",
        include_str!("../../../fixtures/kernel/policy/local-admission-allows-cli-tool.json"),
    ),
    (
        "local-admission-allows-connected-wildcard-grant",
        include_str!(
            "../../../fixtures/kernel/policy/local-admission-allows-connected-wildcard-grant.json"
        ),
    ),
    (
        "local-admission-denies-connected-prefix-substring",
        include_str!(
            "../../../fixtures/kernel/policy/local-admission-denies-connected-prefix-substring.json"
        ),
    ),
    (
        "local-admission-denies-inline-python-through-env",
        include_str!(
            "../../../fixtures/kernel/policy/local-admission-denies-inline-python-through-env.json"
        ),
    ),
    (
        "local-admission-denies-inline-windows-path-interpreter",
        include_str!(
            "../../../fixtures/kernel/policy/local-admission-denies-inline-windows-path-interpreter.json"
        ),
    ),
    (
        "local-admission-denies-unsupported-source",
        include_str!(
            "../../../fixtures/kernel/policy/local-admission-denies-unsupported-source.json"
        ),
    ),
    (
        "retry-admission-allows-readonly-retry",
        include_str!("../../../fixtures/kernel/policy/retry-admission-allows-readonly-retry.json"),
    ),
    (
        "retry-admission-denies-mutating-without-key",
        include_str!(
            "../../../fixtures/kernel/policy/retry-admission-denies-mutating-without-key.json"
        ),
    ),
    (
        "sandbox-denies-readonly-network",
        include_str!("../../../fixtures/kernel/policy/sandbox-denies-readonly-network.json"),
    ),
    (
        "sandbox-normalize-defaults",
        include_str!("../../../fixtures/kernel/policy/sandbox-normalize-defaults.json"),
    ),
    (
        "sandbox-requires-approval-boolean",
        include_str!("../../../fixtures/kernel/policy/sandbox-requires-approval-boolean.json"),
    ),
    (
        "sandbox-requires-unrestricted-approval",
        include_str!("../../../fixtures/kernel/policy/sandbox-requires-unrestricted-approval.json"),
    ),
];

#[derive(Debug, Deserialize)]
struct Fixture {
    input: PolicyInput,
    expected: ExpectedOutput,
}

#[derive(Debug, Deserialize)]
struct ExpectedOutput {
    value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
enum PolicyInput {
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
}

#[test]
fn policy_fixtures_match_rust_policy() -> Result<(), serde_json::Error> {
    for (name, source) in FIXTURES {
        let fixture: Fixture = serde_json::from_str(source)?;
        let actual = evaluate_policy_input(fixture.input)?;
        assert_eq!(actual, fixture.expected.value, "{name}");
    }
    Ok(())
}

fn evaluate_policy_input(input: PolicyInput) -> Result<serde_json::Value, serde_json::Error> {
    match input {
        PolicyInput::AdmitLocalSkill { skill, options } => {
            serde_json::to_value(admit_local_skill(&skill, &options))
        }
        PolicyInput::AdmitRetryPolicy { request } => {
            serde_json::to_value(admit_retry_policy(&request))
        }
        PolicyInput::AdmitGraphStepScopes { request } => {
            serde_json::to_value(admit_graph_step_scopes(&request))
        }
        PolicyInput::NormalizeSandboxDeclaration { sandbox } => {
            serde_json::to_value(normalize_sandbox_declaration(sandbox.as_ref()))
        }
        PolicyInput::SandboxRequiresApproval { sandbox } => {
            serde_json::to_value(sandbox_requires_approval(sandbox.as_ref()))
        }
        PolicyInput::AdmitSandbox { sandbox, options } => {
            serde_json::to_value(admit_sandbox(sandbox.as_ref(), &options))
        }
    }
}
