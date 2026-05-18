use super::{AdmissionDecision, RetryAdmissionRequest};

#[must_use]
pub fn admit_retry_policy(request: &RetryAdmissionRequest) -> AdmissionDecision {
    let max_attempts = request.retry.as_ref().map_or(1, |retry| retry.max_attempts);

    if max_attempts <= 1 {
        return AdmissionDecision::Allow {
            reasons: vec!["retry policy not requested".to_owned()],
        };
    }

    if request.mutating.unwrap_or(false) && idempotency_key_is_missing(request) {
        return AdmissionDecision::Deny {
            reasons: vec![format!(
                "step '{}' declares mutating retry without an idempotency key",
                request.step_id
            )],
        };
    }

    AdmissionDecision::Allow {
        reasons: vec!["retry policy allowed".to_owned()],
    }
}

fn idempotency_key_is_missing(request: &RetryAdmissionRequest) -> bool {
    request.idempotency_key.as_deref().is_none_or(str::is_empty)
}

#[cfg(test)]
mod tests {
    use super::admit_retry_policy;
    use crate::policy::{AdmissionDecision, RetryAdmissionRequest, RetryPolicy};

    #[test]
    fn empty_idempotency_key_matches_typescript_falsiness() {
        let decision = admit_retry_policy(&RetryAdmissionRequest {
            step_id: "deploy".to_owned(),
            retry: Some(RetryPolicy { max_attempts: 2 }),
            mutating: Some(true),
            idempotency_key: Some(String::new()),
        });

        assert!(matches!(decision, AdmissionDecision::Deny { .. }));
    }
}
