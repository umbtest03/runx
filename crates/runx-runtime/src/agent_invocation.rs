use std::collections::BTreeMap;
use std::path::Path;

use runx_contracts::{
    AgentActInvocation, AgentActSourceType, JsonObject, JsonValue, ResolutionRequest,
};

use crate::SkillInvocation;

const TRUST_BOUNDARY: &str = "native-managed: runx executes the model and tool loop directly, receipts the result, and only yields to a surface for explicit human resolution outside this path";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AgentActInvocationSourceType {
    Agent,
    AgentStep,
}

impl AgentActInvocationSourceType {
    pub(crate) fn from_contract_value(value: &str) -> Option<Self> {
        match value {
            "agent" => Some(Self::Agent),
            "agent-step" => Some(Self::AgentStep),
            _ => None,
        }
    }

    const fn contract_source_type(self) -> AgentActSourceType {
        match self {
            Self::Agent => AgentActSourceType::Agent,
            Self::AgentStep => AgentActSourceType::AgentStep,
        }
    }
}

pub(crate) fn agent_act_resolution_request(
    request: &SkillInvocation,
    source_type: AgentActInvocationSourceType,
) -> ResolutionRequest {
    let id = agent_act_invocation_id(request, source_type);
    ResolutionRequest::AgentAct {
        id: id.clone(),
        invocation: build_agent_act_invocation(request, source_type),
    }
}

pub(crate) fn agent_act_invocation_id(
    request: &SkillInvocation,
    source_type: AgentActInvocationSourceType,
) -> String {
    let skill_name = skill_name(request, source_type);
    match source_type {
        AgentActInvocationSourceType::Agent => {
            format!("agent.{}.output", normalize_request_id(&skill_name))
        }
        AgentActInvocationSourceType::AgentStep => {
            let name = request.source.task.as_deref().unwrap_or(&skill_name);
            format!("agent_step.{}.output", normalize_request_id(name))
        }
    }
}

pub(crate) fn build_agent_act_invocation(
    request: &SkillInvocation,
    source_type: AgentActInvocationSourceType,
) -> AgentActInvocation {
    AgentActInvocation {
        id: agent_act_invocation_id(request, source_type),
        source_type: source_type.contract_source_type(),
        agent: request.source.agent.clone(),
        task: request.source.task.clone(),
        envelope: JsonValue::Object(envelope(request, source_type)),
    }
}

fn envelope(request: &SkillInvocation, source_type: AgentActInvocationSourceType) -> JsonObject {
    let mut envelope = JsonObject::new();
    envelope.insert(
        "run_id".to_owned(),
        JsonValue::String("rx_pending".to_owned()),
    );
    envelope.insert(
        "skill".to_owned(),
        JsonValue::String(skill_name(request, source_type)),
    );
    envelope.insert("instructions".to_owned(), JsonValue::String(String::new()));
    envelope.insert(
        "inputs".to_owned(),
        JsonValue::Object(request.inputs.clone()),
    );
    envelope.insert("allowed_tools".to_owned(), JsonValue::Array(Vec::new()));
    envelope.insert("current_context".to_owned(), JsonValue::Array(Vec::new()));
    envelope.insert(
        "historical_context".to_owned(),
        JsonValue::Array(Vec::new()),
    );
    envelope.insert("provenance".to_owned(), JsonValue::Array(Vec::new()));
    envelope.insert(
        "execution_location".to_owned(),
        JsonValue::Object(execution_location(&request.skill_directory, &request.env)),
    );
    envelope.insert(
        "trust_boundary".to_owned(),
        JsonValue::String(TRUST_BOUNDARY.to_owned()),
    );
    if let Some(output) = &request.source.outputs {
        envelope.insert("output".to_owned(), JsonValue::Object(output.clone()));
    }
    envelope
}

fn execution_location(skill_directory: &Path, env: &BTreeMap<String, String>) -> JsonObject {
    let mut location = JsonObject::new();
    location.insert(
        "skill_directory".to_owned(),
        JsonValue::String(skill_directory.to_string_lossy().into_owned()),
    );
    let tool_roots = parse_configured_tool_roots(env);
    if !tool_roots.is_empty() {
        location.insert(
            "tool_roots".to_owned(),
            JsonValue::Array(tool_roots.into_iter().map(JsonValue::String).collect()),
        );
    }
    location
}

fn parse_configured_tool_roots(env: &BTreeMap<String, String>) -> Vec<String> {
    let Some(value) = env.get("RUNX_TOOL_ROOTS") else {
        return Vec::new();
    };
    std::env::split_paths(value)
        .filter(|path| !path.as_os_str().is_empty())
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

fn skill_name(request: &SkillInvocation, source_type: AgentActInvocationSourceType) -> String {
    if request.skill_name.is_empty() {
        return match source_type {
            AgentActInvocationSourceType::Agent => "skill".to_owned(),
            AgentActInvocationSourceType::AgentStep => "agent-step".to_owned(),
        };
    }
    request.skill_name.clone()
}

fn normalize_request_id(value: &str) -> String {
    let mut normalized = String::new();
    let mut replaced = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-') {
            normalized.push(character);
            replaced = false;
        } else if !replaced {
            normalized.push('_');
            replaced = true;
        }
    }
    normalized
}
