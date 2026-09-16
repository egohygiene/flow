//! Deterministic extension resolution and an injected in-process execution seam.
//!
//! Flow treats extension manifests as declarations, operator locks as authority,
//! and provider events and results as untrusted evidence until validation has
//! completed.

pub mod catalog;
pub mod contracts;
pub mod execution;
pub mod hermetic;

pub use catalog::{
    CatalogError, ExtensionCatalog, ExtensionObservation, ResolutionOutcome, ResolutionRequest,
    ResolvedExtension,
};
pub use contracts::*;
pub use execution::{
    EventSink, EventSinkError, ExecutionError, ExtensionPort, Orchestrator, PortError,
    PortIdentity, ValidatedExecution,
};
pub use hermetic::{HermeticBehavior, HermeticExtension};
