use std::fmt;

use crate::OperationalPolicyError;

use super::PostMergeObserverSignalSource;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostMergeObserverPlanError {
    Policy(OperationalPolicyError),
    ProviderObservationDisabled,
    SourceRequired,
    UnknownSource(String),
    ProviderStateNotTerminal,
    MissingSourceThread {
        source_id: String,
    },
    MissingObserverSignal {
        signal_source: PostMergeObserverSignalSource,
    },
    InvalidObserverCommandReference {
        field: &'static str,
        expected: &'static str,
    },
    MissingObserverCommandReferenceMetadata {
        field: &'static str,
    },
    UnsupportedObserverCommandProvider {
        field: &'static str,
        provider: String,
    },
    VerificationRequired,
    InconsistentObservation(String),
    ReceiptNotSealed,
    ReceiptNotPostMergeObserver,
    MissingReceiptCriterion(String),
    MissingReceiptReference(&'static str),
    MissingReceiptMetadata(&'static str),
    ReceiptPublicationNotAuthorized(String),
}

impl fmt::Display for PostMergeObserverPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Policy(error) => write!(formatter, "operational policy error: {error}"),
            Self::ProviderObservationDisabled
            | Self::SourceRequired
            | Self::UnknownSource(_)
            | Self::ProviderStateNotTerminal
            | Self::MissingSourceThread { .. }
            | Self::VerificationRequired
            | Self::InconsistentObservation(_) => self.fmt_planning_error(formatter),
            Self::MissingObserverSignal { .. }
            | Self::InvalidObserverCommandReference { .. }
            | Self::MissingObserverCommandReferenceMetadata { .. }
            | Self::UnsupportedObserverCommandProvider { .. } => self.fmt_command_error(formatter),
            Self::ReceiptNotSealed
            | Self::ReceiptNotPostMergeObserver
            | Self::MissingReceiptCriterion(_)
            | Self::MissingReceiptReference(_)
            | Self::MissingReceiptMetadata(_)
            | Self::ReceiptPublicationNotAuthorized(_) => self.fmt_receipt_error(formatter),
        }
    }
}

impl PostMergeObserverPlanError {
    fn fmt_planning_error(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderObservationDisabled => {
                formatter.write_str("post-merge observer planning requires observe_provider=true")
            }
            Self::SourceRequired => {
                formatter.write_str("post-merge observer planning requires a source_id")
            }
            Self::UnknownSource(source_id) => {
                write!(
                    formatter,
                    "post-merge observer planning references unknown source '{source_id}'"
                )
            }
            Self::ProviderStateNotTerminal => {
                formatter.write_str("post-merge observer planning requires terminal PR state")
            }
            Self::MissingSourceThread { source_id } => {
                write!(
                    formatter,
                    "source '{source_id}' requires a source-thread target before final publication"
                )
            }
            Self::VerificationRequired => {
                formatter.write_str("merged post-merge observer planning requires verification")
            }
            Self::InconsistentObservation(message) => formatter.write_str(message),
            _ => formatter.write_str("post-merge observer planning error category mismatch"),
        }
    }

    fn fmt_command_error(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingObserverSignal { signal_source } => {
                write!(
                    formatter,
                    "post-merge observer {signal_source:?} command requires a signal reference"
                )
            }
            Self::InvalidObserverCommandReference { field, expected } => {
                write!(
                    formatter,
                    "post-merge observer command field '{field}' must be a {expected} reference"
                )
            }
            Self::MissingObserverCommandReferenceMetadata { field } => {
                write!(
                    formatter,
                    "post-merge observer command field '{field}' requires provider and locator metadata"
                )
            }
            Self::UnsupportedObserverCommandProvider { field, provider } => {
                write!(
                    formatter,
                    "post-merge observer command field '{field}' has unsupported provider '{provider}'"
                )
            }
            _ => formatter.write_str("post-merge observer command error category mismatch"),
        }
    }

    fn fmt_receipt_error(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReceiptNotSealed => {
                formatter.write_str("post-merge publication requires a sealed harness receipt")
            }
            Self::ReceiptNotPostMergeObserver => {
                formatter.write_str("sealed harness receipt is not a post-merge observer closure")
            }
            Self::MissingReceiptCriterion(criterion_id) => {
                write!(
                    formatter,
                    "sealed post-merge receipt is missing required criterion '{criterion_id}'"
                )
            }
            Self::MissingReceiptReference(kind) => {
                write!(
                    formatter,
                    "sealed post-merge receipt is missing required {kind} reference"
                )
            }
            Self::MissingReceiptMetadata(kind) => {
                write!(
                    formatter,
                    "sealed post-merge receipt is missing required {kind} metadata"
                )
            }
            Self::ReceiptPublicationNotAuthorized(message) => formatter.write_str(message),
            _ => formatter.write_str("post-merge observer receipt error category mismatch"),
        }
    }
}

impl std::error::Error for PostMergeObserverPlanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Policy(error) => Some(error),
            _ => None,
        }
    }
}
