use super::{
    PublicCommentOpportunityRequest, PublicCommentPolicyDecision, PublicPolicyDecision,
    PublicPullRequestCandidateRequest, PublicRecentOutcome, PublicWorkPolicy,
    RequiredPublicWorkPolicy,
};

const DEFAULT_BLOCKED_AUTHOR_PATTERNS: &[&str] = &[
    "[bot]",
    "app/",
    "renovate",
    "dependabot",
    "github-actions",
    "github-actions[bot]",
];
const DEFAULT_BLOCKED_HEAD_REF_PREFIXES: &[&str] = &[
    "renovate/",
    "dependabot/",
    "runx/issue-",
    "runx/evidence-projection-derive",
];
const DEFAULT_BLOCKED_EXACT_LABELS: &[&str] = &[
    "dependencies",
    "dependency",
    "deps",
    "rust dependencies",
    "javascript dependencies",
    "python dependencies",
    "artifact drift",
    "artifact-update",
    "artifact update",
    "internal",
];
const DEFAULT_BLOCKED_LABEL_PREFIXES: &[&str] = &["build:", "release:"];
const DEFAULT_TRUST_RECOVERY_STATUSES: &[&str] = &["spam", "minimized", "harmful"];

#[must_use]
pub fn default_public_work_policy() -> RequiredPublicWorkPolicy {
    RequiredPublicWorkPolicy {
        blocked_author_patterns: strings(DEFAULT_BLOCKED_AUTHOR_PATTERNS),
        blocked_head_ref_prefixes: strings(DEFAULT_BLOCKED_HEAD_REF_PREFIXES),
        blocked_exact_labels: strings(DEFAULT_BLOCKED_EXACT_LABELS),
        blocked_label_prefixes: strings(DEFAULT_BLOCKED_LABEL_PREFIXES),
        trust_recovery_statuses: strings(DEFAULT_TRUST_RECOVERY_STATUSES),
        require_welcome_signal_for_pull_request_comments: true,
    }
}

#[must_use]
pub fn normalize_public_work_policy(policy: &PublicWorkPolicy) -> RequiredPublicWorkPolicy {
    let fallback = default_public_work_policy();
    RequiredPublicWorkPolicy {
        blocked_author_patterns: normalize_values(
            policy.blocked_author_patterns.as_deref(),
            &fallback.blocked_author_patterns,
        ),
        blocked_head_ref_prefixes: normalize_values(
            policy.blocked_head_ref_prefixes.as_deref(),
            &fallback.blocked_head_ref_prefixes,
        ),
        blocked_exact_labels: normalize_values(
            policy.blocked_exact_labels.as_deref(),
            &fallback.blocked_exact_labels,
        ),
        blocked_label_prefixes: normalize_values(
            policy.blocked_label_prefixes.as_deref(),
            &fallback.blocked_label_prefixes,
        ),
        trust_recovery_statuses: normalize_values(
            policy.trust_recovery_statuses.as_deref(),
            &fallback.trust_recovery_statuses,
        ),
        require_welcome_signal_for_pull_request_comments: policy
            .require_welcome_signal_for_pull_request_comments
            .unwrap_or(fallback.require_welcome_signal_for_pull_request_comments),
    }
}

#[must_use]
pub fn evaluate_public_pull_request_candidate(
    request: &PublicPullRequestCandidateRequest,
    policy: &PublicWorkPolicy,
) -> PublicPolicyDecision {
    let normalized = normalize_public_work_policy(policy);
    let reasons = pull_request_candidate_reasons(request, &normalized);
    PublicPolicyDecision {
        blocked: !reasons.is_empty(),
        reasons,
    }
}

fn pull_request_candidate_reasons(
    request: &PublicPullRequestCandidateRequest,
    policy: &RequiredPublicWorkPolicy,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if is_blocked_author(request.author_login.as_deref(), policy) {
        reasons.push("bot_authored_pull_request".to_owned());
    }
    if is_dependency_update_pull_request(request, policy) {
        reasons.push("dependency_update_pull_request".to_owned());
    }
    if has_blocked_pull_request_labels(&request.labels, policy) {
        reasons.push("internal_or_build_only_pull_request".to_owned());
    }
    reasons
}

#[must_use]
pub fn evaluate_public_comment_opportunity(
    request: &PublicCommentOpportunityRequest,
    policy: &PublicWorkPolicy,
) -> PublicCommentPolicyDecision {
    let normalized = normalize_public_work_policy(policy);
    let mut reasons = pull_request_candidate_reasons(&request.pull_request, &normalized);
    let welcome_signal = has_welcome_signal(request, &normalized);

    if request.source.as_deref() == Some("github_pull_request")
        && request.lane.as_deref() == Some("issue-triage")
        && normalized.require_welcome_signal_for_pull_request_comments
        && !welcome_signal
    {
        reasons.push("comment_without_welcome_signal".to_owned());
    }
    if request.lane.as_deref() == Some("issue-triage")
        && is_comment_lane_in_trust_recovery(&request.recent_outcomes, &normalized)
    {
        reasons.push("comment_lane_in_trust_recovery".to_owned());
    }

    PublicCommentPolicyDecision {
        blocked: !reasons.is_empty(),
        reasons,
        welcome_signal,
    }
}

