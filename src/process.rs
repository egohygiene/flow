//! Host-neutral framing for the external-provider standard streams.
//!
//! Flow writes one compact [`ExtensionInvocation`] JSON object followed by a
//! line feed. A provider writes zero or more [`ExtensionEvent`] JSON objects,
//! followed by exactly one [`ExtensionResult`] JSON object. Every provider
//! frame must be terminated by LF; CRLF is also accepted when decoding.
//!
//! This module owns only byte framing. It does not launch processes, inspect
//! stderr, enforce resource limits, or perform semantic event/result
//! validation.

use std::str;

use serde_json::Value;
use thiserror::Error;

use crate::contracts::{
    EXTENSION_EVENT_V1, EXTENSION_INVOCATION_V1, EXTENSION_RESULT_V1, ExtensionEvent,
    ExtensionInvocation, ExtensionResult,
};

/// A syntactically complete provider transcript.
///
/// Construction is limited to [`decode_provider_stdout`]. The contained
/// events and result still require Flow-owned semantic and correlation
/// validation before they can become accepted execution evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedProcessTranscript {
    events: Vec<ExtensionEvent>,
    result: ExtensionResult,
}

impl DecodedProcessTranscript {
    #[must_use]
    pub fn events(&self) -> &[ExtensionEvent] {
        &self.events
    }

    #[must_use]
    pub const fn result(&self) -> &ExtensionResult {
        &self.result
    }

    #[must_use]
    pub fn into_parts(self) -> (Vec<ExtensionEvent>, ExtensionResult) {
        (self.events, self.result)
    }
}

/// A deterministic provider-protocol framing failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ProcessProtocolError {
    #[error("failed to serialize the invocation frame: {message}")]
    Serialization { message: String },
    #[error("provider stdout frame {frame_number} is blank")]
    BlankFrame { frame_number: usize },
    #[error(
        "provider stdout ended with partial frame {frame_number} ({byte_count} unterminated bytes)"
    )]
    PartialFrame {
        frame_number: usize,
        byte_count: usize,
    },
    #[error("provider stdout frame {frame_number} is not valid UTF-8")]
    InvalidUtf8 { frame_number: usize },
    #[error("provider stdout frame {frame_number} is malformed JSON")]
    MalformedJson {
        frame_number: usize,
        message: String,
    },
    #[error("provider stdout frame {frame_number} is missing schema_version")]
    MissingSchemaVersion { frame_number: usize },
    #[error("provider stdout frame {frame_number} has a non-string schema_version")]
    InvalidSchemaVersion { frame_number: usize },
    #[error("provider stdout frame {frame_number} uses an unsupported schema_version")]
    UnknownSchemaVersion {
        frame_number: usize,
        schema_version: String,
    },
    #[error("provider stdout frame {frame_number} is an invocation; invocations belong on stdin")]
    InvocationOnStdout { frame_number: usize },
    #[error("provider stdout contains no terminal result frame")]
    MissingResult,
    #[error("provider stdout frame {frame_number} is a duplicate terminal result")]
    DuplicateResult { frame_number: usize },
    #[error("provider stdout frame {frame_number} appears after the terminal result")]
    MessageAfterResult {
        frame_number: usize,
        schema_version: String,
    },
}

/// Encode one Flow-owned invocation as compact JSON followed by LF.
///
/// This encoder is deterministic for the typed invocation model and the
/// locked serialization dependencies. It is not a general canonical-JSON
/// implementation and its output must not be treated as a cryptographic
/// identity.
///
/// # Errors
///
/// Returns [`ProcessProtocolError::Serialization`] if JSON serialization
/// fails. This function does not perform semantic invocation validation.
pub fn encode_invocation_frame(
    invocation: &ExtensionInvocation,
) -> Result<Vec<u8>, ProcessProtocolError> {
    let mut encoded =
        serde_json::to_vec(invocation).map_err(|error| ProcessProtocolError::Serialization {
            message: error.to_string(),
        })?;
    encoded.push(b'\n');
    Ok(encoded)
}

