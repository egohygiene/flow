//! Deterministic extension resolution and bounded execution-evidence seams.
//!
//! Flow treats extension manifests as declarations, operator locks as authority,
//! and provider events and results as untrusted evidence until validation has
//! completed. It can invoke a caller-supplied in-process port or validate an
//! already-captured external-process transcript; it does not launch processes.

pub mod catalog;
pub mod contracts;
pub mod execution;
pub mod hermetic;
pub mod process;
pub mod scenario;

pub use catalog::{
    CatalogError, ExtensionCatalog, ExtensionObservation, ResolutionOutcome, ResolutionRequest,
    ResolvedExtension,
};
pub use contracts::*;
pub use execution::{
    EventSink, EventSinkError, ExecutionError, ExtensionPort, Orchestrator, PortError,
    PortIdentity, ProcessCompletion, ProcessStream, ProcessTranscript, ValidatedExecution,
};
pub use hermetic::{HermeticBehavior, HermeticExtension};
pub use process::{
    DecodedProcessTranscript, ProcessProtocolError, decode_provider_stdout, encode_invocation_frame,
};
pub use scenario::*;
