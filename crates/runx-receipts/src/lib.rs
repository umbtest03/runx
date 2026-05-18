//! Pure Rust receipt model and verification parity for runx.
//!
//! This crate is a placeholder. The TypeScript receipt implementation remains
//! authoritative until receipt fixtures pass in both languages.

pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const ROLE: &str = "receipt model, canonicalization, and verification parity";
pub const IS_PLACEHOLDER: bool = true;

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_matches() {
        assert_eq!(crate::PACKAGE_NAME, "runx-receipts");
    }
}
