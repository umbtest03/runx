use runx_contracts::JsonValue;
use serde::{Deserialize, Serialize};

use super::{AuthorityKind, LocalAdmissionGrant, scope::scope_allows};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct ConnectedAuthRequirement {
    pub provider: String,
    pub scopes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_kind: Option<AuthorityKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_locator: Option<String>,
}

pub(crate) fn connected_auth_requirement(
    auth: Option<&JsonValue>,
) -> Option<ConnectedAuthRequirement> {
    match auth {
        None | Some(JsonValue::Null) | Some(JsonValue::Bool(false)) => None,
        Some(JsonValue::Object(object)) => requirement_from_object(object),
        Some(_) => Some(ConnectedAuthRequirement {
            provider: "unknown".to_owned(),
            scopes: Vec::new(),
            scope_family: None,
            authority_kind: None,
            target_repo: None,
            target_locator: None,
        }),
    }
}

pub(crate) fn find_matching_grant<'a>(
    requirement: &ConnectedAuthRequirement,
    grants: &'a [LocalAdmissionGrant],
) -> Option<&'a LocalAdmissionGrant> {
    grants.iter().find(|grant| {
        grant.provider == requirement.provider
            && grant.status != Some(super::LocalAdmissionGrantStatus::Revoked)
            && requirement.scopes.iter().all(|scope| {
                grant
                    .scopes
                    .iter()
                    .any(|granted_scope| scope_allows(granted_scope, scope))
            })
            && grant_reference_matches(requirement, grant)
    })
}

pub(crate) fn grant_reference_matches(
    requirement: &ConnectedAuthRequirement,
    grant: &LocalAdmissionGrant,
) -> bool {
    if !has_requirement_reference(requirement) {
        return !has_grant_reference(grant);
    }

    grant.scope_family == requirement.scope_family
        && grant.authority_kind == requirement.authority_kind
        && grant.target_repo == requirement.target_repo
        && grant.target_locator == requirement.target_locator
}

pub(crate) fn has_grant_reference(grant: &LocalAdmissionGrant) -> bool {
    truthy_string(&grant.scope_family)
        || grant.authority_kind.is_some()
        || truthy_string(&grant.target_repo)
        || truthy_string(&grant.target_locator)
}

fn has_requirement_reference(requirement: &ConnectedAuthRequirement) -> bool {
    truthy_string(&requirement.scope_family)
        || requirement.authority_kind.is_some()
        || truthy_string(&requirement.target_repo)
        || truthy_string(&requirement.target_locator)
}

fn requirement_from_object(
    object: &runx_contracts::JsonObject,
) -> Option<ConnectedAuthRequirement> {
    let auth_type = string_field(object, "type");
    if matches!(auth_type, Some("env" | "none" | "local")) {
        return None;
    }

    Some(ConnectedAuthRequirement {
        provider: string_field(object, "provider")
            .or(auth_type)
            .unwrap_or("unknown")
            .to_owned(),
        scopes: string_array_field(object, "scopes"),
        scope_family: owned_string_field(object, "scope_family"),
        authority_kind: authority_kind_field(object, "authority_kind"),
        target_repo: owned_string_field(object, "target_repo"),
        target_locator: owned_string_field(object, "target_locator"),
    })
}

fn string_field<'a>(object: &'a runx_contracts::JsonObject, field: &str) -> Option<&'a str> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Some(value),
        _ => None,
    }
}

fn owned_string_field(object: &runx_contracts::JsonObject, field: &str) -> Option<String> {
    string_field(object, field).map(ToOwned::to_owned)
}

fn string_array_field(object: &runx_contracts::JsonObject, field: &str) -> Vec<String> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .filter_map(|value| match value {
                JsonValue::String(scope) => Some(scope.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn authority_kind_field(object: &runx_contracts::JsonObject, field: &str) -> Option<AuthorityKind> {
    match string_field(object, field) {
        Some("read_only") => Some(AuthorityKind::ReadOnly),
        Some("constructive") => Some(AuthorityKind::Constructive),
        Some("destructive") => Some(AuthorityKind::Destructive),
        _ => None,
    }
}

fn truthy_string(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{ConnectedAuthRequirement, find_matching_grant};
    use crate::policy::{LocalAdmissionGrant, LocalAdmissionGrantStatus};

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn first_matching_grant_wins(first_id in grant_id(), second_id in grant_id()) {
            prop_assume!(first_id != second_id);
            let requirement = ConnectedAuthRequirement {
                provider: "github".to_owned(),
                scopes: vec!["repo:read".to_owned()],
                scope_family: None,
                authority_kind: None,
                target_repo: None,
                target_locator: None,
            };
            let first = matching_grant(first_id.clone(), "repo:*");
            let second = matching_grant(second_id, "*");
            let grants = vec![first, second];

            let matched = find_matching_grant(&requirement, &grants);

            prop_assert_eq!(
                matched.map(|grant| grant.grant_id.as_str()),
                Some(first_id.as_str()),
            );
        }
    }

    fn matching_grant(grant_id: String, scope: &str) -> LocalAdmissionGrant {
        LocalAdmissionGrant {
            grant_id,
            provider: "github".to_owned(),
            scopes: vec![scope.to_owned()],
            status: Some(LocalAdmissionGrantStatus::Active),
            scope_family: None,
            authority_kind: None,
            target_repo: None,
            target_locator: None,
        }
    }

    fn grant_id() -> impl Strategy<Value = String> {
        prop::sample::select(&["grant_a", "grant_b", "grant_c", "grant_d"]).prop_map(str::to_owned)
    }
}
