//! The configuration file `cfgtool` validates.
//!
//! 1. A file is JSON with a `version` field. This build reads version 1.
//! 2. `workers` is a count. Zero means one worker per core.
//! 3. `listen` is the socket address the server binds.
//! 4. `fixtures/` holds files written by shipped builds. They must keep loading.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The configuration version this build reads.
pub const VERSION: u8 = 1;

/// A loaded configuration.
#[derive(Clone, PartialEq, Eq, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
pub struct Config {
    /// The file's version.
    pub version: u8,
    /// The worker count. Zero means one worker per core.
    pub workers: u32,
    /// The socket address the server binds.
    pub listen: String,
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
    /// The file is not a configuration.
    #[error("{path} is not a configuration: {source}", path = .path.display())]
    Malformed {
        /// The file.
        path: PathBuf,
        /// The parse error.
        #[source]
        source: serde_json::Error,
    },
    /// The file's version is not one this build reads.
    #[error("{path}: version {version} is not supported; this build reads version {supported}", path = .path.display(), supported = VERSION)]
    UnsupportedVersion {
        /// The file.
        path: PathBuf,
        /// The version the file names.
        version: u8,
    },
}

/// Loads and checks the configuration at `path`.
///
/// # Errors
///
/// See [`LoadError`].
pub fn load(path: &Path) -> Result<Config, LoadError> {
    let text = fs::read_to_string(path).map_err(|source| LoadError::Io {
        path: path.to_owned(),
        source,
    })?;
    let config: Config = serde_json::from_str(&text).map_err(|source| LoadError::Malformed {
        path: path.to_owned(),
        source,
    })?;
    if config.version != VERSION {
        return Err(LoadError::UnsupportedVersion {
            path: path.to_owned(),
            version: config.version,
        });
    }
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
    fn test_a_version_1_file_loads() -> anyhow::Result<()> {
        let config = load(&fixture("v1-fixed.json"))?;
        assert_eq!(config.version, 1);
        assert_eq!(config.workers, 4);
        assert_eq!(config.listen, "127.0.0.1:8080");
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
