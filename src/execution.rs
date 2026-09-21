//! Flow-owned validation around one caller-injected in-process extension port.

use std::collections::HashSet;

use thiserror::Error;

use crate::ResolvedExtension;
use crate::authority::AuthorizedProcess;
use crate::contracts::{
    EventKind, ExecutionModeKind, ExtensionEvent, ExtensionInvocation, ExtensionResult,
    InvocationPhase, Outcome, Trust, ValidationError,
};
use crate::execution_subjects::{ExecutionSubjectLock, MatchedExecutionSubjects};
use crate::process::{
    DecodedProcessTranscript, ProcessProtocolError, decode_provider_stdout, encode_invocation_frame,
};

/// Exact identity advertised by an injected in-process port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortIdentity {
    pub extension_id: String,
    pub version: String,
    pub publisher_id: String,
    pub integrity: String,
    pub execution_mode_name: String,
    pub execution_mode_kind: ExecutionModeKind,
}

impl PortIdentity {
    #[must_use]
    pub fn from_resolved(resolved: &ResolvedExtension) -> Self {
        Self {
            extension_id: resolved.extension_id().to_owned(),
            version: resolved.version().to_owned(),
            publisher_id: resolved.publisher_id().to_owned(),
            integrity: resolved.integrity().value.clone(),
            execution_mode_name: resolved.execution_mode().name.clone(),
            execution_mode_kind: resolved.execution_mode().kind,
        }
    }
}

/// A destination for individually validated, correlated provider events.
pub trait EventSink {
    /// Consume one already schema-validated and invocation-correlated event.
    ///
    /// # Errors
    ///
    /// Returns an observer-specific error. Flow stops forwarding subsequent
    /// events and rejects the execution evidence.
    fn emit(&mut self, event: &ExtensionEvent) -> Result<(), EventSinkError>;
}

impl EventSink for Vec<ExtensionEvent> {
    fn emit(&mut self, event: &ExtensionEvent) -> Result<(), EventSinkError> {
        self.push(event.clone());
        Ok(())
    }
}

impl<F> EventSink for F
where
    F: FnMut(&ExtensionEvent) -> Result<(), EventSinkError>,
{
    fn emit(&mut self, event: &ExtensionEvent) -> Result<(), EventSinkError> {
        self(event)
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct EventSinkError {
    pub message: String,
}

impl EventSinkError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// The only runtime seam in this checkpoint.
///
/// Implementations are supplied by the caller. Flow performs no dynamic loading
/// and starts no process here.
pub trait ExtensionPort {
    fn identity(&self) -> &PortIdentity;

    /// Invoke the provider with a Flow-owned request and event boundary.
    ///
    /// # Errors
    ///
    /// Returns a provider failure. Once this method is entered, Flow will not
    /// fall back to another provider.
    fn invoke(
        &self,
        invocation: &ExtensionInvocation,
        events: &mut dyn EventSink,
    ) -> Result<ExtensionResult, PortError>;
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct PortError {
    pub message: String,
}

impl PortError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Host-reported completion for one already-captured provider process.
///
/// This is an observation supplied by a future runner. It does not launch,
/// signal, wait for, or otherwise control a process.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessCompletion {
    /// The process exited. `None` means no portable exit code was available,
    /// for example after signal termination.
    Exited { code: Option<i32> },
    /// The runner reports that the execution deadline elapsed.
    TimedOut,
    /// The runner reports that forced cancellation terminated the process.
    Cancelled,
}

/// Captured process stream whose configured byte bound was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessStream {
    Stdout,
    Stderr,
}

/// Borrowed, host-captured process evidence awaiting Flow validation.
///
/// Raw stdout and stderr remain caller-owned sensitive evidence. Flow never
/// places their bytes in `ValidatedExecution`, portable provenance, or error
/// display output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessTranscript<'a> {
    completion: ProcessCompletion,
    stdout: &'a [u8],
    stderr: &'a [u8],
}

impl<'a> ProcessTranscript<'a> {
    #[must_use]
    pub const fn new(completion: ProcessCompletion, stdout: &'a [u8], stderr: &'a [u8]) -> Self {
        Self {
            completion,
            stdout,
            stderr,
        }
    }

