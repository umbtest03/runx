use std::collections::BTreeMap;
use std::path::PathBuf;

use runx_contracts::JsonObject;
pub use runx_contracts::{
    DevFixtureAssertion, DevFixtureAssertionKind, DevFixtureResult, DevFixtureStatus, DevReport,
    DevReportSchema, DevReportStatus,
};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevLoopOptions {
    pub root: PathBuf,
    pub unit_path: Option<PathBuf>,
    pub lane: DevLane,
}

impl DevLoopOptions {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            unit_path: None,
            lane: DevLane::Deterministic,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DevLane {
    Deterministic,
    RepoIntegration,
    Agent,
    All,
    Other(String),
}

impl DevLane {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Deterministic => "deterministic",
            Self::RepoIntegration => "repo-integration",
            Self::Agent => "agent",
            Self::All => "all",
            Self::Other(value) => value,
        }
    }
}

impl From<&str> for DevLane {
    fn from(value: &str) -> Self {
        match value {
            "deterministic" => Self::Deterministic,
            "repo-integration" => Self::RepoIntegration,
            "agent" => Self::Agent,
            "all" => Self::All,
            other => Self::Other(other.to_owned()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedDevFixture {
    pub path: PathBuf,
    pub name: String,
    pub lane: String,
    pub target: JsonObject,
    pub document: JsonObject,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevFixtureExecutionRoots {
    pub cwd: PathBuf,
    pub repo_root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PreparedDevFixtureWorkspace {
    pub root: Option<PathBuf>,
    pub tokens: BTreeMap<String, String>,
}

#[derive(Debug, Error)]
pub enum DevError {
    #[error("failed to read dev fixture {path}: {source}")]
    ReadFixture {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse dev fixture {path}: {source}")]
    ParseFixture {
        path: PathBuf,
        #[source]
        source: serde_norway::Error,
    },
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse JSON at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("dev fixture workspace path must be relative: {path}")]
    AbsoluteWorkspacePath { path: String },
    #[error("dev fixture workspace path escapes root: {path}")]
    EscapingWorkspacePath { path: String },
    #[error("failed to run fixture command {command}: {source}")]
    Spawn {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("dev fixture command `{command}` failed with status {status}: {output}")]
    FixtureCommand {
        command: String,
        status: i32,
        output: String,
    },
    #[error(transparent)]
    Runtime(#[from] crate::RuntimeError),
}

pub trait DevFixtureExecutor {
    fn run_fixture(
        &self,
        root: &std::path::Path,
        fixture: &ParsedDevFixture,
    ) -> Result<DevFixtureResult, DevError>;
}

#[derive(Clone, Debug, Default)]
pub struct LocalDevFixtureExecutor;
