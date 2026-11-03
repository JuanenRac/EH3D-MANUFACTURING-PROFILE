// SPDX-License-Identifier: GPL-3.0-or-later
//! The settings of the library. Every tuned value is a line of the settings file; a missing line keeps the conservative default.

use crate::error::MproError;

/// The settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// The most bytes a profile or a measurements file may have.
    pub max_file_bytes: u64,
    /// The most limits, tools and restrictions a profile may hold (each).
    pub max_entries: usize,
    /// The unit suffixes a quantity may end in (without the underscore).
    pub units: Vec<String>,
    /// A supplier profile, and every restriction, must name its source.
    pub require_source: bool,
    /// A measurement that no limit mentions is reported as an error of the input rather than ignored.
    pub reject_unknown_measurements: bool,
    /// A measurement outside a limit by no more than this percent of the limit still passes (0: the limit is exact).
    pub tolerance_percent: f64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_file_bytes: 16 * 1024 * 1024,
            max_entries: 10_000,
            units: ["mm", "um", "mm_min", "rpm", "deg", "count", "percent", "c"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            require_source: true,
            reject_unknown_measurements: false,
            tolerance_percent: 0.0,
        }
    }
}

fn flag(text: &str) -> Option<bool> {
    match text {
        "true" | "on" | "yes" => Some(true),
        "false" | "off" | "no" => Some(false),
        _ => None,
    }
}

impl Config {
    /// Parse a settings file (`key = value` lines, `#` comments).
    pub fn parse(text: &str) -> Result<Config, MproError> {
        let mut config = Config::default();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or(MproError::ConfigSyntax { line })?;
            let (key, value) = (key.trim(), value.trim());
            let bad = || MproError::ConfigBadValue {
                line,
                key: key.to_string(),
            };
            match key {
                "max_file_bytes" => config.max_file_bytes = value.parse().map_err(|_| bad())?,
                "max_entries" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if n == 0 {
                        return Err(bad());
                    }
                    config.max_entries = n;
                }
                "units" => {
                    let list: Vec<String> = value
                        .split(',')
                        .map(|u| u.trim().to_ascii_lowercase())
                        .collect();
                    if list.iter().any(|u| {
                        u.is_empty() || !u.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    }) {
                        return Err(bad());
                    }
                    config.units = list;
                }
                "require_source" => config.require_source = flag(value).ok_or_else(bad)?,
                "reject_unknown_measurements" => {
                    config.reject_unknown_measurements = flag(value).ok_or_else(bad)?
                }
                "tolerance_percent" => {
                    let n: f64 = value.parse().map_err(|_| bad())?;
                    if !n.is_finite() || !(0.0..=50.0).contains(&n) {
                        return Err(bad());
                    }
                    config.tolerance_percent = n;
                }
                _ => {
                    return Err(MproError::ConfigUnknownKey {
                        line,
                        key: key.to_string(),
                    })
                }
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_conservative() {
        let c = Config::parse("").unwrap();
        assert!(c.require_source && !c.reject_unknown_measurements);
        assert_eq!(c.tolerance_percent, 0.0);
        assert!(c.units.iter().any(|u| u == "mm"));
    }

    #[test]
    fn lines_change_values_and_bad_lines_are_refused() {
        let c = Config::parse("units = mm, rpm\ntolerance_percent = 2.5\nrequire_source = off\n")
            .unwrap();
        assert_eq!(c.units, vec!["mm", "rpm"]);
        assert_eq!(c.tolerance_percent, 2.5);
        assert!(!c.require_source);
        assert!(matches!(
            Config::parse("x\n"),
            Err(MproError::ConfigSyntax { line: 1 })
        ));
        assert!(matches!(
            Config::parse("a = 1\n"),
            Err(MproError::ConfigUnknownKey { line: 1, .. })
        ));
        for bad in [
            "units = \n",
            "units = mm,,um\n",
            "tolerance_percent = 80\n",
            "tolerance_percent = nan\n",
            "max_entries = 0\n",
            "require_source = maybe\n",
        ] {
            assert!(
                matches!(
                    Config::parse(bad),
                    Err(MproError::ConfigBadValue { line: 1, .. })
                ),
                "{bad:?}"
            );
        }
    }
}
