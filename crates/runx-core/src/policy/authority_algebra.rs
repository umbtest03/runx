use runx_contracts::Reference;

#[must_use]
pub fn same_reference_address(child: &Reference, parent: &Reference) -> bool {
    child.reference_type == parent.reference_type && child.uri == parent.uri
}

#[must_use]
pub fn items_subset<T: PartialEq>(child: &[T], parent: &[T]) -> bool {
    child.iter().all(|item| parent.contains(item))
}

#[must_use]
pub fn parent_items_preserved<T: PartialEq>(child: &[T], parent: &[T]) -> bool {
    parent.iter().all(|item| child.contains(item))
}

#[must_use]
pub fn optional_exact_or_narrower<T: Eq>(child: &Option<T>, parent: &Option<T>) -> bool {
    match (child, parent) {
        (_, None) => true,
        (Some(child), Some(parent)) => child == parent,
        (None, Some(_)) => false,
    }
}

#[must_use]
pub fn optional_bound_subset<T: Ord + Copy>(child: Option<T>, parent: Option<T>) -> bool {
    match (child, parent) {
        (Some(child), Some(parent)) => child <= parent,
        (None, Some(_)) => false,
        (Some(_), None) | (None, None) => true,
    }
}

#[must_use]
pub fn optional_ref_bound_subset<T: Ord>(child: Option<&T>, parent: Option<&T>) -> bool {
    match (child, parent) {
        (Some(child), Some(parent)) => child <= parent,
        (None, Some(_)) => false,
        (Some(_), None) | (None, None) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::{items_subset, optional_bound_subset, parent_items_preserved};

    #[test]
    fn item_subset_is_reflexive() {
        let values = ["read", "write", "verify"];

        assert!(items_subset(&values, &values));
    }

    #[test]
    fn parent_items_are_preserved_when_child_keeps_parent_requirements() {
        let parent = ["approval", "mfa"];
        let child = ["approval", "mfa", "reason"];

        assert!(parent_items_preserved(&child, &parent));
        assert!(!parent_items_preserved(&["approval"], &parent));
    }

    #[test]
    fn optional_bounds_deny_missing_or_larger_child_bounds() {
        assert!(optional_bound_subset(Some(5_u64), Some(10_u64)));
        assert!(!optional_bound_subset(Some(11_u64), Some(10_u64)));
        assert!(!optional_bound_subset::<u64>(None, Some(10_u64)));
        assert!(optional_bound_subset(Some(10_u64), None));
    }
}
