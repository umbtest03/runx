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
    connected_auth_checked_at: Option<&str>,
    wildcard_scopes_trusted: bool,
) -> Option<&'a LocalAdmissionGrant> {
    grants.iter().find(|grant| {
        grant.provider == requirement.provider
            // Fail closed: only an explicitly active grant admits. A missing
            // status (omitted JSON deserializes to `None`) must not be treated
            // as live.
            && grant.status == Some(super::LocalAdmissionGrantStatus::Active)
            && grant_lifetime_allows(grant, connected_auth_checked_at)
            && requirement.scopes.iter().all(|scope| {
                grant
                    .scopes
                    .iter()
                    .any(|granted_scope| {
                        scope_allows(granted_scope, scope, wildcard_scopes_trusted)
                    })
            })
            && grant_reference_matches(requirement, grant)
    })
}

fn grant_lifetime_allows(grant: &LocalAdmissionGrant, checked_at: Option<&str>) -> bool {
    let Some(expires_at) = grant.expires_at.as_deref() else {
        return false;
    };
    let Some(checked_at) = checked_at.and_then(parse_rfc3339_moment) else {
        return false;
    };
    let Some(expires_at) = parse_rfc3339_moment(expires_at) else {
        return false;
    };
    if checked_at >= expires_at {
        return false;
    }

    match grant.not_before.as_deref().map(parse_rfc3339_moment) {
        Some(Some(not_before)) => checked_at >= not_before,
        Some(None) => false,
        None => true,
    }
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

fn parse_rfc3339_moment(value: &str) -> Option<(i64, i64, u32)> {
    let (date, time_and_offset) = value.split_once('T')?;
    let (year, month, day) = parse_date(date)?;
    let (time, offset_seconds) = parse_time_and_offset(time_and_offset)?;
    let (hour, minute, second, nanos) = parse_time(time)?;
    let day_seconds = i64::from(hour)
        .checked_mul(3_600)?
        .checked_add(i64::from(minute).checked_mul(60)?)?
        .checked_add(i64::from(second))?
        .checked_sub(i64::from(offset_seconds))?;
    let days = days_from_civil(year, month, day)?.checked_add(day_seconds.div_euclid(86_400))?;
    Some((days, day_seconds.rem_euclid(86_400), nanos))
}

fn parse_date(value: &str) -> Option<(i32, u32, u32)> {
    let mut parts = value.split('-');
    let year = parts.next()?;
    let month = parts.next()?;
    let day = parts.next()?;
    if year.len() != 4 || month.len() != 2 || day.len() != 2 {
        return None;
    }
    let year = parse_i32(year)?;
    let month = parse_u32(month)?;
    let day = parse_u32(day)?;
    if parts.next().is_some()
        || !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
    {
        return None;
    }
    Some((year, month, day))
}

fn parse_time_and_offset(value: &str) -> Option<(&str, i32)> {
    if let Some(time) = value.strip_suffix('Z') {
        return Some((time, 0));
    }
    let offset_index = value
        .char_indices()
        .skip(1)
        .find_map(|(index, character)| matches!(character, '+' | '-').then_some(index))?;
    let time = &value[..offset_index];
    let offset = &value[offset_index..];
    let sign = if offset.starts_with('+') { 1 } else { -1 };
    let mut parts = offset[1..].split(':');
    let hours = parts.next()?;
    let minutes = parts.next()?;
    if hours.len() != 2 || minutes.len() != 2 {
        return None;
    }
    let hours = parse_i32(hours)?;
    let minutes = parse_i32(minutes)?;
    if parts.next().is_some() || !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
        return None;
    }
    Some((time, sign * ((hours * 3_600) + (minutes * 60))))
}

fn parse_time(value: &str) -> Option<(u32, u32, u32, u32)> {
    let mut parts = value.split(':');
    let hour = parts.next()?;
    let minute = parts.next()?;
    let seconds = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let (second_text, fraction) = seconds.split_once('.').unwrap_or((seconds, ""));
    if hour.len() != 2 || minute.len() != 2 || second_text.len() != 2 {
        return None;
    }
    let hour = parse_u32(hour)?;
    let minute = parse_u32(minute)?;
    let second = parse_u32(second_text)?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some((hour, minute, second, parse_nanos(fraction)?))
}

