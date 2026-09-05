//! A codec for a small framed protocol.
//!
//! 1. A frame on the wire is a fixed header followed by the payload bytes.
//! 2. The capture tool writes frames as JSON, so the header types also derive serde.
//! 3. Every route that builds a header rejects what the doc comment on `Header` forbids.
//!    The JSON route and the byte route share one check per type.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The one protocol version this crate reads.
pub const VERSION: u8 = 1;

/// A frame priority from `0` (lowest) to `Priority::MAX`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "u8", into = "u8")]
pub struct Priority(u8);

/// A priority above `Priority::MAX` was given.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
#[error("priority {0} is above the maximum {max}", max = Priority::MAX)]
pub struct PriorityTooHigh(pub u8);

impl Priority {
    /// The highest priority.
    pub const MAX: u8 = 3;

    /// Returns the priority, or `None` above `Priority::MAX`.
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level <= Self::MAX {
            Some(Self(level))
        } else {
            None
        }
    }

    /// The priority as a number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Priority {
    type Error = PriorityTooHigh;

    fn try_from(level: u8) -> Result<Self, PriorityTooHigh> {
        Self::new(level).ok_or(PriorityTooHigh(level))
    }
}

impl From<Priority> for u8 {
    fn from(priority: Priority) -> Self {
        priority.get()
    }
}

/// A payload length from `0` to `PayloadLen::MAX` bytes.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "u16", into = "u16")]
pub struct PayloadLen(u16);

/// A payload length above `PayloadLen::MAX` was given.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
#[error("a payload of {0} bytes is above the limit of {max}", max = PayloadLen::MAX)]
pub struct PayloadTooLong(pub u16);

impl PayloadLen {
    /// The largest payload a frame carries.
    pub const MAX: u16 = 4096;

    /// Returns the length, or `None` above `PayloadLen::MAX`.
    #[must_use]
    pub const fn new(len: u16) -> Option<Self> {
        if len <= Self::MAX {
            Some(Self(len))
        } else {
            None
        }
    }

    /// The length as a number.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PayloadLen {
    type Error = PayloadTooLong;

    fn try_from(len: u16) -> Result<Self, PayloadTooLong> {
        Self::new(len).ok_or(PayloadTooLong(len))
    }
}

impl From<PayloadLen> for u16 {
    fn from(len: PayloadLen) -> Self {
        len.get()
    }
}

/// A signed, nonzero adjustment a frame applies to the receiver's window.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "i32", into = "i32")]
pub struct Delta(i32);

/// A zero delta was given.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
#[error("a delta is never zero")]
pub struct ZeroDelta;

impl Delta {
    /// Returns the delta, or `None` for zero.
    #[must_use]
    pub const fn new(value: i32) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    /// The delta as a number.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }

    /// The unsigned distance. Nonzero, and `i32::MIN` fits: its magnitude is `2^31`.
    #[must_use]
    pub const fn magnitude(self) -> u32 {
        self.0.unsigned_abs()
    }

    /// The opposite delta, or `None` for `i32::MIN`, whose opposite does not fit in an `i32`.
    #[must_use]
    pub const fn checked_invert(self) -> Option<Self> {
        match self.0.checked_neg() {
            // The opposite of a nonzero value is nonzero.
            Some(inverted) => Some(Self(inverted)),
            None => None,
        }
    }
}

impl TryFrom<i32> for Delta {
    type Error = ZeroDelta;

    fn try_from(value: i32) -> Result<Self, ZeroDelta> {
        Self::new(value).ok_or(ZeroDelta)
    }
}

impl From<Delta> for i32 {
    fn from(delta: Delta) -> Self {
        delta.get()
    }
}

/// A decoded frame header. Whichever route builds one, a header never carries an unsupported
/// version, a priority above `Priority::MAX`, or a payload length above `PayloadLen::MAX`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "RawHeader")]
pub struct Header {
    /// The protocol version. Always `VERSION`.
    pub version: u8,
    /// The frame priority.
    pub priority: Priority,
    /// The payload length that follows the header.
    pub payload_len: PayloadLen,
}