    #[must_use]
    pub const fn completion(self) -> ProcessCompletion {
        self.completion
    }

    #[must_use]
    pub const fn stdout(self) -> &'a [u8] {
        self.stdout
    }

    #[must_use]
    pub const fn stderr(self) -> &'a [u8] {
        self.stderr
    }
}

/// Execution failed before or after invocation. Provider evidence is retained
/// for inspection but is never converted into `ValidatedExecution` on error.
#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("invalid invocation: {source}")]
    InvalidInvocation {
        #[source]
        source: ValidationError,
    },
    #[error("execution preflight mismatch: {message}")]
    Preflight { message: String },
    #[error("provider invocation failed: {source}")]
    Provider {
        #[source]
        source: PortError,
        events: Vec<ExtensionEvent>,
    },
    #[error("provider emitted invalid event evidence: {message}")]
    InvalidEvent {
        message: String,
        events: Vec<ExtensionEvent>,
        result: Option<Box<ExtensionResult>>,
    },
    #[error("event observer rejected provider evidence: {source}")]
    EventSink {
        #[source]
        source: EventSinkError,
        events: Vec<ExtensionEvent>,
        result: Option<Box<ExtensionResult>>,
    },
    #[error("provider returned invalid terminal evidence: {message}")]
    InvalidResult {
        message: String,
        events: Vec<ExtensionEvent>,
        result: Box<ExtensionResult>,
    },
    #[error("external process protocol failed: {source}")]
    ProcessProtocol {
        #[source]
        source: ProcessProtocolError,
    },
    #[error(
        "captured process {stream:?} exceeded its byte limit: observed {observed}, limit {limit}"
    )]
    ProcessOutputLimit {
        stream: ProcessStream,
        limit: u64,
        observed: u64,
    },
    #[error("external process did not exit successfully (portable exit code: {code:?})")]
    ProcessExit { code: Option<i32> },
    #[error("external process timed out")]
    ProcessTimeout,
    #[error("external process was forcibly cancelled")]
    ProcessCancelled,
}

impl ExecutionError {
    /// Raw events attempted by the provider, including the rejected event.
    #[must_use]
    pub fn events(&self) -> &[ExtensionEvent] {
        match self {
            Self::InvalidInvocation { .. }
            | Self::Preflight { .. }
            | Self::ProcessProtocol { .. }
            | Self::ProcessOutputLimit { .. }
            | Self::ProcessExit { .. }
            | Self::ProcessTimeout
            | Self::ProcessCancelled => &[],
            Self::Provider { events, .. }
            | Self::InvalidEvent { events, .. }
            | Self::EventSink { events, .. }
            | Self::InvalidResult { events, .. } => events,
        }
    }

    /// Raw terminal provider evidence, when invocation returned one.
    #[must_use]
    pub fn provider_result(&self) -> Option<&ExtensionResult> {
        match self {
            Self::InvalidEvent { result, .. } | Self::EventSink { result, .. } => result.as_deref(),
            Self::InvalidResult { result, .. } => Some(result.as_ref()),
            Self::InvalidInvocation { .. }
            | Self::Preflight { .. }
            | Self::Provider { .. }
            | Self::ProcessProtocol { .. }
            | Self::ProcessOutputLimit { .. }
            | Self::ProcessExit { .. }
            | Self::ProcessTimeout
            | Self::ProcessCancelled => None,
        }
    }
}

/// Evidence that passed Flow-owned contract and correlation checks.
///
/// This type does not authenticate caller-issued configuration, authorization,
/// or grant digests; canonicalization and authorization builders are outside
/// this checkpoint. Artifact checks correlate provider-reported identifiers
/// only; they do not resolve locators, verify bytes, or perform domain-output
/// validation.
///
/// The fields remain private so unvalidated provider evidence cannot be
/// promoted with a struct literal:
///
/// ```compile_fail
/// use flow::ValidatedExecution;
///
/// let _unvalidated = ValidatedExecution {};
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedExecution {
    events: Vec<ExtensionEvent>,
    result: ExtensionResult,
}

