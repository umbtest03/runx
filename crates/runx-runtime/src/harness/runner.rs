use std::path::{Path, PathBuf};

use runx_contracts::HarnessReceipt;
use thiserror::Error;

use crate::RuntimeError;
use crate::adapter::{SkillAdapter, SkillInvocation, SkillOutput};
use crate::graph::load_skill;
use crate::harness::assertions::{assert_expectations, status_from_disposition};
use crate::harness::fixtures::{
    HarnessExpectedStatus, HarnessFixture, HarnessFixtureError, HarnessFixtureKind,
    fixture_kind_name, load_harness_fixture,
};
use crate::receipts::step_receipt;
use crate::runner::{GraphRun, Runtime, RuntimeOptions};

#[derive(Clone, Debug)]
pub struct HarnessReplayOutput {
    pub fixture: HarnessFixture,
    pub status: HarnessExpectedStatus,
    pub receipt: HarnessReceipt,
    pub step_receipts: Vec<HarnessReceipt>,
    pub skill_output: Option<SkillOutput>,
}

#[derive(Debug, Error)]
pub enum HarnessReplayError {
    #[error(transparent)]
    Fixture(#[from] HarnessFixtureError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error("harness fixture target {target} has no parent directory")]
    TargetWithoutParent { target: PathBuf },
    #[error("harness expectation mismatch at {field}: expected {expected}, actual {actual}")]
    Mismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
    #[error("harness receipt digest failed: {message}")]
    ReceiptDigest { message: String },
    #[error("harness receipt proof failed for {receipt_id}: {findings}")]
    ReceiptProofInvalid {
        receipt_id: String,
        findings: String,
    },
    #[error("harness fixture mode {mode} at {field_path} is not yet supported by the Rust harness")]
    UnsupportedFixtureMode { mode: String, field_path: String },
    #[error(
        "native cli-tool harness replay is unavailable because runx-runtime was built without the cli-tool feature"
    )]
    CliToolFeatureDisabled,
}

pub fn run_harness_fixture(
    fixture_path: impl AsRef<Path>,
) -> Result<HarnessReplayOutput, HarnessReplayError> {
    #[cfg(feature = "cli-tool")]
    {
        run_harness_fixture_with_adapter(
            fixture_path,
            crate::adapters::cli_tool::CliToolAdapter,
            RuntimeOptions::default(),
        )
    }
    #[cfg(not(feature = "cli-tool"))]
    {
        let _ = fixture_path;
        Err(HarnessReplayError::CliToolFeatureDisabled)
    }
}

#[cfg(feature = "cli-tool")]
pub fn run_harness_fixture_cli_tool(
    fixture_path: impl AsRef<Path>,
) -> Result<HarnessReplayOutput, HarnessReplayError> {
    run_harness_fixture_with_adapter(
        fixture_path,
        crate::adapters::cli_tool::CliToolAdapter,
        RuntimeOptions::default(),
    )
}

pub fn run_harness_fixture_with_adapter<A>(
    fixture_path: impl AsRef<Path>,
    adapter: A,
    options: RuntimeOptions,
) -> Result<HarnessReplayOutput, HarnessReplayError>
where
    A: SkillAdapter,
{
    let fixture_path = fixture_path.as_ref();
    let fixture = load_harness_fixture(fixture_path)?;
    let target_path = resolve_target_path(fixture_path, &fixture.target)?;
    let output = match fixture.kind {
        HarnessFixtureKind::Skill => run_skill_fixture(&fixture, target_path, adapter, options)?,
        HarnessFixtureKind::Graph => run_graph_fixture(&fixture, &target_path, adapter, options)?,
        HarnessFixtureKind::Mcp
        | HarnessFixtureKind::A2a
        | HarnessFixtureKind::Agent
        | HarnessFixtureKind::AgentStep => {
            return Err(HarnessReplayError::UnsupportedFixtureMode {
                mode: fixture_kind_name(&fixture.kind).to_owned(),
                field_path: "kind".to_owned(),
            });
        }
    };
    assert_expectations(&output)?;
    Ok(output)
}

fn run_skill_fixture<A>(
    fixture: &HarnessFixture,
    skill_dir: PathBuf,
    adapter: A,
    options: RuntimeOptions,
) -> Result<HarnessReplayOutput, HarnessReplayError>
where
    A: SkillAdapter,
{
    let skill = load_skill(&skill_dir)?;
    reject_unsupported_source_type(&skill.source.source_type)?;
    let mut env = options.env.clone();
    env.extend(fixture.env.clone());
    let skill_name = skill.name.clone();
    let skill_output = adapter.invoke(SkillInvocation {
        skill_name: skill.name,
        source: skill.source,
        inputs: fixture.inputs.clone(),
        skill_directory: skill_dir,
        env,
    })?;
    let receipt = step_receipt(
        &fixture.name,
        &skill_name,
        1,
        &skill_output,
        &options.created_at,
    )?;
    Ok(HarnessReplayOutput {
        fixture: fixture.clone(),
        status: status_from_disposition(&receipt.seal.disposition),
        receipt,
        step_receipts: Vec::new(),
        skill_output: Some(skill_output),
    })
}

fn reject_unsupported_source_type(source_type: &str) -> Result<(), HarnessReplayError> {
    match source_type {
        "cli-tool" | "harness-hook" => Ok(()),
        "mcp" | "a2a" | "agent" | "agent-step" => Err(HarnessReplayError::UnsupportedFixtureMode {
            mode: source_type.to_owned(),
            field_path: "source.type".to_owned(),
        }),
        other => Err(HarnessReplayError::UnsupportedFixtureMode {
            mode: other.to_owned(),
            field_path: "source.type".to_owned(),
        }),
    }
}

fn run_graph_fixture<A>(
    fixture: &HarnessFixture,
    graph_path: &Path,
    adapter: A,
    mut options: RuntimeOptions,
) -> Result<HarnessReplayOutput, HarnessReplayError>
where
    A: SkillAdapter,
{
    options.env.extend(fixture.env.clone());
    let runtime = Runtime::new(adapter, options);
    let graph_run = runtime.run_graph_file(graph_path)?;
    let output = replay_output_from_graph(fixture, graph_run);
    Ok(output)
}

fn replay_output_from_graph(fixture: &HarnessFixture, graph_run: GraphRun) -> HarnessReplayOutput {
    let step_receipts = graph_run
        .steps
        .iter()
        .map(|step| step.receipt.clone())
        .collect::<Vec<_>>();
    HarnessReplayOutput {
        fixture: fixture.clone(),
        status: status_from_disposition(&graph_run.receipt.seal.disposition),
        receipt: graph_run.receipt,
        step_receipts,
        skill_output: None,
    }
}

fn resolve_target_path(fixture_path: &Path, target: &str) -> Result<PathBuf, HarnessReplayError> {
    let Some(parent) = fixture_path.parent() else {
        return Err(HarnessReplayError::TargetWithoutParent {
            target: fixture_path.to_path_buf(),
        });
    };
    Ok(parent.join(target))
}