fn parse_nanos(value: &str) -> Option<u32> {
    if value.is_empty() {
        return Some(0);
    }
    if value.len() > 9 || !value.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    let mut nanos = parse_u32(value)?;
    for _ in value.len()..9 {
        nanos = nanos.checked_mul(10)?;
    }
    Some(nanos)
}

fn parse_i32(value: &str) -> Option<i32> {
    if value.is_empty() || !value.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn parse_u32(value: &str) -> Option<u32> {
    if value.is_empty() || !value.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    let year = i64::from(year) - i64::from((month <= 2) as i32);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day = i64::from(day);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era.checked_mul(146_097)?
        .checked_add(day_of_era)?
        .checked_sub(719_468)
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

            let matched = find_matching_grant(
                &requirement,
                &grants,
                Some("2026-05-22T00:00:00Z"),
                false,
            );

            prop_assert_eq!(
                matched.map(|grant| grant.grant_id.as_str()),
                Some(first_id.as_str()),
            );
        }
    }

    #[test]
    fn missing_status_denies_even_when_lifetime_is_valid() {
        let requirement = github_repo_read_requirement();
        let mut grant = matching_grant("grant_a".to_owned(), "repo:*");
        grant.status = None;
        let grants = vec![grant];

        let matched =
            find_matching_grant(&requirement, &grants, Some("2026-05-22T00:00:00Z"), false);

        assert!(matched.is_none());
    }

    #[test]
    fn active_grant_without_expiry_denies() {
        let requirement = github_repo_read_requirement();
        let mut grant = matching_grant("grant_a".to_owned(), "repo:*");
        grant.expires_at = None;
        let grants = vec![grant];

        let matched =
            find_matching_grant(&requirement, &grants, Some("2026-05-22T00:00:00Z"), false);

        assert!(matched.is_none());
    }

    #[test]
    fn active_grant_without_checked_at_denies() {
        let requirement = github_repo_read_requirement();
        let grants = vec![matching_grant("grant_a".to_owned(), "repo:*")];

        let matched = find_matching_grant(&requirement, &grants, None, false);

        assert!(matched.is_none());
    }

    #[test]
    fn expired_grant_denies() {
        let requirement = github_repo_read_requirement();
        let grants = vec![matching_grant("grant_a".to_owned(), "repo:*")];

        let matched =
            find_matching_grant(&requirement, &grants, Some("2026-05-23T00:00:00Z"), false);

        assert!(matched.is_none());
    }

    #[test]
    fn malformed_lifetime_denies() {
        let requirement = github_repo_read_requirement();
        let mut grant = matching_grant("grant_a".to_owned(), "repo:*");
        grant.expires_at = Some("2026-5-23T00:00:00Z".to_owned());
        let grants = vec![grant];

        let matched =
            find_matching_grant(&requirement, &grants, Some("2026-05-22T00:00:00Z"), false);

        assert!(matched.is_none());
    }

    #[test]
    fn not_before_future_grant_denies() {
        let requirement = github_repo_read_requirement();
        let mut grant = matching_grant("grant_a".to_owned(), "repo:*");
        grant.not_before = Some("2026-05-23T00:00:00Z".to_owned());
        let grants = vec![grant];

        let matched =
            find_matching_grant(&requirement, &grants, Some("2026-05-22T00:00:00Z"), false);

        assert!(matched.is_none());
    }

    fn github_repo_read_requirement() -> ConnectedAuthRequirement {
        ConnectedAuthRequirement {
            provider: "github".to_owned(),
            scopes: vec!["repo:read".to_owned()],
            scope_family: None,
            authority_kind: None,
            target_repo: None,
            target_locator: None,
        }
    }

    fn matching_grant(grant_id: String, scope: &str) -> LocalAdmissionGrant {
        LocalAdmissionGrant {
            grant_id,
            provider: "github".to_owned(),
            scopes: vec![scope.to_owned()],
            status: Some(LocalAdmissionGrantStatus::Active),
            not_before: Some("2026-05-21T00:00:00Z".to_owned()),
            expires_at: Some("2026-05-23T00:00:00Z".to_owned()),
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
