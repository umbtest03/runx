//! Rust SDK placeholder for runx clients and host protocol helpers.
//!
//! This crate is a placeholder. The first implementation will call the
//! authoritative `runx --json` CLI rather than executing skills natively.
//! SDK v0 depends on `runx-contracts`, not `runx-core`.

pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const ROLE: &str = "CLI-backed Rust SDK and host protocol helpers";
pub const IS_PLACEHOLDER: bool = true;

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_matches() {
        assert_eq!(crate::PACKAGE_NAME, "runx-sdk");
    }
}