fn is_blocked_author(author_login: Option<&str>, policy: &RequiredPublicWorkPolicy) -> bool {
    let login = normalize(author_login.unwrap_or_default());
    !login.is_empty()
        && policy
            .blocked_author_patterns
            .iter()
            .any(|pattern| login.contains(pattern))
}

fn is_dependency_update_pull_request(
    request: &PublicPullRequestCandidateRequest,
    policy: &RequiredPublicWorkPolicy,
) -> bool {
    let normalized_labels = normalize_labels(&request.labels);
    let normalized_title = normalize(request.title.as_deref().unwrap_or_default());
    let normalized_head = normalize(request.head_ref_name.as_deref().unwrap_or_default());
    if policy
        .blocked_head_ref_prefixes
        .iter()
        .any(|prefix| normalized_head.starts_with(prefix))
    {
        return true;
    }
    if normalized_labels
        .iter()
        .any(|label| policy.blocked_exact_labels.contains(label))
    {
        return true;
    }
    if has_update_verb(&normalized_title) && has_version_number(&normalized_title) {
        return true;
    }
    normalized_title.contains("dependency")
        || normalized_title.contains("dependencies")
        || normalized_title.contains("deps")
}

fn has_blocked_pull_request_labels(labels: &[String], policy: &RequiredPublicWorkPolicy) -> bool {
    normalize_labels(labels).iter().any(|label| {
        policy.blocked_exact_labels.contains(label)
            || policy
                .blocked_label_prefixes
                .iter()
                .any(|prefix| label.starts_with(prefix))
    })
}

fn has_welcome_signal(
    request: &PublicCommentOpportunityRequest,
    policy: &RequiredPublicWorkPolicy,
) -> bool {
    if !policy.require_welcome_signal_for_pull_request_comments
        || request.source.as_deref() != Some("github_pull_request")
    {
        return true;
    }
    let association = request
        .author_association
        .as_deref()
        .unwrap_or_default()
        .to_uppercase();
    if matches!(
        association.as_str(),
        "OWNER" | "MEMBER" | "COLLABORATOR" | "CONTRIBUTOR"
    ) {
        return true;
    }
    number_or_zero(request.comments_count) + number_or_zero(request.review_comments_count) > 0.0
}

fn is_comment_lane_in_trust_recovery(
    recent_outcomes: &[PublicRecentOutcome],
    policy: &RequiredPublicWorkPolicy,
) -> bool {
    recent_outcomes.iter().any(|entry| {
        policy
            .trust_recovery_statuses
            .contains(&normalize(entry.status.as_deref().unwrap_or_default()))
    })
}

fn has_update_verb(title: &str) -> bool {
    title
        .split(|value: char| !value.is_ascii_alphanumeric() && value != '_')
        .any(|word| matches!(word, "update" | "upgrade" | "bump"))
}

fn has_version_number(title: &str) -> bool {
    title
        .char_indices()
        .any(|(index, _)| is_word_boundary_start(title, index) && parses_version(&title[index..]))
}

fn normalize_labels(labels: &[String]) -> Vec<String> {
    labels
        .iter()
        .map(|label| normalize(label))
        .filter(|label| !label.is_empty())
        .collect()
}

fn normalize_values(values: Option<&[String]>, fallback: &[String]) -> Vec<String> {
    values.map_or_else(
        || fallback.to_vec(),
        |entries| {
            entries
                .iter()
                .map(|value| normalize(value))
                .filter(|value| !value.is_empty())
                .collect()
        },
    )
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn number_or_zero(value: Option<f64>) -> f64 {
    value.unwrap_or(0.0)
}

fn is_word_boundary_start(value: &str, index: usize) -> bool {
    let Some(current) = value[index..].chars().next() else {
        return false;
    };
    if !is_regex_word_char(current) {
        return false;
    }
    value[..index]
        .chars()
        .next_back()
        .is_none_or(|previous| !is_regex_word_char(previous))
}

fn is_regex_word_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

fn parses_version(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = usize::from(bytes.first().is_some_and(|value| *value == b'v'));
    let left_start = index;
    while bytes.get(index).is_some_and(|value| value.is_ascii_digit()) {
        index += 1;
    }
    if index == left_start || bytes.get(index) != Some(&b'.') {
        return false;
    }
    index += 1;
    let right_start = index;
    while bytes.get(index).is_some_and(|value| value.is_ascii_digit()) {
        index += 1;
    }
    index > right_start
}
