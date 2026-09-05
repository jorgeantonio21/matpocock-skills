//! A persistent representation change: version 2 makes `workers` stricter, and version 1 files still load.
//!
//! 1. Version 1 stored `workers: 0` to mean "one per core". Version 2 writes `"auto"` and rejects zero.
//! 2. `load` reads both. A version 1 zero becomes `Workers::Auto`: the meaning is kept, not rounded to one.
//! 3. The tests hold the historical bytes, the migrated meaning, the stricter rejection, and a corrupt input.

use std::num::NonZeroU32;

use serde::Deserialize;
use thiserror::Error;

/// How many workers to start.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Workers {
    /// One per core, decided at start.
    Auto,
    /// A fixed count.
    Fixed(NonZeroU32),
}

/// The loaded configuration.
#[derive(Copy, Clone, PartialEq, Eq, Debug)] // std
#[derive(Deserialize)] // serde
#[serde(try_from = "Stored")]
pub struct Config {
    /// How many workers to start.
    pub workers: Workers,
}

/// The stored form of every version. Structure only: the meaning is checked in `Config::try_from`.
#[derive(Deserialize)]
struct Stored {
    version: u8,
    workers: StoredWorkers,
}

/// Version 1 wrote a number. Version 2 writes a number or the word `auto`.
#[derive(Deserialize)]
#[serde(untagged)]
enum StoredWorkers {
    Count(u32),
    Word(String),
}

/// Why a stored configuration did not load.
#[derive(Debug, Error)]
pub enum LoadError {
    /// The bytes are not a configuration of any version.
    #[error("not a configuration: {0}")]
    Malformed(#[from] serde_json::Error),
    /// The version is newer than this build reads.
    #[error("configuration version {0} is not supported")]
    UnsupportedVersion(u8),
    /// Version 2 does not accept a zero count. Version 1 did, and `load` migrates it.
    #[error("a worker count of zero is not valid from version 2; write \"auto\"")]
    ZeroWorkers,
    /// The word is not one version 2 defines.
    #[error("unknown worker count {0:?}; write a number or \"auto\"")]
    UnknownWord(String),
}

/// Loads a configuration of version 1 or 2.
///
/// # Errors
///
/// See [`LoadError`].
pub fn load(json: &str) -> Result<Config, LoadError> {
    let stored: Stored = serde_json::from_str(json)?;
    Config::try_from(stored)
}

impl TryFrom<Stored> for Config {
    type Error = LoadError;

    fn try_from(stored: Stored) -> Result<Self, LoadError> {
        let workers = match (stored.version, stored.workers) {
            // Version 1 wrote zero for "one per core". The meaning is kept.
            (1, StoredWorkers::Count(0)) => Workers::Auto,
            (2, StoredWorkers::Count(0)) => return Err(LoadError::ZeroWorkers),
            (1 | 2, StoredWorkers::Count(count)) => {
                let count = NonZeroU32::new(count).ok_or(LoadError::ZeroWorkers)?;
                Workers::Fixed(count)
            }
            (2, StoredWorkers::Word(word)) if word == "auto" => Workers::Auto,
            (1 | 2, StoredWorkers::Word(word)) => return Err(LoadError::UnknownWord(word)),
            (version, StoredWorkers::Count(_) | StoredWorkers::Word(_)) => {
                return Err(LoadError::UnsupportedVersion(version));
            }
        };
        Ok(Self { workers })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file written by a version 1 build. The bytes must keep loading as long as version 1 is supported.
    const GOLDEN_V1_AUTO: &str = r#"{"version":1,"workers":0}"#;
    const GOLDEN_V1_FIXED: &str = r#"{"version":1,"workers":4}"#;

    #[test]
    fn test_historical_zero_keeps_its_meaning() -> anyhow::Result<()> {
        assert_eq!(load(GOLDEN_V1_AUTO)?.workers, Workers::Auto);
        assert_eq!(load(GOLDEN_V1_FIXED)?.workers.fixed(), Some(4));
        Ok(())
    }

    #[test]
    fn test_version_2_rejects_the_zero_version_1_accepted() {
        assert!(matches!(
            load(r#"{"version":2,"workers":0}"#),
            Err(LoadError::ZeroWorkers)
        ));
        assert!(matches!(
            load(r#"{"version":2,"workers":"auto"}"#),
            Ok(Config {
                workers: Workers::Auto
            })
        ));
    }

    #[test]
    fn test_corrupt_and_unknown_inputs_are_errors_not_defaults() {
        assert!(matches!(
            load(r#"{"version":2,"workers":"#),
            Err(LoadError::Malformed(_))
        ));
        assert!(matches!(
            load(r#"{"version":3,"workers":"auto"}"#),
            Err(LoadError::UnsupportedVersion(3))
        ));
        assert!(matches!(
            load(r#"{"version":2,"workers":"many"}"#),
            Err(LoadError::UnknownWord(_))
        ));
    }

    impl Workers {
        fn fixed(self) -> Option<u32> {
            match self {
                Self::Fixed(count) => Some(count.get()),
                Self::Auto => None,
            }
        }
    }
}