/// The fields as the input wrote them. The meaning is checked in `Header::try_from`.
#[derive(Copy, Clone, Debug)] // std
#[derive(Deserialize)] // serde
struct RawHeader {
    version: u8,
    priority: Priority,
    payload_len: PayloadLen,
}

/// A frame: a header and the payload bytes it announces.
#[derive(Clone, PartialEq, Eq, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "RawFrame")]
pub struct Frame {
    /// The header.
    pub header: Header,
    /// The payload. Its length is `header.payload_len`.
    pub payload: Vec<u8>,
}

/// A frame as the input wrote it. The length match is checked in `Frame::try_from`.
#[derive(Clone, Debug)] // std
#[derive(Deserialize)] // serde
struct RawFrame {
    header: Header,
    payload: Vec<u8>,
}

/// Why bytes did not decode as a header.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Fewer than `Header::SIZE` bytes were given.
    #[error("a header is {expected} bytes, {actual} were given")]
    Truncated { expected: usize, actual: usize },
    /// The version byte is not `VERSION`.
    #[error("version {0} is not supported")]
    UnsupportedVersion(u8),
    /// The priority byte is above `Priority::MAX`.
    #[error(transparent)]
    PriorityTooHigh(#[from] PriorityTooHigh),
    /// The length field is above `PayloadLen::MAX`.
    #[error(transparent)]
    PayloadTooLong(#[from] PayloadTooLong),
}

/// Why a frame's header and payload do not go together.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
#[error("the header announces {announced} payload bytes and {actual} were given")]
pub struct PayloadMismatch {
    /// The header's `payload_len`.
    pub announced: u16,
    /// The payload's length.
    pub actual: usize,
}

/// Why a capture did not load.
#[derive(Debug, Error)]
pub enum CaptureError {
    /// The text is not a JSON array of frames, or a frame fails a header check.
    #[error("the capture is not a list of frames: {0}")]
    Json(#[from] serde_json::Error),
}

impl Header {
    /// The encoded size: the version, the priority, then the length in big-endian.
    pub const SIZE: usize = 4;

    /// Decodes a header from the first `Header::SIZE` bytes of `bytes`.
    ///
    /// # Errors
    ///
    /// See [`DecodeError`].
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let Some(&[version, priority, len_hi, len_lo]) = bytes.get(..Self::SIZE) else {
            return Err(DecodeError::Truncated {
                expected: Self::SIZE,
                actual: bytes.len(),
            });
        };
        let raw = RawHeader {
            version,
            priority: Priority::try_from(priority)?,
            payload_len: PayloadLen::try_from(u16::from_be_bytes([len_hi, len_lo]))?,
        };
        Self::try_from(raw)
    }

    /// Encodes the header. `decode(&encode())` returns the same header.
    #[must_use]
    pub fn encode(self) -> [u8; Self::SIZE] {
        let [len_hi, len_lo] = self.payload_len.get().to_be_bytes();
        [self.version, self.priority.get(), len_hi, len_lo]
    }
}

impl TryFrom<RawHeader> for Header {
    type Error = DecodeError;

    fn try_from(raw: RawHeader) -> Result<Self, DecodeError> {
        if raw.version != VERSION {
            return Err(DecodeError::UnsupportedVersion(raw.version));
        }
        Ok(Self {
            version: raw.version,
            priority: raw.priority,
            payload_len: raw.payload_len,
        })
    }
}

impl TryFrom<RawFrame> for Frame {
    type Error = PayloadMismatch;

    fn try_from(raw: RawFrame) -> Result<Self, PayloadMismatch> {
        let announced = raw.header.payload_len.get();
        if usize::from(announced) != raw.payload.len() {
            return Err(PayloadMismatch {
                announced,
                actual: raw.payload.len(),
            });
        }
        Ok(Self {
            header: raw.header,
            payload: raw.payload,
        })
    }
}

/// Loads the frames the capture tool wrote as a JSON array.
///
/// # Errors
///
/// See [`CaptureError`]. A frame that fails a header check, or whose payload does not match
/// its header, fails the whole capture.
pub fn load_capture(json: &str) -> Result<Vec<Frame>, CaptureError> {
    Ok(serde_json::from_str(json)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_reads_the_fields_in_order() -> anyhow::Result<()> {
        // version 1, priority 2, length 0x0100 = 256
        let header = Header::decode(&[1, 2, 0x01, 0x00])?;
        assert_eq!(header.version, VERSION);
        assert_eq!(header.priority.get(), 2);
        assert_eq!(header.payload_len.get(), 256);
        Ok(())
    }

    #[test]
    fn test_decode_rejects_a_short_input() {
        assert_eq!(
            Header::decode(&[1, 2, 0]),
            Err(DecodeError::Truncated {
                expected: Header::SIZE,
                actual: 3
            })
        );
    }

    #[test]
    fn test_decode_rejects_the_wrong_version() {
        assert_eq!(
            Header::decode(&[2, 0, 0, 0]),
            Err(DecodeError::UnsupportedVersion(2))
        );
    }

    #[test]
    fn test_decode_rejects_a_priority_above_max() {
        assert_eq!(
            Header::decode(&[1, 4, 0, 0]),
            Err(DecodeError::PriorityTooHigh(PriorityTooHigh(4)))
        );
    }

    #[test]
    fn test_decode_rejects_a_payload_above_the_limit() {
        // 0x1001 = 4097, one above PayloadLen::MAX
        assert_eq!(
            Header::decode(&[1, 0, 0x10, 0x01]),
            Err(DecodeError::PayloadTooLong(PayloadTooLong(4097)))
        );
    }

    #[test]
    fn test_encode_is_the_inverse_of_decode() -> anyhow::Result<()> {
        let bytes = [1, 3, 0x0F, 0xFF];
        assert_eq!(Header::decode(&bytes)?.encode(), bytes);
        Ok(())
    }

    #[test]
    fn test_json_routes_take_the_constructor_check() {
        assert!(serde_json::from_str::<Priority>("4").is_err());
        assert!(serde_json::from_str::<PayloadLen>("4097").is_err());
        assert!(serde_json::from_str::<Delta>("0").is_err());
    }

    #[test]
    fn test_capture_rejects_a_payload_that_does_not_match_its_header() {
        let json = r#"[{"header":{"version":1,"priority":0,"payload_len":2},"payload":[1,2,3]}]"#;
        assert!(load_capture(json).is_err());
    }

    #[test]
    fn test_capture_loads_a_valid_frame() -> anyhow::Result<()> {
        let json = r#"[{"header":{"version":1,"priority":2,"payload_len":3},"payload":[1,2,3]}]"#;
        let frames = load_capture(json)?;
        assert_eq!(frames.len(), 1);
        assert_eq!(
            frames.first().map(|frame| frame.payload.as_slice()),
            Some(&[1, 2, 3][..])
        );
        Ok(())
    }

    #[test]
    fn test_checked_invert_reports_the_one_overflow() -> anyhow::Result<()> {
        let min = Delta::new(i32::MIN).ok_or_else(|| anyhow::anyhow!("i32::MIN is nonzero"))?;
        assert_eq!(min.checked_invert(), None);
        // |i32::MIN| = 2^31
        assert_eq!(min.magnitude(), 2_147_483_648);
        let five = Delta::new(5).ok_or_else(|| anyhow::anyhow!("5 is nonzero"))?;
        assert_eq!(five.checked_invert().map(Delta::get), Some(-5));
        Ok(())
    }
}
