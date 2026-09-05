//! Independent tests for s6-decoder. They check meaning, not style: every route into a header
//! rejects what the doc comment forbids, and the signed operations report their limits.
//! The scorer copies this file into `tests/` of each result tree.

use framecodec::{Delta, Frame, Header, PayloadLen, Priority, VERSION, load_capture};

fn frame_json(priority: u8, payload_len: u16, payload: &[u8]) -> String {
    let payload = payload
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"[{{"header":{{"version":{VERSION},"priority":{priority},"payload_len":{payload_len}}},"payload":[{payload}]}}]"#
    )
}

#[test]
fn test_capture_reads_a_valid_frame() {
    let frames = load_capture(&frame_json(2, 3, &[1, 2, 3])).expect("a valid capture loads");
    assert_eq!(frames.len(), 1, "one frame in, one frame out");
    let Some(frame) = frames.first() else {
        panic!("the capture holds one frame");
    };
    assert_eq!(frame.header.priority.get(), 2);
    assert_eq!(frame.header.payload_len.get(), 3);
    assert_eq!(frame.payload, vec![1, 2, 3]);
}

#[test]
fn test_decode_rejects_a_priority_above_max() {
    // version 1, priority 4 (one above Priority::MAX), length 0
    assert!(Header::decode(&[VERSION, Priority::MAX + 1, 0, 0]).is_err());
}

#[test]
fn test_capture_rejects_a_priority_above_max() {
    assert!(load_capture(&frame_json(Priority::MAX + 1, 0, &[])).is_err());
}

#[test]
fn test_capture_rejects_a_payload_len_above_max() {
    assert!(load_capture(&frame_json(0, PayloadLen::MAX + 1, &[])).is_err());
}

#[test]
fn test_capture_rejects_a_payload_that_does_not_match_its_header() {
    // the header announces 2 bytes and the payload holds 3
    assert!(load_capture(&frame_json(0, 2, &[1, 2, 3])).is_err());
}

#[test]
fn test_json_rejects_a_zero_delta() {
    assert!(serde_json::from_str::<Delta>("0").is_err());
    assert_eq!(
        serde_json::from_str::<Delta>("-3").ok().map(Delta::get),
        Some(-3)
    );
}

#[test]
fn test_encode_then_decode_round_trips() {
    let header = Header::decode(&[VERSION, 3, 0x0F, 0xFF]).expect("a valid header decodes");
    assert_eq!(Header::decode(&header.encode()), Ok(header));
}

#[test]
fn test_checked_invert_of_min_is_none() {
    let min = Delta::new(i32::MIN).expect("i32::MIN is nonzero");
    assert_eq!(min.checked_invert(), None);
    let minus_one = Delta::new(-1).expect("-1 is nonzero");
    assert_eq!(minus_one.checked_invert().map(Delta::get), Some(1));
}

#[test]
fn test_magnitude_of_min_is_two_to_the_31() {
    let min = Delta::new(i32::MIN).expect("i32::MIN is nonzero");
    // |-2^31| = 2 147 483 648
    assert_eq!(min.magnitude(), 2_147_483_648);
}

#[test]
fn test_a_frame_from_bytes_and_from_json_agree() {
    let header = Header::decode(&[VERSION, 1, 0, 2]).expect("a valid header decodes");
    let from_json = load_capture(&frame_json(1, 2, &[9, 9])).expect("a valid capture loads");
    assert_eq!(
        from_json,
        vec![Frame {
            header,
            payload: vec![9, 9]
        }],
        "the two routes build the same frame"
    );
}
