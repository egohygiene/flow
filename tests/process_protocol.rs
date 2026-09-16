use flow::process::{
    ProcessProtocolError, decode_provider_stdout, encode_invocation_frame,
};
use flow::{ExtensionEvent, ExtensionInvocation, ExtensionResult};
use serde::Serialize;
use serde_json::json;

fn invocation() -> ExtensionInvocation {
    serde_json::from_str(include_str!(
        "../contracts/examples/extension-invocation.v1.example.json"
    ))
    .expect("checked-in invocation must deserialize")
}

fn event() -> ExtensionEvent {
    serde_json::from_str(include_str!(
        "../contracts/examples/extension-event.v1.example.json"
    ))
    .expect("checked-in event must deserialize")
}

fn result() -> ExtensionResult {
    serde_json::from_str(include_str!(
        "../contracts/examples/extension-result.v1.example.json"
    ))
    .expect("checked-in result must deserialize")
}

fn line(value: &impl Serialize) -> Vec<u8> {
    let mut encoded = serde_json::to_vec(value).expect("fixture must serialize");
    encoded.push(b'\n');
    encoded
}

fn transcript(frames: &[Vec<u8>]) -> Vec<u8> {
    frames.iter().flatten().copied().collect()
}

#[test]
fn invocation_encoding_is_deterministic_compact_json_with_one_lf() {
    let mut invocation = invocation();
    invocation.configuration.values.insert(
        "multiline".to_owned(),
        json!("first line\nsecond line"),
    );

    let first = encode_invocation_frame(&invocation).unwrap();
    let second = encode_invocation_frame(&invocation).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.last(), Some(&b'\n'));
    assert_eq!(first.iter().filter(|byte| **byte == b'\n').count(), 1);
    assert!(!first[..first.len() - 1].contains(&b'\r'));
    assert_eq!(
        serde_json::from_slice::<ExtensionInvocation>(&first[..first.len() - 1]).unwrap(),
        invocation
    );
}

#[test]
fn provider_transcript_decodes_lf_and_crlf_frames() {
    let expected_event = event();
    let expected_result = result();
    let lf = transcript(&[line(&expected_event), line(&expected_result)]);
    let crlf = lf
        .iter()
        .flat_map(|byte| {
            if *byte == b'\n' {
                vec![b'\r', b'\n']
            } else {
                vec![*byte]
            }
        })
        .collect::<Vec<_>>();

    for encoded in [&lf, &crlf] {
        let decoded = decode_provider_stdout(encoded).unwrap();
        assert_eq!(decoded.events(), std::slice::from_ref(&expected_event));
        assert_eq!(decoded.result(), &expected_result);

        let (events, decoded_result) = decoded.into_parts();
        assert_eq!(events, [expected_event.clone()]);
        assert_eq!(decoded_result, expected_result);
    }
}

#[test]
fn caller_chunk_boundaries_do_not_affect_closed_transcript_decoding() {
    let encoded = transcript(&[line(&event()), line(&result())]);
    let expected = decode_provider_stdout(&encoded).unwrap();

    for chunk_size in [1, 2, 3, 7, 64, encoded.len()] {
        let captured = encoded
            .chunks(chunk_size)
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(decode_provider_stdout(&captured).unwrap(), expected);
    }
}

#[test]
fn terminal_result_without_events_is_a_closed_framing_transcript() {
    let expected = result();
    let decoded = decode_provider_stdout(&line(&expected)).unwrap();

    assert!(decoded.events().is_empty());
    assert_eq!(decoded.result(), &expected);
}

#[test]
fn blank_frame_is_rejected() {
    assert_eq!(
        decode_provider_stdout(b"\n").unwrap_err(),
        ProcessProtocolError::BlankFrame { frame_number: 1 }
    );
    assert_eq!(
        decode_provider_stdout(b"\r\n").unwrap_err(),
        ProcessProtocolError::BlankFrame { frame_number: 1 }
    );
}

#[test]
fn unterminated_final_frame_is_rejected_without_implicit_eof_completion() {
    let encoded = serde_json::to_vec(&result()).unwrap();

    assert_eq!(
        decode_provider_stdout(&encoded).unwrap_err(),
        ProcessProtocolError::PartialFrame {
            frame_number: 1,
            byte_count: encoded.len(),
        }
    );

    let first = line(&event());
    let mut after_event = first.clone();
    after_event.extend_from_slice(&encoded);
    assert_eq!(
        decode_provider_stdout(&after_event).unwrap_err(),
        ProcessProtocolError::PartialFrame {
            frame_number: 2,
            byte_count: encoded.len(),
        }
    );
}

