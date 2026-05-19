//! Native Rust runtime skeleton for runx execution.
//!
//! The runtime owns impure boundaries: filesystem reads, subprocess execution,
//! sandbox preparation, caller reporting, and harness receipt emission. Pure
//! parser/core/receipt crates stay upstream of this crate.

pub mod adapter;
pub mod caller;
pub mod doctor;
pub mod error;
mod fanout;
mod graph;
pub mod harness;
pub mod journal;
pub mod receipt_paths;
pub mod receipt_store;
pub mod receipt_tree;
pub mod receipts;
pub mod registry;
pub mod runner;
pub mod sandbox;
pub mod scaffold;
pub mod tool_catalogs;

#[cfg(any(
    feature = "cli-tool",
    feature = "catalog",
    feature = "mcp",
    feature = "a2a",
    feature = "agent"
))]
pub mod adapters;

pub use adapter::{InvocationStatus, SkillAdapter, SkillInvocation, SkillOutput};
pub use caller::{Caller, NoopCaller};
pub use doctor::{DoctorOptions, default_doctor_options, run_doctor};
pub use error::RuntimeError;
pub use harness::{
    HarnessExpectedStatus, HarnessFixture, HarnessFixtureError, HarnessFixtureKind,
    HarnessReceiptExpectation, HarnessReplayError, HarnessReplayOutput, HarnessReplayReceipt,
    load_harness_fixture, parse_harness_fixture, run_harness_fixture,
    run_harness_fixture_with_adapter,
};
pub use journal::{ExecutionJournal, JournalEntry};
pub use receipt_paths::{
    INIT_CWD_ENV, RUNTIME_RECEIPTS_DIR_CONFIG_KEY, RUNX_CWD_ENV, RUNX_PROJECT_DIR_ENV,
    RUNX_RECEIPT_DIR_ENV, ReceiptPathInputs, ReceiptPathSource, ReceiptStoreLabel,
    ResolvedReceiptPath, RuntimeReceiptConfig, resolve_project_runx_dir, resolve_receipt_path,
    resolve_workspace_base, safe_receipt_store_label,
};
pub use receipt_store::{
    LocalReceiptStore, ReceiptStoreError, ReceiptStoreIndex, ReceiptStoreIndexEntry,
};
pub use receipt_tree::{
    RuntimeReceiptResolver, validate_runtime_receipt_tree, verify_runtime_receipt_tree,
};
pub use registry::{RegistryInstallMetadataInput, registry_install_receipt_metadata};
#[cfg(feature = "cli-tool")]
pub use runner::run_graph_file;
pub use runner::{GraphCheckpoint, GraphRun, Runtime, RuntimeOptions, StepRun};
pub use scaffold::{
    InitAction, InitGeneratedValues, RunxInitOptions, RunxInitResult, RunxInstallState,
    RunxNewOptions, RunxNewResult, RunxProjectState, ScaffoldError, ensure_runx_install_state,
    ensure_runx_project_state, packet_namespace_for_name, runx_init, sanitize_runx_package_name,
    scaffold_runx_package,
};
pub use tool_catalogs::{
    ToolBuildOptions, ToolCatalogError, ToolInspectOptions, ToolSearchOptions, build_tool_catalogs,
    inspect_tool, search_tools,
};

pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
