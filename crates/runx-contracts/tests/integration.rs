//! Single integration-test binary for runx-contracts.
//!
//! Each module below is one integration test file, compiled and linked once
//! as a single binary instead of one binary per file. `autotests = false` in
//! Cargo.toml keeps Cargo from also building each file as its own binary.
//! See .scafld/specs/active/test-surface-build-consolidation.md.

mod act_assignment_fixtures;
mod aster_control_fixtures;
mod credential_delivery_fixtures;
mod doctor_fixtures;
mod execution_fixtures;
mod external_adapter_fixtures;
mod harness_spine_fixtures;
mod host_protocol_fixtures;
mod nitrosend_external_fixture;
mod operational_policy;
mod post_merge_observer;
mod reference;
mod schema_generator_check;
mod schema_validation;
mod schema_wire_conformance;
mod target_runner;
mod thread_outbox_provider_fixtures;
