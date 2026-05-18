//! Native Rust runtime placeholder for runx execution and adapters.
//!
//! This crate is intentionally empty of runtime behavior until the pure crates
//! and feature-parity matrix are ready. Adapter families are planned as runtime
//! features instead of a separate crate.

pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const ROLE: &str = "native runtime for execution, adapter features, MCP, and sandboxing";
pub const IS_PLACEHOLDER: bool = true;

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_matches() {
        assert_eq!(crate::PACKAGE_NAME, "runx-runtime");
    }
}
