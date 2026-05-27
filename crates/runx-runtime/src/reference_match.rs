use runx_contracts::Reference;

pub(crate) fn same_reference(left: &Reference, right: &Reference) -> bool {
    left.reference_type == right.reference_type && left.uri == right.uri
}
