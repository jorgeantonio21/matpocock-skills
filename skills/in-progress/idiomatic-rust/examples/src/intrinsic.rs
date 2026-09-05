//! An intrinsic value invariant: a concurrency limit is never zero.
//!
//! 1. The representation carries the invariant. `NonZeroU32` has no zero value.
//! 2. Every route in goes through `new`: `TryFrom`, `FromStr`, and `Deserialize`.
//! 3. An operation returns `Self` only when it preserves the invariant. `halve` stops at one.

use std::num::{NonZeroU32, ParseIntError};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The number of jobs a pool runs at once. Never zero: a zero limit blocks every job forever.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "u32", into = "u32")]
pub struct Concurrency(NonZeroU32);

/// A concurrency limit of zero was given.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
#[error("a concurrency limit of zero blocks every job")]
pub struct ZeroConcurrency;

/// A string did not parse as a concurrency limit.
#[derive(Debug, Error)]
pub enum ParseConcurrencyError {
    /// The string is not a number.
    #[error("not a number")]
    NotANumber(#[from] ParseIntError),
    /// The number is zero.
    #[error(transparent)]
    Zero(#[from] ZeroConcurrency),
}

impl Concurrency {
    /// The smallest limit a pool accepts.
    pub const ONE: Self = Self(NonZeroU32::MIN);

    /// Returns the limit, or `None` when `value` is zero.
    #[must_use]
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// The limit as a plain number, for a log line or a semaphore.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }

    /// Half the limit, rounded down and never below one. The invariant survives the operation.
    #[must_use]
    pub const fn halve(self) -> Self {
        match NonZeroU32::new(self.0.get() / 2) {
            Some(half) => Self(half),
            None => Self::ONE,
        }
    }
}

impl TryFrom<u32> for Concurrency {
    type Error = ZeroConcurrency;

    fn try_from(value: u32) -> Result<Self, ZeroConcurrency> {
        Self::new(value).ok_or(ZeroConcurrency)
    }
}

impl From<Concurrency> for u32 {
    fn from(limit: Concurrency) -> Self {
        limit.get()
    }
}

impl FromStr for Concurrency {
    type Err = ParseConcurrencyError;

    fn from_str(text: &str) -> Result<Self, ParseConcurrencyError> {
        let value: u32 = text.parse()?;
        Ok(Self::try_from(value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_is_rejected_on_every_route() {
        assert!(Concurrency::new(0).is_none());
        assert!("0".parse::<Concurrency>().is_err());
        assert!(serde_json::from_str::<Concurrency>("0").is_err());
    }

    #[test]
    fn test_every_route_accepts_the_same_value() -> anyhow::Result<()> {
        let from_new = Concurrency::new(4).ok_or(ZeroConcurrency)?;
        assert_eq!("4".parse::<Concurrency>()?, from_new);
        assert_eq!(serde_json::from_str::<Concurrency>("4")?, from_new);
        assert_eq!(serde_json::to_string(&from_new)?, "4");
        Ok(())
    }

    #[test]
    fn test_halve_stops_at_one() {
        // 5 / 2 = 2; 1 / 2 = 0, which the invariant forbids, so the result is 1
        assert_eq!(Concurrency::ONE.halve(), Concurrency::ONE);
        assert_eq!(
            Concurrency::new(5).map(Concurrency::halve),
            Concurrency::new(2)
        );
    }
}