impl ValidatedExecution {
    #[must_use]
    pub fn events(&self) -> &[ExtensionEvent] {
        &self.events
    }

    #[must_use]
    pub fn result(&self) -> &ExtensionResult {
        &self.result
    }

    #[must_use]
    pub fn into_parts(self) -> (Vec<ExtensionEvent>, ExtensionResult) {
        (self.events, self.result)
    }
}

/// Stateless coordinator for one already-resolved invocation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Orchestrator;

impl Orchestrator {
    /// Invoke exactly one injected port and validate all returned evidence.
    ///
    /// The API deliberately accepts no alternate port, making post-invocation
    /// fallback impossible at this seam. The port is trusted in-process code;
    /// this checkpoint does not enforce declared time/output limits, cancel it,
    /// isolate panics, sandbox it, or constrain its side effects.
    ///
    /// # Errors
    ///
    /// Returns an error for preflight mismatches, provider failures, rejected
    /// event observations, observer failures, or invalid terminal evidence.
    pub fn execute(
        resolved: &ResolvedExtension,
        invocation: &ExtensionInvocation,
        port: &dyn ExtensionPort,
        event_sink: &mut dyn EventSink,
    ) -> Result<ValidatedExecution, ExecutionError> {
        invocation
            .validate()
            .map_err(|source| ExecutionError::InvalidInvocation { source })?;
        validate_in_process_preflight(resolved, invocation, port.identity())?;

        let mut validating_sink = ValidatingEventSink::new(invocation, event_sink);
        let provider_result = port.invoke(invocation, &mut validating_sink);
        let (events, issue) = validating_sink.finish();
        match issue {
            Some(EventIssue::Invalid(message)) => {
                return Err(ExecutionError::InvalidEvent {
                    message,
                    events,
                    result: provider_result.ok().map(Box::new),
                });
            }
            Some(EventIssue::Downstream(source)) => {
                return Err(ExecutionError::EventSink {
                    source,
                    events,
                    result: provider_result.ok().map(Box::new),
                });
            }
            None => {}
        }
        let result = provider_result.map_err(|source| ExecutionError::Provider {
            source,
            events: events.clone(),
        })?;

        validate_result(invocation, &events, &result).map_err(|message| {
            ExecutionError::InvalidResult {
                message,
                events: events.clone(),
                result: Box::new(result.clone()),
            }
        })?;
        Ok(ValidatedExecution { events, result })
    }

    /// Encode one validated process-mode invocation for provider stdin.
    ///
    /// Encoding requires a fresh opaque package/executable match for the exact
    /// subject lock and invocation context. The token proves digest equality
    /// to that lock, not publisher authenticity or launch-time file identity.
    ///
    /// The returned compact JSON document ends with exactly one LF. Its bytes
    /// are a deterministic transport projection, not execution identity or a
    /// general canonical-JSON representation.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid invocation, process or execution-subject
    /// preflight mismatch, or serialization failure.
    pub fn encode_process_request(
        resolved: &ResolvedExtension,
        invocation: &ExtensionInvocation,
        subject_lock: &ExecutionSubjectLock,
        subjects: &MatchedExecutionSubjects,
        authority: &AuthorizedProcess,
    ) -> Result<Vec<u8>, ExecutionError> {
        invocation
            .validate()
            .map_err(|source| ExecutionError::InvalidInvocation { source })?;
        validate_common_preflight(resolved, invocation, ExecutionModeKind::Process)?;
        validate_process_subject_preflight(resolved, invocation, subject_lock, subjects)?;
        validate_process_authority_preflight(
            resolved,
            invocation,
            subject_lock,
            subjects,
            authority,
        )?;
        encode_invocation_frame(invocation)
            .map_err(|source| ExecutionError::ProcessProtocol { source })
    }

