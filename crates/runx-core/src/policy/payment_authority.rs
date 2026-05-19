use runx_contracts::{
    AuthorityCapability, AuthorityCondition, AuthorityResourceFamily, AuthorityTerm, AuthorityVerb,
    PaymentAuthorityBounds, PaymentCredentialForm, Reference,
};

/// Returns true when `child` is no broader than `parent` under the pure payment
/// authority algebra.
///
/// The comparator is intentionally fail-closed: missing required payment
/// dimensions make the terms incomparable, and incomparable terms are denied by
/// returning false.
#[must_use]
pub fn is_payment_authority_subset(child: &AuthorityTerm, parent: &AuthorityTerm) -> bool {
    child.resource_family == AuthorityResourceFamily::Payment
        && parent.resource_family == AuthorityResourceFamily::Payment
        && same_authority_resource(&child.resource_ref, &parent.resource_ref)
        && verbs_subset(&child.verbs, &parent.verbs)
        && capabilities_subset(child, parent)
        && parent_conditions_preserved(child, parent)
        && parent_approvals_preserved(child, parent)
        && expiry_subset(child, parent)
        && payment_bounds_subset(child, parent)
}

fn same_authority_resource(child: &Reference, parent: &Reference) -> bool {
    child.reference_type == parent.reference_type && child.uri == parent.uri
}

fn payment_bounds_subset(child: &AuthorityTerm, parent: &AuthorityTerm) -> bool {
    let Some(child_payment) = child.bounds.payment.as_ref() else {
        return false;
    };
    let Some(parent_payment) = parent.bounds.payment.as_ref() else {
        return false;
    };

    required_currency_equal(child_payment, parent_payment)
        && minor_unit_caps_subset(child, child_payment, parent_payment)
        && rails_subset(child_payment, parent_payment)
        && optional_exact_or_narrower(&child_payment.realm, &parent_payment.realm)
        && optional_exact_or_narrower(&child_payment.counterparty, &parent_payment.counterparty)
        && optional_exact_or_narrower(&child_payment.operation, &parent_payment.operation)
        && optional_exact_or_narrower(&child_payment.period, &parent_payment.period)
        && required_booleans_subset(child_payment, parent_payment)
        && optional_u64_lte_when_parent_set(child_payment.quote_ttl_ms, parent_payment.quote_ttl_ms)
        && optional_u64_lte_when_parent_set(
            child_payment.approval_threshold_minor,
            parent_payment.approval_threshold_minor,
        )
        && optional_exact_or_narrower(
            &child_payment.credential_form,
            &parent_payment.credential_form,
        )
        && single_use_spend_capability_for_reserve_or_spend(child, parent)
}

fn verbs_subset(child: &[AuthorityVerb], parent: &[AuthorityVerb]) -> bool {
    child.iter().all(|verb| parent.contains(verb))
}

fn capabilities_subset(child: &AuthorityTerm, parent: &AuthorityTerm) -> bool {
    child
        .capabilities
        .iter()
        .all(|capability| parent.capabilities.contains(capability))
}

fn parent_conditions_preserved(child: &AuthorityTerm, parent: &AuthorityTerm) -> bool {
    parent
        .conditions
        .iter()
        .all(|condition| condition_is_preserved(condition, &child.conditions))
}

fn condition_is_preserved(
    parent: &AuthorityCondition,
    child_conditions: &[AuthorityCondition],
) -> bool {
    child_conditions.iter().any(|child| child == parent)
}

fn parent_approvals_preserved(child: &AuthorityTerm, parent: &AuthorityTerm) -> bool {
    parent
        .approvals
        .iter()
        .all(|approval| child.approvals.contains(approval))
}

fn expiry_subset(child: &AuthorityTerm, parent: &AuthorityTerm) -> bool {
    match (&child.expires_at, &parent.expires_at) {
        (_, None) => true,
        (Some(child), Some(parent)) => child <= parent,
        (None, Some(_)) => false,
    }
}

fn required_currency_equal(
    child: &PaymentAuthorityBounds,
    parent: &PaymentAuthorityBounds,
) -> bool {
    child.currency == parent.currency
}

fn minor_unit_caps_subset(
    child: &AuthorityTerm,
    child_payment: &PaymentAuthorityBounds,
    parent_payment: &PaymentAuthorityBounds,
) -> bool {
    if uses_minor_units(child) && child_payment.max_per_call_minor.is_none() {
        return false;
    }
    if uses_minor_units(child) && parent_payment.max_per_call_minor.is_none() {
        return false;
    }

    optional_cap_subset(
        child_payment.max_per_call_minor,
        parent_payment.max_per_call_minor,
    ) && optional_cap_subset(
        child_payment.max_per_run_minor,
        parent_payment.max_per_run_minor,
    ) && optional_cap_subset(
        child_payment.max_per_period_minor,
        parent_payment.max_per_period_minor,
    )
}

fn uses_minor_units(term: &AuthorityTerm) -> bool {
    term.verbs.iter().any(|verb| {
        matches!(
            verb,
            AuthorityVerb::Quote
                | AuthorityVerb::Reserve
                | AuthorityVerb::Spend
                | AuthorityVerb::Refund
        )
    })
}

fn optional_cap_subset(child: Option<u64>, parent: Option<u64>) -> bool {
    match (child, parent) {
        (Some(child), Some(parent)) => child <= parent,
        (None, Some(_)) => false,
        (Some(_), None) | (None, None) => true,
    }
}

fn rails_subset(child: &PaymentAuthorityBounds, parent: &PaymentAuthorityBounds) -> bool {
    !child.rails.is_empty()
        && !parent.rails.is_empty()
        && child.rails.iter().all(|rail| parent.rails.contains(rail))
}

fn optional_exact_or_narrower<T: Eq>(child: &Option<T>, parent: &Option<T>) -> bool {
    match (child, parent) {
        (_, None) => true,
        (Some(child), Some(parent)) => child == parent,
        (None, Some(_)) => false,
    }
}

fn required_booleans_subset(
    child: &PaymentAuthorityBounds,
    parent: &PaymentAuthorityBounds,
) -> bool {
    (!parent.quote_required || child.quote_required)
        && (!parent.reservation_required || child.reservation_required)
        && (!parent.idempotency_required || child.idempotency_required)
        && (!parent.recovery_required || child.recovery_required)
        && (!parent.receipt_before_success || child.receipt_before_success)
}

fn optional_u64_lte_when_parent_set(child: Option<u64>, parent: Option<u64>) -> bool {
    match parent {
        Some(parent) => child.is_some_and(|child| child <= parent),
        None => true,
    }
}

fn single_use_spend_capability_for_reserve_or_spend(
    child: &AuthorityTerm,
    parent: &AuthorityTerm,
) -> bool {
    if !requires_single_use_capability(child) {
        return true;
    }
    let child_payment = child.bounds.payment.as_ref();
    let parent_payment = parent.bounds.payment.as_ref();
    child_payment.is_some_and(|payment| payment.single_use_spend)
        && parent_payment.is_some_and(|payment| payment.single_use_spend)
        && child
            .capabilities
            .contains(&AuthorityCapability::PaymentSingleUseSpend)
        && parent
            .capabilities
            .contains(&AuthorityCapability::PaymentSingleUseSpend)
        && child
            .bounds
            .payment
            .as_ref()
            .and_then(|payment| payment.credential_form.as_ref())
            == Some(&PaymentCredentialForm::SingleUseSpendCapability)
}

fn requires_single_use_capability(term: &AuthorityTerm) -> bool {
    term.verbs
        .iter()
        .any(|verb| matches!(verb, AuthorityVerb::Spend))
}
