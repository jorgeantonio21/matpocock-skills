//! The configuration file `cfgtool` validates and migrates.
//!
//! 1. A file is JSON with a `version` field. This build reads versions 1 and 2, and writes 2.
//! 2. Version 1 wrote `workers: 0` for one worker per core. Version 2 writes `"auto"` and rejects zero.
//! 3. A count is from 1 to `Workers::MAX`.
//! 4. `listen` is the socket address the server binds.
//! 5. `fixtures/` holds files written by shipped builds. They must keep loading.

use std::fs;
use std::io;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The configuration version this build writes.
pub const VERSION: u8 = 2;

/// How many workers to start.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Workers {
    /// One per core, decided at start.
    Auto,
    /// A fixed count from 1 to `Workers::MAX`.
    Fixed(NonZeroU32),
}

/// A worker count outside `1..=Workers::MAX`.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
#[error("a worker count is from 1 to {max}, not {0}; write \"auto\" for one per core", max = Workers::MAX)]
pub struct WorkersOutOfRange(pub u32);

impl Workers {
    /// The most workers a configuration may ask for.
    pub const MAX: u32 = 64;

    /// A fixed count, or `None` outside `1..=Workers::MAX`.
    #[must_use]
    pub const fn fixed(count: u32) -> Option<Self> {
        if count > Self::MAX {
            return None;
        }
        match NonZeroU32::new(count) {
            Some(count) => Some(Self::Fixed(count)),
            None => None,
        }
    }
}

impl std::fmt::Display for Workers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => f.write_str("auto"),
            Self::Fixed(count) => write!(f, "{count}"),
        }
    }
}

/// A loaded configuration, whichever version wrote it.
#[derive(Clone, PartialEq, Eq, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "Stored", into = "Stored")]
pub struct Config {
    /// The version the file named.
    pub version: u8,
    /// How many workers to start.
    pub workers: Workers,
    /// The socket address the server binds.
    pub listen: String,
}

/// The stored form of every version. Structure only: the meaning is checked in `Config::try_from`.
#[derive(Clone, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
struct Stored {
    version: u8,
    workers: StoredWorkers,
    listen: String,
}

/// Version 1 wrote a number. Version 2 writes a number or the word `auto`.
#[derive(Clone, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(untagged)]
enum StoredWorkers {
    Count(u32),
    Word(String),
}

/// Why a stored configuration has no meaning this build accepts.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The version is not one this build reads.
    #[error("version {0} is not supported; this build reads versions 1 and {supported}", supported = VERSION)]
    UnsupportedVersion(u8),
    /// The count is outside the range. Version 1's zero is migrated, not rejected.
    #[error(transparent)]
    Workers(#[from] WorkersOutOfRange),
    /// The word is not one version 2 defines.
    #[error("unknown worker count {0:?}; write a number or \"auto\"")]
    UnknownWord(String),
}

impl TryFrom<Stored> for Config {
    type Error = ConfigError;

    fn try_from(stored: Stored) -> Result<Self, ConfigError> {
        let workers = match (stored.version, stored.workers) {
            // Version 1 wrote zero for "one per core". The meaning is kept.
            (1, StoredWorkers::Count(0)) => Workers::Auto,
            (1 | 2, StoredWorkers::Count(count)) => {
                Workers::fixed(count).ok_or(WorkersOutOfRange(count))?
            }
            (2, StoredWorkers::Word(word)) if word == "auto" => Workers::Auto,
            (1 | 2, StoredWorkers::Word(word)) => return Err(ConfigError::UnknownWord(word)),
            (version, StoredWorkers::Count(_) | StoredWorkers::Word(_)) => {
                return Err(ConfigError::UnsupportedVersion(version));
            }
        };
        Ok(Self {
            version: stored.version,
            workers,
            listen: stored.listen,
        })
    }
}