    /// Validate one already-captured provider process transcript.
    ///
    /// This method is deliberately host-neutral: it does not launch, capture,
    /// time out, cancel, signal, or reap a process. The caller owns those
    /// operations and supplies bounded completion/stdout/stderr observations.
    /// Flow checks the declared byte limits and completion state, parses the
    /// protocol-only stdout stream, and routes the decoded evidence through the
    /// same event/result acceptance gate as in-process execution.
    /// A matching package/executable token is required before any supplied
    /// transcript can enter that gate.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid invocation or preflight evidence,
    /// output limits, abnormal completion, protocol framing, observer
    /// rejection, or invalid provider events/results.
    pub fn validate_process_transcript(
        resolved: &ResolvedExtension,
        invocation: &ExtensionInvocation,
        subject_lock: &ExecutionSubjectLock,
        subjects: &MatchedExecutionSubjects,
        authority: &AuthorizedProcess,
        transcript: ProcessTranscript<'_>,
        event_sink: &mut dyn EventSink,
    ) -> Result<ValidatedExecution, ExecutionError> {
        invocation
            .validate()
            .map_err(|source| ExecutionError::InvalidInvocation { source })?;
        validate_common_preflight(resolved, invocation, ExecutionModeKind::Process)?;
        validate_process_subject_preflight(resolved, invocation, subject_lock, subjects)?;
        validate_process_authority_preflight(
            resolved,
            invocation,
            subject_lock,
            subjects,
            authority,
        )?;

        validate_captured_length(
            ProcessStream::Stdout,
            invocation.limits.max_stdout_bytes,
            transcript.stdout().len(),
        )?;
        validate_captured_length(
            ProcessStream::Stderr,
            invocation.limits.max_stderr_bytes,
            transcript.stderr().len(),
        )?;

        match transcript.completion() {
            ProcessCompletion::Exited { code: Some(0) } => {}
            ProcessCompletion::Exited { code } => {
                return Err(ExecutionError::ProcessExit { code });
            }
            ProcessCompletion::TimedOut => return Err(ExecutionError::ProcessTimeout),
            ProcessCompletion::Cancelled => return Err(ExecutionError::ProcessCancelled),
        }

        let decoded = decode_provider_stdout(transcript.stdout())
            .map_err(|source| ExecutionError::ProcessProtocol { source })?;
        validate_decoded_process_transcript(invocation, decoded, event_sink)
    }
}

fn validate_process_authority_preflight(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    subject_lock: &ExecutionSubjectLock,
    subjects: &MatchedExecutionSubjects,
    authority: &AuthorizedProcess,
) -> Result<(), ExecutionError> {
    authority
        .matches_context(resolved, invocation, subject_lock, subjects)
        .map_err(|message| ExecutionError::Preflight { message })
}

fn validate_process_subject_preflight(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    subject_lock: &ExecutionSubjectLock,
    subjects: &MatchedExecutionSubjects,
) -> Result<(), ExecutionError> {
    subjects
        .matches_context(resolved, invocation, subject_lock)
        .map_err(|message| ExecutionError::Preflight { message })
}

fn validate_in_process_preflight(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    port: &PortIdentity,
) -> Result<(), ExecutionError> {
    validate_common_preflight(resolved, invocation, ExecutionModeKind::InProcess)?;
    if resolved.trust() != Trust::Trusted {
        return preflight("the injected in-process seam requires trusted operator policy");
    }
    if port.extension_id != resolved.extension_id()
        || port.version != resolved.version()
        || port.publisher_id != resolved.publisher_id()
        || port.integrity != resolved.integrity().value
        || port.execution_mode_name != resolved.execution_mode().name
        || port.execution_mode_kind != resolved.execution_mode().kind
    {
        return preflight("the injected port identity or execution mode does not match resolution");
    }
    Ok(())
}

