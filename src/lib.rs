//! Deterministic extension resolution and bounded execution-evidence seams.
//!
//! Flow treats extension manifests as declarations, operator locks as authority,
//! and provider events and results as untrusted evidence until validation has
//! completed. It can invoke a caller-supplied in-process port or validate an
//! already-captured external-process transcript after exact subject and
//! authority/isolation preflight. Its local runner can launch the exact
//! trusted-unconfined executable; sandboxing and interrupted lifecycle control
//! remain separate work.

pub mod artifacts;
pub mod authority;
pub mod catalog;
pub mod contracts;
pub mod execution;
pub mod execution_subjects;
pub mod hermetic;
pub mod process;
pub mod runner;
pub mod scenario;

pub use artifacts::*;
pub use authority::*;
pub use catalog::{
    CatalogError, ExtensionCatalog, ExtensionObservation, ResolutionOutcome, ResolutionRequest,
    ResolvedExtension,
};
pub use contracts::*;
pub use execution::{
    EventSink, EventSinkError, ExecutionError, ExtensionPort, Orchestrator, PortError,
    PortIdentity, ProcessCompletion, ProcessStream, ProcessTranscript, ValidatedExecution,
};
pub use execution_subjects::*;
pub use hermetic::{HermeticBehavior, HermeticExtension};
pub use process::{
    DecodedProcessTranscript, ProcessProtocolError, decode_provider_stdout, encode_invocation_frame,
};
pub use runner::{
    LocalProcessRunner, NoSecrets, ProcessPipe, ProcessRunnerError, ProcessWorker, SecretResolver,
    SecretValue,
};
pub use scenario::*;
