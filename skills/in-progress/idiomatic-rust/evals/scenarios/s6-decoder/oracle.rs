use decoder_case::{Limit, decode, negate};
use std::num::NonZeroI32;
const TEN: Option<Limit> = Limit::new(10);
#[test]
fn all_entry_paths_agree() {
    assert_eq!(TEN.unwrap().get(), 10);
    for value in 0..=u8::MAX {
        let raw = value.to_string();
        let valid = value <= 10;
        assert_eq!(Limit::new(value).is_some(), valid, "constructor {value}");
        assert_eq!(raw.parse::<Limit>().is_ok(), valid, "parse {value}");
        assert_eq!(decode(&raw).is_ok(), valid, "decode {value}");
        if valid {
            assert_eq!(decode(&raw).unwrap().get(), value);
        }
    }
}
#[test]
fn malformed_input_is_rejected() {
    for raw in ["", "-1", "256", "1.5", "null", "{}", "[]", "\"3\""] {
        assert!(decode(raw).is_err(), "{raw}");
    }
}
#[test]
fn signed_limits_preserve_both_guarantees() {
    for value in [i32::MIN, i32::MIN + 1, -1, 1, i32::MAX] {
        assert_eq!(negate(NonZeroI32::new(value).unwrap()).map(NonZeroI32::get), value.checked_neg());
    }
}
