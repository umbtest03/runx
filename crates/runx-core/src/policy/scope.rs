use std::collections::BTreeSet;

pub(crate) fn scope_allows(granted_scope: &str, requested_scope: &str) -> bool {
    if granted_scope == "*" || granted_scope == requested_scope {
        return true;
    }

    granted_scope
        .strip_suffix('*')
        .filter(|prefix| prefix.ends_with(':'))
        .is_some_and(|prefix| requested_scope.starts_with(prefix))
}

pub(crate) fn unique_strings(values: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut unique = Vec::new();

    for value in values {
        if seen.insert(value.clone()) {
            unique.push(value.clone());
        }
    }

    unique
}

#[cfg(test)]
mod tests {
    use super::{scope_allows, unique_strings};

    #[test]
    fn wildcard_scope_allows_any_request() {
        assert!(scope_allows("*", "repo:read"));
    }

    #[test]
    fn prefix_wildcard_allows_strict_prefix_matches() {
        assert!(scope_allows("repo:*", "repo:read"));
        assert!(!scope_allows("repo:*", "deploy:prod"));
        assert!(!scope_allows("repo:*", "repository:read"));
        assert!(!scope_allows(":*", "repo:read"));
    }

    #[test]
    fn unique_strings_preserves_first_seen_order() {
        let values = vec![
            "repo:read".to_owned(),
            "repo:write".to_owned(),
            "repo:read".to_owned(),
        ];

        assert_eq!(unique_strings(&values), vec!["repo:read", "repo:write"]);
    }
}
