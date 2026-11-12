//! Deterministic extension resolution and bounded execution-evidence seams.
//!
//! Flow treats extension manifests as declarations, operator locks as authority,
//! and provider events and results as untrusted evidence until validation has
//! completed. It can invoke a caller-supplied in-process port or validate an
//! already-captured external-process transcript after exact subject and
//! authority/isolation preflight. Its local runner can launch the exact
//! trusted-unconfined executable with bounded transport, timeout/cancellation
//! control, and direct-child reaping; sandboxing, descendant containment, and
//! automatic graph recovery remain separate work. The durable coordinator stores
//! prepared intent, acceptance, and explicit recovery decisions in a local workspace.

pub mod ai_preparation;
pub mod artifacts;
pub mod authority;
pub mod catalog;
pub mod command;
pub mod contracts;
pub mod execution;
pub mod execution_subjects;
pub mod hermetic;
pub mod local_input;
pub mod native;
pub mod optiflow;
pub mod process;
pub mod release_cohort;
pub mod release;
pub mod runner;
pub mod scenario;
pub mod state;

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
    CancellationSignal, LocalProcessRunner, NeverCancelled, NoSecrets, ProcessPipe,
    ProcessRunnerError, ProcessWorker, SecretResolver, SecretValue,
};
pub use scenario::*;
pub use state::*;
