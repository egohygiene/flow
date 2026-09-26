//! Versioned local run state and durable coordination of prepared process steps.
//!
//! The store records intent before launch and acceptance after host validation.
//! Reopening never launches a provider or recreates opaque trust tokens. See
//! `docs/integrations/durable-state.md` for storage and recovery limits.

mod execution;
mod model;
mod store;

pub use execution::{
    DurableExecutionError, ProcessStepContext, RecoveryApproval, ResumeEligibility,
    validation_implementation_digest,
};
pub use model::*;
pub use store::RunStore;

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// A precise boundary whose current evidence disagrees with saved intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateBoundary {
    Plan,
    Invocation,
    Provider,
    Capability,
    Configuration,
    Authority,
    Bindings,
    Inputs,
    Artifacts,
    Validation,
}

/// Store failures never carry raw provider messages or source bytes.
#[derive(Debug, Error)]
pub enum StateError {
    #[error("the run workspace is already open")]
    Busy,
    #[error("the run workspace already exists")]
    AlreadyExists,
    #[error("the run workspace contains an incomplete write; preserve it for recovery")]
    IncompleteWrite,
    #[error("the run workspace contains an unexpected or unsafe filesystem entry")]
    UnsafeEntry,
    #[error("the durable record exceeds its resource budget")]
    Limit,
    #[error("unsupported durable record schema")]
    UnsupportedSchema,
    #[error("malformed durable record")]
    Malformed,
    #[error("durable record identity or sequence is corrupt")]
    Corrupt,
    #[error("invalid durable state: {rule}")]
    Invalid { rule: &'static str },
    #[error("current evidence disagrees with durable state at {boundary:?}")]
    Stale { boundary: StateBoundary },
    #[error("step is not ready for execution or the requested transition")]
    Transition,
    #[error("step identifier does not exist in this plan")]
    UnknownStep,
    #[error("the store must be reopened after a failed commit")]
    ReopenRequired,
    #[error("workspace I/O failed during {operation}")]
    Io {
        operation: &'static str,
        #[source]
        source: std::io::Error,
    },
}

fn require(condition: bool, rule: &'static str) -> Result<(), StateError> {
    if condition {
        Ok(())
    } else {
        Err(StateError::Invalid { rule })
    }
}

fn schema(actual: &str, expected: &str) -> Result<(), StateError> {
    if actual == expected {
        Ok(())
    } else {
        Err(StateError::UnsupportedSchema)
    }
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_id(value: &str, prefix: &str) -> bool {
    value.len() <= 160 && crate::artifacts::has_prefixed_id(value, prefix)
}

fn canonical_bytes(value: &impl Serialize) -> Result<Vec<u8>, StateError> {
    fn ordered(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let sorted: std::collections::BTreeMap<_, _> = map.into_iter().collect();
                Value::Object(
                    sorted
                        .into_iter()
                        .map(|(key, value)| (key, ordered(value)))
                        .collect(),
                )
            }
            Value::Array(values) => Value::Array(values.into_iter().map(ordered).collect()),
            other => other,
        }
    }
    serde_json::to_vec(&ordered(
        serde_json::to_value(value).map_err(|_| StateError::Malformed)?,
    ))
    .map_err(|_| StateError::Malformed)
}

fn digest(value: &impl Serialize) -> Result<String, StateError> {
    Ok(format!("{:x}", Sha256::digest(canonical_bytes(value)?)))
}

fn io_error(operation: &'static str, source: std::io::Error) -> StateError {
    StateError::Io { operation, source }
}
