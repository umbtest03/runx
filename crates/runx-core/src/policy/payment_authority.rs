use std::collections::BTreeSet;

use runx_contracts::{AuthorityResourceFamily, AuthorityTerm, AuthorityVerb};
use serde_json::Value;

const PAYMENT_CAP_KEYS: &[(&str, &[&str])] = &[
    (
        "quote",
        &[
            "quote_minor_units",
            "quoteMinorUnits",
            "max_quote_minor_units",
            "maxQuoteMinorUnits",
            "quote_minor_unit_cap",
            "quoteMinorUnitCap",
            "quote_minor_units_cap",
            "quoteMinorUnitsCap",
        ],
    ),
    (
        "reserve",
        &[
            "reserve_minor_units",
            "reserveMinorUnits",
            "max_reserve_minor_units",
            "maxReserveMinorUnits",
            "reserve_minor_unit_cap",
            "reserveMinorUnitCap",
            "reserve_minor_units_cap",
            "reserveMinorUnitsCap",
        ],
    ),
    (
        "spend",
        &[
            "spend_minor_units",
            "spendMinorUnits",
            "max_spend_minor_units",
            "maxSpendMinorUnits",
            "spend_minor_unit_cap",
            "spendMinorUnitCap",
            "spend_minor_units_cap",
            "spendMinorUnitsCap",
        ],
    ),
    (
        "refund",
        &[
            "refund_minor_units",
            "refundMinorUnits",
            "max_refund_minor_units",
            "maxRefundMinorUnits",
            "refund_minor_unit_cap",
            "refundMinorUnitCap",
            "refund_minor_units_cap",
            "refundMinorUnitsCap",
        ],
    ),
];

const SHARED_CAP_KEYS: &[&str] = &[
    "minor_units",
    "minorUnits",
    "max_minor_units",
    "maxMinorUnits",
    "minor_unit_cap",
    "minorUnitCap",
    "minor_units_cap",
    "minorUnitsCap",
];

const BOOLEAN_NARROWING_KEYS: &[&[&str]] = &[
    &["quote_required", "quoteRequired"],
    &["reservation_required", "reservationRequired"],
    &["idempotency_required", "idempotencyRequired"],
    &[
        "recovery_by_idempotency_required",
        "recoveryByIdempotencyRequired",
    ],
    &["receipt_before_success", "receiptBeforeSuccess"],
];

/// Returns true when `child` is no broader than `parent` under the pure
/// payment authority policy.
///
/// Incomparable terms are denied by returning false.
pub fn is_payment_authority_subset(child: &AuthorityTerm, parent: &AuthorityTerm) -> bool {
    if child.resource_family != AuthorityResourceFamily::Payment {
        return false;
    }
    if parent.resource_family != AuthorityResourceFamily::Payment {
        return false;
    }
    if !verbs_subset(&child.verbs, &parent.verbs) {
        return false;
    }

    let Some(child_payment) = payment_bounds_value(child) else {
        return false;
    };
    let Some(parent_payment) = payment_bounds_value(parent) else {
        return false;
    };

    string_equal(&child_payment, &parent_payment, &["currency"])
        && minor_unit_caps_subset(child, parent, &child_payment, &parent_payment)
        && string_set_subset(&child_payment, &parent_payment, &["rails"])
        && exact_or_narrower(&child_payment, &parent_payment, &["realm"])
        && exact_or_narrower(&child_payment, &parent_payment, &["counterparty"])
        && exact_or_narrower(&child_payment, &parent_payment, &["operation"])
        && expiry_subset(child, parent)
        && required_booleans_subset(&child_payment, &parent_payment)
        && optional_u64_lte_when_parent_set(
            &child_payment,
            &parent_payment,
            &["quote_ttl_seconds", "quoteTtlSeconds"],
        )
        && optional_u64_lte_when_parent_set(
            &child_payment,
            &parent_payment,
            &["approval_over_minor", "approvalOverMinor"],
        )
        && exact_or_narrower(
            &child_payment,
            &parent_payment,
            &["credential_form", "credentialForm"],
        )
        && single_use_spend_capability_for_reserve_or_spend(child, &child_payment)
}

fn payment_bounds_value(term: &AuthorityTerm) -> Option<Value> {
    serde_json::to_value(&term.bounds)
        .ok()?
        .as_object()?
        .get("payment")
        .cloned()
        .filter(|value| !value.is_null())
}

fn verbs_subset(child: &[AuthorityVerb], parent: &[AuthorityVerb]) -> bool {
    child.iter().all(|verb| parent.contains(verb))
}