/// Decode a closed provider stdout transcript.
///
/// Each frame must contain one existing extension-event or extension-result
/// contract. The terminal result is required, unique, and final. This function
/// validates framing and closed JSON shapes only; it deliberately does not
/// validate event ordering, invocation correlation, diagnostics, outcomes, or
/// artifacts.
///
/// # Errors
///
/// Returns a typed protocol error for invalid framing, encoding, JSON shape,
/// schema version, or transcript state.
pub fn decode_provider_stdout(
    stdout: &[u8],
) -> Result<DecodedProcessTranscript, ProcessProtocolError> {
    let mut events = Vec::new();
    let mut result = None;
    let mut cursor = 0;
    let mut frame_number = 1;

    while cursor < stdout.len() {
        let Some(relative_end) = stdout[cursor..].iter().position(|byte| *byte == b'\n') else {
            return Err(ProcessProtocolError::PartialFrame {
                frame_number,
                byte_count: stdout.len() - cursor,
            });
        };
        let frame_end = cursor + relative_end;
        let mut frame = &stdout[cursor..frame_end];
        cursor = frame_end + 1;

        if frame.last() == Some(&b'\r') {
            frame = &frame[..frame.len() - 1];
        }
        if frame.is_empty() {
            return Err(ProcessProtocolError::BlankFrame { frame_number });
        }

        match decode_provider_frame(frame, frame_number)? {
            ProviderFrame::Event(event) => {
                if result.is_some() {
                    return Err(ProcessProtocolError::MessageAfterResult {
                        frame_number,
                        schema_version: EXTENSION_EVENT_V1.to_owned(),
                    });
                }
                events.push(*event);
            }
            ProviderFrame::Result(decoded_result) => {
                if result.is_some() {
                    return Err(ProcessProtocolError::DuplicateResult { frame_number });
                }
                result = Some(decoded_result);
            }
        }

        frame_number += 1;
    }

    let result = result.ok_or(ProcessProtocolError::MissingResult)?;
    Ok(DecodedProcessTranscript {
        events,
        result: *result,
    })
}

enum ProviderFrame {
    Event(Box<ExtensionEvent>),
    Result(Box<ExtensionResult>),
}

fn decode_provider_frame(
    frame: &[u8],
    frame_number: usize,
) -> Result<ProviderFrame, ProcessProtocolError> {
    let frame =
        str::from_utf8(frame).map_err(|_| ProcessProtocolError::InvalidUtf8 { frame_number })?;
    let value: Value =
        serde_json::from_str(frame).map_err(|error| ProcessProtocolError::MalformedJson {
            frame_number,
            message: error.to_string(),
        })?;
    let schema_value = value
        .as_object()
        .and_then(|object| object.get("schema_version"))
        .ok_or(ProcessProtocolError::MissingSchemaVersion { frame_number })?;
    let schema_version = schema_value
        .as_str()
        .ok_or(ProcessProtocolError::InvalidSchemaVersion { frame_number })?;

    match schema_version {
        EXTENSION_EVENT_V1 => serde_json::from_str(frame)
            .map(Box::new)
            .map(ProviderFrame::Event)
            .map_err(|error| ProcessProtocolError::MalformedJson {
                frame_number,
                message: error.to_string(),
            }),
        EXTENSION_RESULT_V1 => serde_json::from_str(frame)
            .map(Box::new)
            .map(ProviderFrame::Result)
            .map_err(|error| ProcessProtocolError::MalformedJson {
                frame_number,
                message: error.to_string(),
            }),
        EXTENSION_INVOCATION_V1 => Err(ProcessProtocolError::InvocationOnStdout { frame_number }),
        _ => Err(ProcessProtocolError::UnknownSchemaVersion {
            frame_number,
            schema_version: schema_version.to_owned(),
        }),
    }
}
