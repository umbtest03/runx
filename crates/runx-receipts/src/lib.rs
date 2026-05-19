//! Pure Rust harness receipt verification for runx.
//!
//! This crate owns the post-cutover receipt layer: a receipt is the sealed
//! proof of a harness node, with acts and decisions proven through that seal.

mod canonical;
mod tree;
mod verify;

pub use canonical::{canonical_receipt_digest, canonical_receipt_json};
pub use runx_contracts::{
    HARNESS_RECEIPT_SCHEMA, HarnessReceipt, HarnessReceiptSchema, HarnessSeal, HarnessState,
    ReceiptIssuer, ReceiptIssuerType, ReceiptSignature, SealCriterion, SignatureAlgorithm,
};
pub use tree::{validate_receipt_tree, verify_receipt_tree};
pub use verify::{
    ReceiptError, ReceiptFinding, ReceiptFindingCode, ReceiptVerification, validate_harness,
    validate_harness_receipt, verify_harness, verify_harness_receipt,
};

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_matches() {
        assert_eq!(env!("CARGO_PKG_NAME"), "runx-receipts");
    }
}