fn validate_common_preflight(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    expected_mode: ExecutionModeKind,
) -> Result<(), ExecutionError> {
    if resolved.execution_mode().kind != expected_mode {
        return preflight(format!(
            "the resolved execution mode is not {expected_mode:?}"
        ));
    }
    if invocation.phase != InvocationPhase::Execute {
        return preflight("the execution seam requires invocation phase execute");
    }
    if invocation.extension.extension_id != resolved.extension_id()
        || invocation.extension.version != resolved.version()
        || invocation.extension.publisher_id != resolved.publisher_id()
        || invocation.extension.integrity != resolved.integrity().value
    {
        return preflight("the invocation extension identity does not match resolution");
    }
    if invocation.capability_id != resolved.capability().capability_id {
        return preflight("the invocation capability does not match resolution");
    }
    if invocation.interface.name != resolved.execution_mode().name
        || invocation.interface.kind != resolved.execution_mode().kind
        || invocation.interface.protocol != resolved.execution_mode().protocol
    {
        return preflight("the invocation interface does not match resolution");
    }
    if invocation.authorization.lock_id != resolved.lock_id() {
        return preflight("the invocation authorization does not reference the resolved lock");
    }
    if invocation.configuration.schema_id != resolved.capability().configuration_schema {
        return preflight("the invocation configuration schema does not match the capability");
    }
    if invocation.expected_output_types != resolved.capability().produces {
        return preflight("the invocation expected outputs do not match the capability");
    }
    if invocation.limits != resolved.execution_mode().limits {
        return preflight("the invocation limits do not match the resolved execution mode");
    }
    Ok(())
}

fn validate_captured_length(
    stream: ProcessStream,
    limit: u64,
    observed: usize,
) -> Result<(), ExecutionError> {
    let observed = u64::try_from(observed).unwrap_or(u64::MAX);
    if observed > limit {
        return Err(ExecutionError::ProcessOutputLimit {
            stream,
            limit,
            observed,
        });
    }
    Ok(())
}

fn validate_decoded_process_transcript(
    invocation: &ExtensionInvocation,
    decoded: DecodedProcessTranscript,
    event_sink: &mut dyn EventSink,
) -> Result<ValidatedExecution, ExecutionError> {
    let (raw_events, result) = decoded.into_parts();
    let mut validating_sink = ValidatingEventSink::new(invocation, event_sink);
    for event in &raw_events {
        let _emission_result = validating_sink.emit(event);
    }
    let (events, issue) = validating_sink.finish();
    match issue {
        Some(EventIssue::Invalid(message)) => {
            return Err(ExecutionError::InvalidEvent {
                message,
                events,
                result: Some(Box::new(result)),
            });
        }
        Some(EventIssue::Downstream(source)) => {
            return Err(ExecutionError::EventSink {
                source,
                events,
                result: Some(Box::new(result)),
            });
        }
        None => {}
    }

    validate_result(invocation, &events, &result).map_err(|message| {
        ExecutionError::InvalidResult {
            message,
            events: events.clone(),
            result: Box::new(result.clone()),
        }
    })?;
    Ok(ValidatedExecution { events, result })
}

fn validate_result(
    invocation: &ExtensionInvocation,
    events: &[ExtensionEvent],
    result: &ExtensionResult,
) -> Result<(), String> {
    result.validate().map_err(|error| error.to_string())?;
    if result
        .diagnostics
        .iter()
        .any(|diagnostic| !diagnostic.redacted)
    {
        return Err("result contains an unredacted diagnostic".to_owned());
    }
    if result.run_id != invocation.run_id {
        return Err("result run_id does not match the invocation".to_owned());
    }
    if result.invocation_id != invocation.invocation_id {
        return Err("result invocation_id does not match the invocation".to_owned());
    }
    if result.extension_id != invocation.extension.extension_id
        || result.extension_version != invocation.extension.version
        || result.extension_integrity != invocation.extension.integrity
    {
        return Err("result extension identity does not match the invocation".to_owned());
    }
    if result.capability_id != invocation.capability_id {
        return Err("result capability_id does not match the invocation".to_owned());
    }
    if result.configuration_digest != invocation.configuration.digest {
        return Err("result configuration digest does not match the invocation".to_owned());
    }
    if result.authorization_id != invocation.authorization.authorization_id {
        return Err("result authorization_id does not match the invocation".to_owned());
    }
    let input_artifact_ids: HashSet<&str> = invocation
        .input_artifacts
        .iter()
        .map(|artifact| artifact.artifact_id.as_str())
        .collect();
    if result
        .consumed_artifacts
        .iter()
        .any(|artifact_id| !input_artifact_ids.contains(artifact_id.as_str()))
    {
        return Err("result consumed_artifacts contains an undeclared input identifier".to_owned());
    }
    let produced_artifact_ids: HashSet<&str> = result
        .produced_artifacts
        .iter()
        .map(String::as_str)
        .collect();
    if events
        .iter()
        .filter(|event| event.kind == EventKind::ArtifactProduced)
        .flat_map(|event| &event.artifact_refs)
        .any(|artifact_id| !produced_artifact_ids.contains(artifact_id.as_str()))
    {
        return Err(
            "artifact-produced event references an identifier absent from the result".to_owned(),
        );
    }
    let terminal = events
        .last()
        .ok_or_else(|| "provider emitted no terminal event".to_owned())?;
    let expected_kind = match result.outcome {
        Outcome::Produced | Outcome::Reused | Outcome::Skipped => EventKind::PhaseCompleted,
        Outcome::Blocked | Outcome::Unavailable | Outcome::Failed => EventKind::PhaseFailed,
        Outcome::Cancelled => EventKind::Cancelled,
    };
    if terminal.kind != expected_kind || terminal.state != result.outcome.event_state() {
        return Err("terminal event kind/state does not match the result outcome".to_owned());
    }
    Ok(())
}

