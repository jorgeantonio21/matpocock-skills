//! A codec for a small framed protocol.
//!
//! 1. A frame on the wire is a fixed header followed by the payload bytes.
//! 2. The capture tool writes frames as JSON, so the header types also derive serde.
//! 3. Every route that builds a header rejects what the doc comment on `Header` forbids.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The one protocol version this crate reads.
pub const VERSION: u8 = 1;

/// A frame priority from `0` (lowest) to `Priority::MAX`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
pub struct Priority(u8);

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

/// A payload length from `0` to `PayloadLen::MAX` bytes.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
pub struct PayloadLen(u16);

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

/// A signed, nonzero adjustment a frame applies to the receiver's window.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
pub struct Delta(i32);

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

    /// The opposite delta.
    #[must_use]
    pub const fn invert(self) -> Self {
        Self(-self.0)
    }
}

/// A decoded frame header. Whichever route builds one, a header never carries an unsupported
/// version, a priority above `Priority::MAX`, or a payload length above `PayloadLen::MAX`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
pub struct Header {
    /// The protocol version. Always `VERSION`.
    pub version: u8,
    /// The frame priority.
    pub priority: Priority,
    /// The payload length that follows the header.
    pub payload_len: PayloadLen,
}

/// A frame: a header and the payload bytes it announces.
#[derive(Clone, PartialEq, Eq, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
pub struct Frame {
    /// The header.
    pub header: Header,
    /// The payload. Its length is `header.payload_len`.
    pub payload: Vec<u8>,
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
    /// The length field is above `PayloadLen::MAX`.
    #[error("a payload of {0} bytes is above the limit of {max}", max = PayloadLen::MAX)]
    PayloadTooLong(u16),
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
        if version != VERSION {
            return Err(DecodeError::UnsupportedVersion(version));
        }
        let len = u16::from_be_bytes([len_hi, len_lo]);
        let payload_len = PayloadLen::new(len).ok_or(DecodeError::PayloadTooLong(len))?;
        Ok(Self {
            version,
            priority: Priority(priority),
            payload_len,
        })
    }
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
    fn test_decode_rejects_a_payload_above_the_limit() {
        // 0x1001 = 4097, one above PayloadLen::MAX
        assert_eq!(
            Header::decode(&[1, 0, 0x10, 0x01]),
            Err(DecodeError::PayloadTooLong(4097))
        );
    }

    #[test]
    fn test_priority_and_len_constructors_hold_their_bounds() {
        assert!(Priority::new(3).is_some());
        assert!(Priority::new(4).is_none());
        assert!(PayloadLen::new(4096).is_some());
        assert!(PayloadLen::new(4097).is_none());
        assert!(Delta::new(0).is_none());
    }

    #[test]
    fn test_invert_flips_the_sign() -> anyhow::Result<()> {
        let delta = Delta::new(5).ok_or_else(|| anyhow::anyhow!("5 is nonzero"))?;
        assert_eq!(delta.invert().get(), -5);
        Ok(())
    }
}
