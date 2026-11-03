// SPDX-License-Identifier: GPL-3.0-or-later
//! The typed errors of the library. Nothing panics on bad input: every refusal is one of these.

use std::path::PathBuf;

/// Everything that can go wrong while reading a profile or a configuration.
#[derive(Debug, thiserror::Error)]
pub enum MproError {
    /// The configuration has a line that is not `key = value`.
    #[error("configuration line {line}: expected `key = value`")]
    ConfigSyntax {
        /// 1-based line number.
        line: usize,
    },
    /// The configuration names a key this version does not know.
    #[error("configuration line {line}: unknown key `{key}`")]
    ConfigUnknownKey {
        /// 1-based line number.
        line: usize,
        /// The key as written.
        key: String,
    },
    /// The configuration value cannot be used.
    #[error("configuration line {line}: bad value for `{key}`")]
    ConfigBadValue {
        /// 1-based line number.
        line: usize,
        /// The key as written.
        key: String,
    },
    /// The profile is not valid JSON of the expected shape.
    #[error("the profile cannot be read: {0}")]
    BadJson(String),
    /// A measurement names a quantity that no limit of the profile mentions, and the settings say that is an error.
    #[error("no limit of the profile mentions the measurement `{0}`")]
    UnknownMeasurement(String),
    /// A bound of the configuration was exceeded.
    #[error("limit exceeded: {what}")]
    TooLarge {
        /// Which bound.
        what: &'static str,
    },
    /// A file could not be read.
    #[error("cannot read {path}: {source}")]
    Io {
        /// The path that failed (as given by the caller).
        path: PathBuf,
        /// The operating system error.
        #[source]
        source: std::io::Error,
    },
}
