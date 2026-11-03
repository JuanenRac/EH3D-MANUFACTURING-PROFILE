// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-MANUFACTURING-PROFILE: profiles of the capabilities of workshop and factory machinery and of the restrictions of suppliers.
//!
//! A profile is plain JSON. Its limits are named `min_<quantity>` and `max_<quantity>`, and a quantity always ends in its unit (`track_mm`, `spindle_rpm`), so no
//! number travels without one. A profile is checked here, never repaired. It can be cited by the SHA-256 of its content (the same content gives the same hash) and
//! a set of measurements of a design can be compared with it, which gives violations that name the limit, the source of the limit and the remedy.
//!
//! The limits stored here are inputs for checks. This crate is not a machine controller and never starts motion: the limits, the interlocks and the emergency stop of
//! the machine stay authoritative.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod config;
pub mod error;
pub mod profile;

pub use error::MproError;
pub use profile::{Profile, Violation};

/// The stable code of a finding. Codes are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// A number whose name does not end in a known unit.
    UnitlessField,
    /// A limit that is not `min_<quantity>` or `max_<quantity>`, or whose value is not a finite number.
    BadLimit,
    /// The same tool number is defined twice with different data.
    ConflictingTool,
    /// A restriction (or a supplier profile) that names no source.
    NoSource,
    /// A measurement of a design is outside a limit.
    Violation,
}

impl Code {
    /// The text of the code, such as `PHOENIX-MPRO-001`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::UnitlessField => "PHOENIX-MPRO-001",
            Code::BadLimit => "PHOENIX-MPRO-002",
            Code::ConflictingTool => "PHOENIX-MPRO-003",
            Code::NoSource => "PHOENIX-MPRO-004",
            Code::Violation => "PHOENIX-MPRO-006",
        }
    }
}
