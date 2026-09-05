//! A raw-to-validated boundary: parsing structure and validating meaning are two steps.
//!
//! 1. `RawHeader` is what the input says. serde checks the structure: the field names and types.
//! 2. `Header::try_from` checks the meaning: a supported version and a payload that fits a frame.
//! 3. `Header` derives `Deserialize` through `RawHeader`, so the JSON route cannot skip step 2.
//!    The byte route builds a `RawHeader` and takes the same step.

use serde::Deserialize;
use thiserror::Error;

/// A header as decoded, before any meaning is checked.
#[derive(Copy, Clone, PartialEq, Eq, Debug)] // std
#[derive(Deserialize)] // serde
pub struct RawHeader {
    /// The protocol version the sender wrote.
    pub version: u8,
    /// The payload length the sender wrote.
    pub payload_len: u32,
}

/// A header the decoder accepted: a supported version and a payload that fits in a frame.
#[derive(Copy, Clone, PartialEq, Eq, Debug)] // std
#[derive(Deserialize)] // serde
#[serde(try_from = "RawHeader")]
pub struct Header {
    payload_len: u32,
}

/// A structurally valid header with a meaning this decoder rejects.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
pub enum HeaderError {
    /// The version is not the one this decoder reads.
    #[error("version {0} is not supported; this decoder reads version {supported}", supported = Header::VERSION)]
    UnsupportedVersion(u8),
    /// The payload does not fit in a frame.
    #[error("a payload of {len} bytes is above the frame limit of {max}")]
    PayloadTooLong {
        /// The length the sender wrote.
        len: u32,
        /// The largest payload a frame holds.
        max: u32,
    },
}

impl Header {
    /// The one protocol version this decoder reads.
    pub const VERSION: u8 = 2;
    /// The largest payload a frame holds.
    pub const MAX_PAYLOAD: u32 = 64 * 1024;
    /// The encoded size: the version byte, then the length in big-endian.
    pub const SIZE: usize = 5;

    /// Decodes the fixed prefix of a frame.
    ///
    /// # Errors
    ///
    /// The same errors as [`Header::try_from`]: the bytes are structurally a header, and the
    /// meaning is checked in the one place.
    pub fn from_bytes(bytes: [u8; Self::SIZE]) -> Result<Self, HeaderError> {
        let [version, len @ ..] = bytes;
        Self::try_from(RawHeader {
            version,
            payload_len: u32::from_be_bytes(len),
        })
    }

    /// The payload length. At most [`Header::MAX_PAYLOAD`].
    #[must_use]
    pub const fn payload_len(self) -> u32 {
        self.payload_len
    }
}

impl TryFrom<RawHeader> for Header {
    type Error = HeaderError;

    fn try_from(raw: RawHeader) -> Result<Self, HeaderError> {
        if raw.version != Self::VERSION {
            return Err(HeaderError::UnsupportedVersion(raw.version));
        }
        if raw.payload_len > Self::MAX_PAYLOAD {
            return Err(HeaderError::PayloadTooLong {
                len: raw.payload_len,
                max: Self::MAX_PAYLOAD,
            });
        }
        Ok(Self {
            payload_len: raw.payload_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_route_rejects_a_meaning_the_structure_allows() {
        // the JSON is a well-formed RawHeader; version 9 is the semantic rejection
        let json = r#"{"version":9,"payload_len":10}"#;
        assert!(serde_json::from_str::<Header>(json).is_err());
        assert!(serde_json::from_str::<RawHeader>(json).is_ok());
    }

    #[test]
    fn test_byte_route_takes_the_same_check() {
        // 0x0001_0001 = 65 537, one above MAX_PAYLOAD
        let too_long = [Header::VERSION, 0x00, 0x01, 0x00, 0x01];
        assert_eq!(
            Header::from_bytes(too_long),
            Err(HeaderError::PayloadTooLong {
                len: 65_537,
                max: Header::MAX_PAYLOAD
            })
        );
        assert_eq!(
            Header::from_bytes([9, 0, 0, 0, 1]),
            Err(HeaderError::UnsupportedVersion(9))
        );
    }

    #[test]
    fn test_both_routes_accept_the_same_header() -> anyhow::Result<()> {
        let from_bytes = Header::from_bytes([Header::VERSION, 0, 0, 1, 0])?;
        let from_json = serde_json::from_str::<Header>(r#"{"version":2,"payload_len":256}"#)?;
        assert_eq!(from_bytes, from_json);
        assert_eq!(from_bytes.payload_len(), 256);
        Ok(())
    }
}