fn preflight<T>(message: impl Into<String>) -> Result<T, ExecutionError> {
    Err(ExecutionError::Preflight {
        message: message.into(),
    })
}

#[derive(Debug)]
enum EventIssue {
    Invalid(String),
    Downstream(EventSinkError),
}

struct ValidatingEventSink<'a> {
    invocation: &'a ExtensionInvocation,
    downstream: &'a mut dyn EventSink,
    raw_events: Vec<ExtensionEvent>,
    event_ids: HashSet<String>,
    last_sequence: Option<u64>,
    terminal_seen: bool,
    issue: Option<EventIssue>,
}

impl<'a> ValidatingEventSink<'a> {
    fn new(invocation: &'a ExtensionInvocation, downstream: &'a mut dyn EventSink) -> Self {
        Self {
            invocation,
            downstream,
            raw_events: Vec::new(),
            event_ids: HashSet::new(),
            last_sequence: None,
            terminal_seen: false,
            issue: None,
        }
    }

    fn finish(self) -> (Vec<ExtensionEvent>, Option<EventIssue>) {
        (self.raw_events, self.issue)
    }

    fn reject(&mut self, message: impl Into<String>) -> Result<(), EventSinkError> {
        let message = message.into();
        if self.issue.is_none() {
            self.issue = Some(EventIssue::Invalid(message.clone()));
        }
        Err(EventSinkError::new(message))
    }
}

impl EventSink for ValidatingEventSink<'_> {
    fn emit(&mut self, event: &ExtensionEvent) -> Result<(), EventSinkError> {
        self.raw_events.push(event.clone());
        if self.issue.is_some() {
            return Err(EventSinkError::new(
                "an earlier event was rejected for this invocation",
            ));
        }
        if let Err(error) = event.validate() {
            return self.reject(error.to_string());
        }
        if event
            .diagnostics
            .iter()
            .any(|diagnostic| !diagnostic.redacted)
        {
            return self.reject("event contains an unredacted diagnostic");
        }
        if event.run_id != self.invocation.run_id
            || event.invocation_id != self.invocation.invocation_id
        {
            return self.reject("event run/invocation identity does not match the invocation");
        }
        if event.phase != self.invocation.phase.lifecycle_point() {
            return self.reject("event phase does not match the invocation phase");
        }
        if self.terminal_seen {
            return self.reject("provider emitted an event after a terminal event");
        }
        if !self.event_ids.insert(event.event_id.clone()) {
            return self.reject("provider emitted a duplicate event_id");
        }
        if self
            .last_sequence
            .is_some_and(|previous| event.sequence <= previous)
        {
            return self.reject(
                "event sequences must be unique and strictly increasing; gaps are allowed",
            );
        }
        self.last_sequence = Some(event.sequence);
        self.terminal_seen = matches!(
            event.kind,
            EventKind::PhaseCompleted | EventKind::PhaseFailed | EventKind::Cancelled
        );
        if let Err(source) = self.downstream.emit(event) {
            self.issue = Some(EventIssue::Downstream(source.clone()));
            return Err(source);
        }
        Ok(())
    }
}