fn string_equal(child: &Value, parent: &Value, keys: &[&str]) -> bool {
    string_value(child, keys)
        .zip(string_value(parent, keys))
        .is_some_and(|(child, parent)| child == parent)
}

fn exact_or_narrower(child: &Value, parent: &Value, keys: &[&str]) -> bool {
    match (string_value(child, keys), string_value(parent, keys)) {
        (_, None) => true,
        (Some(child), Some(parent)) => child == parent,
        (None, Some(_)) => false,
    }
}

fn string_set_subset(child: &Value, parent: &Value, keys: &[&str]) -> bool {
    let Some(child_values) = string_set(child, keys) else {
        return false;
    };
    let Some(parent_values) = string_set(parent, keys) else {
        return false;
    };

    child_values.is_subset(&parent_values)
}

fn string_set(value: &Value, keys: &[&str]) -> Option<BTreeSet<String>> {
    let values = value_for_any_key(value, keys)?;
    let array = values.as_array()?;
    Some(
        array
            .iter()
            .map(|value| value.as_str().map(str::to_owned))
            .collect::<Option<BTreeSet<_>>>()?,
    )
}

fn minor_unit_caps_subset(
    child: &AuthorityTerm,
    parent: &AuthorityTerm,
    child_payment: &Value,
    parent_payment: &Value,
) -> bool {
    payment_cap_dimensions(child)
        .into_iter()
        .all(|dimension| match minor_unit_cap(child_payment, dimension) {
            Some(child_cap) => minor_unit_cap(parent_payment, dimension)
                .is_some_and(|parent_cap| child_cap <= parent_cap),
            None => false,
        })
        && payment_cap_dimensions(parent)
            .into_iter()
            .all(|dimension| minor_unit_cap(child_payment, dimension).is_some())
}

fn payment_cap_dimensions(term: &AuthorityTerm) -> BTreeSet<&'static str> {
    term.verbs
        .iter()
        .filter_map(|verb| match verb {
            AuthorityVerb::Quote => Some("quote"),
            AuthorityVerb::Reserve => Some("reserve"),
            AuthorityVerb::Spend => Some("spend"),
            AuthorityVerb::Refund => Some("refund"),
            AuthorityVerb::Verify => None,
            _ => None,
        })
        .collect()
}

fn minor_unit_cap(value: &Value, dimension: &str) -> Option<u64> {
    PAYMENT_CAP_KEYS
        .iter()
        .find(|(candidate, _)| *candidate == dimension)
        .and_then(|(_, keys)| u64_value(value, keys))
        .or_else(|| u64_value(value, SHARED_CAP_KEYS))
}

fn expiry_subset(child: &AuthorityTerm, parent: &AuthorityTerm) -> bool {
    match (&child.expires_at, &parent.expires_at) {
        (_, None) => true,
        (Some(child), Some(parent)) => child <= parent,
        (None, Some(_)) => false,
    }
}

fn required_booleans_subset(child: &Value, parent: &Value) -> bool {
    BOOLEAN_NARROWING_KEYS.iter().all(|keys| {
        let parent_required = bool_value(parent, keys).unwrap_or(false);
        !parent_required || bool_value(child, keys).unwrap_or(false)
    })
}

fn optional_u64_lte_when_parent_set(child: &Value, parent: &Value, keys: &[&str]) -> bool {
    match u64_value(parent, keys) {
        Some(parent_value) => u64_value(child, keys).is_some_and(|child_value| child_value <= parent_value),
        None => true,
    }
}

fn single_use_spend_capability_for_reserve_or_spend(term: &AuthorityTerm, payment: &Value) -> bool {
    let requires_single_use = term
        .verbs
        .iter()
        .any(|verb| matches!(verb, AuthorityVerb::Spend | AuthorityVerb::Reserve));
    !requires_single_use
        || bool_value(
            payment,
            &[
                "single_use_spend_capability",
                "singleUseSpendCapability",
            ],
        )
        .unwrap_or(false)
}

fn string_value<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    value_for_any_key(value, keys)?.as_str()
}

fn u64_value(value: &Value, keys: &[&str]) -> Option<u64> {
    value_for_any_key(value, keys)?.as_u64()
}

fn bool_value(value: &Value, keys: &[&str]) -> Option<bool> {
    value_for_any_key(value, keys)?.as_bool()
}

fn value_for_any_key<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    let object = value.as_object()?;
    keys.iter().find_map(|key| object.get(*key))
}