impl From<Config> for Stored {
    /// The version 2 form, whichever version was loaded.
    fn from(config: Config) -> Self {
        let workers = match config.workers {
            Workers::Auto => StoredWorkers::Word("auto".to_owned()),
            Workers::Fixed(count) => StoredWorkers::Count(count.get()),
        };
        Self {
            version: VERSION,
            workers,
            listen: config.listen,
        }
    }
}

/// Why a file did not load.
#[derive(Debug, Error)]
pub enum LoadError {
    /// The file could not be read.
    #[error("read {path}: {source}", path = .path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// The read error.
        #[source]
        source: io::Error,
    },
    /// The file is not a configuration, or its meaning is rejected (see [`ConfigError`]).
    #[error("{path}: {source}", path = .path.display())]
    Malformed {
        /// The file.
        path: PathBuf,
        /// The parse or meaning error.
        #[source]
        source: serde_json::Error,
    },
}

/// Why a file was not migrated.
#[derive(Debug, Error)]
pub enum MigrateError {
    /// The input did not load, so nothing was written.
    #[error(transparent)]
    Load(#[from] LoadError),
    /// The output could not be written.
    #[error("write {path}: {source}", path = .path.display())]
    Write {
        /// The output file.
        path: PathBuf,
        /// The write error.
        #[source]
        source: io::Error,
    },
}

/// Loads and checks the configuration at `path`, of version 1 or 2.
///
/// # Errors
///
/// See [`LoadError`].
pub fn load(path: &Path) -> Result<Config, LoadError> {
    let text = fs::read_to_string(path).map_err(|source| LoadError::Io {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| LoadError::Malformed {
        path: path.to_owned(),
        source,
    })
}

/// Writes the version 2 form of the configuration at `from` to `to`. Nothing is written when
/// `from` does not load.
///
/// # Errors
///
/// See [`MigrateError`].
pub fn migrate(from: &Path, to: &Path) -> Result<Config, MigrateError> {
    let config = load(from)?;
    let stored = Stored::from(config.clone());
    let text = serde_json::to_string_pretty(&stored).map_err(|source| MigrateError::Write {
        path: to.to_owned(),
        source: source.into(),
    })?;
    fs::write(to, text).map_err(|source| MigrateError::Write {
        path: to.to_owned(),
        source,
    })?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn test_a_version_1_zero_loads_as_auto() -> anyhow::Result<()> {
        let config = load(&fixture("v1-auto.json"))?;
        assert_eq!(config.version, 1);
        assert_eq!(config.workers, Workers::Auto);
        Ok(())
    }

    #[test]
    fn test_a_version_1_count_loads_as_fixed() -> anyhow::Result<()> {
        let config = load(&fixture("v1-fixed.json"))?;
        assert_eq!(
            config.workers,
            Workers::fixed(4).ok_or_else(|| anyhow::anyhow!("4 is in range"))?
        );
        assert_eq!(config.listen, "127.0.0.1:8080");
        Ok(())
    }

    #[test]
    fn test_version_2_rejects_zero_and_above_max() {
        for json in [
            r#"{"version": 2, "workers": 0, "listen": "a"}"#,
            r#"{"version": 2, "workers": 65, "listen": "a"}"#,
            r#"{"version": 2, "workers": "many", "listen": "a"}"#,
            r#"{"version": 3, "workers": "auto", "listen": "a"}"#,
        ] {
            assert!(
                serde_json::from_str::<Config>(json).is_err(),
                "{json} is rejected"
            );
        }
    }

    #[test]
    fn test_the_written_form_is_version_2() -> anyhow::Result<()> {
        let config = load(&fixture("v1-auto.json"))?;
        let text = serde_json::to_string(&config)?;
        assert_eq!(
            text,
            r#"{"version":2,"workers":"auto","listen":"127.0.0.1:8080"}"#
        );
        Ok(())
    }

    #[test]
    fn test_a_missing_file_names_its_path() {
        let missing = fixture("missing.json");
        let error = load(&missing).expect_err("the file does not exist");
        assert!(
            error.to_string().contains("missing.json"),
            "the message names the file: {error}"
        );
    }
}
