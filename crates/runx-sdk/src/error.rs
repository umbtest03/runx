use std::io;

#[derive(Debug, thiserror::Error)]
pub enum RunxError {
    #[error("runx command is empty")]
    EmptyCommand,
    #[error("runx command failed to start: {0}")]
    Io(#[from] io::Error),
    #[error("runx command exited with status {status:?}: {stderr}")]
    CommandStatus {
        args: Vec<String>,
        status: Option<i32>,
        stderr: String,
    },
    #[error("runx command emitted invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("runx JSON output must be an object")]
    ExpectedObject,
    #[error("runx JSON field `{field}` is required")]
    MissingField { field: &'static str },
    #[error("runx JSON field `{field}` has the wrong shape")]
    InvalidField { field: &'static str },
    #[error("runx command stdin was unavailable")]
    MissingStdin,
}

pub type RunxResult<T> = Result<T, RunxError>;
