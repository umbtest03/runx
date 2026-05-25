//! CLI-backed Rust SDK for runx clients and host protocol helpers.
//!
//! SDK v0 calls the authoritative `runx --json` CLI. It does not execute
//! skills natively and does not replace the TypeScript runtime.

pub mod act;
pub mod client;
pub mod command;
pub mod error;
pub mod host;

pub use client::{
    ContinuePayload, RunSkillOptions, RunxClient, RunxClientOptions, RunxJsonReport,
    SkillSearchResult,
};
pub use error::{RunxError, RunxResult};

pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const ROLE: &str = "CLI-backed Rust SDK";

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_matches() {
        assert_eq!(crate::PACKAGE_NAME, "runx-sdk");
    }
}