#[test]
fn invalid_utf8_frame_is_rejected() {
    assert_eq!(
        decode_provider_stdout(&[0xff, b'\n']).unwrap_err(),
        ProcessProtocolError::InvalidUtf8 { frame_number: 1 }
    );
}

#[test]
fn malformed_json_frame_is_rejected() {
    let error = decode_provider_stdout(b"{not-json}\n").unwrap_err();

    assert!(matches!(
        error,
        ProcessProtocolError::MalformedJson {
            frame_number: 1,
            ..
        }
    ));
}

#[test]
fn byte_order_mark_and_pretty_printed_multiline_json_are_rejected() {
    let mut with_bom = vec![0xef, 0xbb, 0xbf];
    with_bom.extend(line(&result()));
    assert!(matches!(
        decode_provider_stdout(&with_bom),
        Err(ProcessProtocolError::MalformedJson {
            frame_number: 1,
            ..
        })
    ));

    let mut multiline = serde_json::to_vec_pretty(&result()).unwrap();
    multiline.push(b'\n');
    assert!(matches!(
        decode_provider_stdout(&multiline),
        Err(ProcessProtocolError::MalformedJson {
            frame_number: 1,
            ..
        })
    ));
}

#[test]
fn missing_or_non_string_schema_version_is_rejected() {
    assert_eq!(
        decode_provider_stdout(b"{}\n").unwrap_err(),
        ProcessProtocolError::MissingSchemaVersion { frame_number: 1 }
    );
    assert_eq!(
        decode_provider_stdout(b"{\"schema_version\":1}\n").unwrap_err(),
        ProcessProtocolError::InvalidSchemaVersion { frame_number: 1 }
    );
}

#[test]
fn unknown_schema_version_is_rejected_without_downgrade() {
    let error = decode_provider_stdout(b"{\"schema_version\":\"flow.extension-event/v2\"}\n")
        .unwrap_err();

    assert_eq!(
        error,
        ProcessProtocolError::UnknownSchemaVersion {
            frame_number: 1,
            schema_version: "flow.extension-event/v2".to_owned(),
        }
    );
    assert!(!error.to_string().contains("flow.extension-event/v2"));
}

#[test]
fn invocation_frame_on_provider_stdout_is_rejected() {
    assert_eq!(
        decode_provider_stdout(&encode_invocation_frame(&invocation()).unwrap()).unwrap_err(),
        ProcessProtocolError::InvocationOnStdout { frame_number: 1 }
    );
}

#[test]
fn empty_or_event_only_stdout_is_missing_a_terminal_result() {
    assert_eq!(
        decode_provider_stdout(&[]).unwrap_err(),
        ProcessProtocolError::MissingResult
    );
    assert_eq!(
        decode_provider_stdout(&line(&event())).unwrap_err(),
        ProcessProtocolError::MissingResult
    );
}

#[test]
fn duplicate_terminal_result_is_rejected() {
    let terminal = line(&result());
    let encoded = transcript(&[terminal.clone(), terminal]);

    assert_eq!(
        decode_provider_stdout(&encoded).unwrap_err(),
        ProcessProtocolError::DuplicateResult { frame_number: 2 }
    );
}

#[test]
fn event_after_terminal_result_is_rejected() {
    let encoded = transcript(&[line(&result()), line(&event())]);

    assert_eq!(
        decode_provider_stdout(&encoded).unwrap_err(),
        ProcessProtocolError::MessageAfterResult {
            frame_number: 2,
            schema_version: "flow.extension-event/v1".to_owned(),
        }
    );
}

#[test]
fn typed_frames_remain_closed_to_unknown_fields() {
    let mut value = serde_json::to_value(result()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_owned(), json!(true));
    let error = decode_provider_stdout(&line(&value)).unwrap_err();

    assert!(matches!(
        &error,
        ProcessProtocolError::MalformedJson {
            frame_number: 1,
            ..
        }
    ));
    assert!(!error.to_string().contains("unexpected"));
}
