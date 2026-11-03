// SPDX-License-Identifier: GPL-3.0-or-later
//! The profile: tools, limits and restrictions with their sources, the checks, the hash that cites it and the comparison with the measurements of a design.

use crate::config::Config;
use crate::error::MproError;
use crate::Code;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Whose capabilities the profile describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// A machine of a workshop or a factory.
    Machine,
    /// The restrictions of a supplier.
    Supplier,
}

/// A tool of the tool table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    /// The number of the tool in the table.
    pub number: u32,
    /// Its diameter in millimetres.
    pub diameter_mm: f64,
    /// A free note.
    #[serde(default)]
    pub note: String,
}

/// One limit: the bound, where it comes from and what to do about a violation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limit {
    /// The bound, in the unit the name of the limit ends in.
    pub value: f64,
    /// The source of the bound (a document of the supplier, the manual of the machine, a measurement).
    #[serde(default)]
    pub source: String,
    /// What to change in the design when the bound is not met.
    #[serde(default)]
    pub remedy: String,
}

/// A profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// The name of the machine or the supplier.
    pub name: String,
    /// Machine or supplier.
    pub kind: Kind,
    /// The source of the profile as a whole.
    #[serde(default)]
    pub source: String,
    /// The tool table.
    #[serde(default)]
    pub tools: Vec<Tool>,
    /// The limits, named `min_<quantity>` or `max_<quantity>` where the quantity ends in a unit.
    #[serde(default)]
    pub limits: BTreeMap<String, Limit>,
}

/// Something wrong in a profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The code.
    pub code: Code,
    /// What it is about (a limit name, or `tool <number>`, or empty for the profile).
    pub subject: String,
    /// What is wrong, in English.
    pub message: String,
}

/// A measurement outside a limit.
#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    /// The name of the limit, such as `min_track_mm`.
    pub limit: String,
    /// The quantity measured, such as `track_mm`.
    pub quantity: String,
    /// The value measured.
    pub measured: f64,
    /// The bound of the limit.
    pub bound: f64,
    /// The source of the bound.
    pub source: String,
    /// What to change.
    pub remedy: String,
}

impl Violation {
    /// The sentence that names the limit, the source and the remedy.
    pub fn explanation(&self) -> String {
        let what = if self.limit.starts_with("min_") {
            "below the minimum"
        } else {
            "above the maximum"
        };
        let mut text = format!(
            "{} is {}, {} {} ({})",
            self.quantity, self.measured, what, self.bound, self.limit
        );
        if !self.source.is_empty() {
            text.push_str(&format!("; source: {}", self.source));
        }
        if !self.remedy.is_empty() {
            text.push_str(&format!("; remedy: {}", self.remedy));
        }
        text
    }
}

/// Split `min_x_mm` into (`true`, `x_mm`) and `max_x_mm` into (`false`, `x_mm`).
fn split_limit(name: &str) -> Option<(bool, &str)> {
    if let Some(q) = name.strip_prefix("min_") {
        Some((true, q))
    } else {
        name.strip_prefix("max_").map(|q| (false, q))
    }
}

fn has_unit(quantity: &str, config: &Config) -> bool {
    config.units.iter().any(|u| {
        quantity
            .strip_suffix(u.as_str())
            .is_some_and(|rest| rest.ends_with('_') && rest.len() > 1)
    })
}

impl Profile {
    /// Read a profile from JSON, within the bounds of the configuration. A field the format does not know is an error, so a number without a unit cannot slip in.
    pub fn from_json(text: &str, config: &Config) -> Result<Profile, MproError> {
        if text.len() as u64 > config.max_file_bytes {
            return Err(MproError::TooLarge {
                what: "profile bytes",
            });
        }
        let profile: Profile =
            serde_json::from_str(text).map_err(|e| MproError::BadJson(e.to_string()))?;
        if profile.tools.len() > config.max_entries || profile.limits.len() > config.max_entries {
            return Err(MproError::TooLarge { what: "entries" });
        }
        Ok(profile)
    }

    /// The SHA-256 (lower-case hexadecimal) of the content: the same content gives the same hash whatever the order of the keys in the file it came from.
    pub fn hash(&self) -> String {
        let text = serde_json::to_string(self).unwrap_or_default();
        let digest = Sha256::digest(text.as_bytes());
        digest.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// What is wrong, in a fixed order.
    pub fn check(&self, config: &Config) -> Vec<Finding> {
        let mut out = Vec::new();
        let f = |code, subject: &str, message: String| Finding {
            code,
            subject: subject.to_string(),
            message,
        };
        if config.require_source && self.kind == Kind::Supplier && self.source.trim().is_empty() {
            out.push(f(
                Code::NoSource,
                "",
                "a supplier profile must name its source".into(),
            ));
        }
        let mut seen: BTreeMap<u32, f64> = BTreeMap::new();
        for tool in &self.tools {
            let subject = format!("tool {}", tool.number);
            if !tool.diameter_mm.is_finite() || tool.diameter_mm <= 0.0 {
                out.push(f(
                    Code::BadLimit,
                    &subject,
                    "the diameter must be a positive number".into(),
                ));
            }
            match seen.get(&tool.number) {
                Some(d) if *d != tool.diameter_mm => out.push(f(
                    Code::ConflictingTool,
                    &subject,
                    "defined twice with different diameters".into(),
                )),
                Some(_) => {}
                None => {
                    seen.insert(tool.number, tool.diameter_mm);
                }
            }
        }
        for (name, limit) in &self.limits {
            match split_limit(name) {
                None => out.push(f(
                    Code::BadLimit,
                    name,
                    "the name of a limit starts with min_ or max_".into(),
                )),
                Some((_, quantity)) => {
                    if !has_unit(quantity, config) {
                        out.push(f(
                            Code::UnitlessField,
                            name,
                            format!(
                                "the quantity `{quantity}` does not end in one of the units {}",
                                config.units.join(", ")
                            ),
                        ));
                    }
                }
            }
            if !limit.value.is_finite() {
                out.push(f(
                    Code::BadLimit,
                    name,
                    "the value is not a finite number".into(),
                ));
            }
            if config.require_source
                && self.kind == Kind::Supplier
                && limit.source.trim().is_empty()
                && self.source.trim().is_empty()
            {
                out.push(f(
                    Code::NoSource,
                    name,
                    "a restriction of a supplier must name its source".into(),
                ));
            }
        }
        out.sort_by(|a, b| (a.code, &a.subject).cmp(&(b.code, &b.subject)));
        out
    }

    /// Compare the measurements of a design (`quantity` to value) with the limits. The violations come sorted by the name of the limit.
    pub fn verify(
        &self,
        measurements: &BTreeMap<String, f64>,
        config: &Config,
    ) -> Result<Vec<Violation>, MproError> {
        if config.reject_unknown_measurements {
            for quantity in measurements.keys() {
                let mentioned = self
                    .limits
                    .keys()
                    .any(|name| split_limit(name).is_some_and(|(_, q)| q == quantity.as_str()));
                if !mentioned {
                    return Err(MproError::UnknownMeasurement(quantity.clone()));
                }
            }
        }
        let mut out = Vec::new();
        for (name, limit) in &self.limits {
            let Some((is_min, quantity)) = split_limit(name) else {
                continue;
            };
            let Some(&measured) = measurements.get(quantity) else {
                continue;
            };
            let tolerance = limit.value.abs() * config.tolerance_percent / 100.0;
            let outside = if is_min {
                measured < limit.value - tolerance
            } else {
                measured > limit.value + tolerance
            };
            if outside {
                out.push(Violation {
                    limit: name.clone(),
                    quantity: quantity.to_string(),
                    measured,
                    bound: limit.value,
                    source: if limit.source.is_empty() {
                        self.source.clone()
                    } else {
                        limit.source.clone()
                    },
                    remedy: limit.remedy.clone(),
                });
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limit(value: f64, source: &str, remedy: &str) -> Limit {
        Limit {
            value,
            source: source.into(),
            remedy: remedy.into(),
        }
    }

    fn supplier() -> Profile {
        let mut limits = BTreeMap::new();
        limits.insert(
            "min_track_mm".to_string(),
            limit(0.15, "price list", "widen the tracks"),
        );
        limits.insert(
            "max_layers_count".to_string(),
            limit(4.0, "price list", "use fewer layers"),
        );
        Profile {
            name: "Board house".into(),
            kind: Kind::Supplier,
            source: "price list".into(),
            tools: vec![
                Tool {
                    number: 1,
                    diameter_mm: 0.8,
                    note: String::new(),
                },
                Tool {
                    number: 2,
                    diameter_mm: 1.0,
                    note: String::new(),
                },
            ],
            limits,
        }
    }

    fn codes(p: &Profile) -> Vec<Code> {
        p.check(&Config::default()).iter().map(|f| f.code).collect()
    }

    #[test]
    fn a_clean_profile_has_no_findings() {
        assert!(codes(&supplier()).is_empty());
    }

    #[test]
    fn a_limit_without_a_unit_or_a_bound_prefix_is_flagged() {
        let mut p = supplier();
        p.limits.insert("min_track".into(), limit(0.1, "x", ""));
        p.limits.insert("track_mm".into(), limit(0.1, "x", ""));
        p.limits
            .insert("max_speed_mm_min".into(), limit(f64::INFINITY, "x", ""));
        let c = codes(&p);
        assert!(c.contains(&Code::UnitlessField));
        assert_eq!(c.iter().filter(|c| **c == Code::BadLimit).count(), 2);
    }

    #[test]
    fn a_number_that_the_format_does_not_know_is_refused_when_reading() {
        let config = Config::default();
        let text = r#"{"name":"m","kind":"machine","tools":[{"number":1,"diameter":0.8}]}"#;
        assert!(matches!(
            Profile::from_json(text, &config),
            Err(MproError::BadJson(_))
        ));
        let text = r#"{"name":"m","kind":"machine","tools":[{"number":1,"diameter_mm":0.8}]}"#;
        assert_eq!(Profile::from_json(text, &config).unwrap().tools.len(), 1);
        for bad in [
            "",
            "[]",
            r#"{"name":"m"}"#,
            r#"{"name":"m","kind":"robot"}"#,
        ] {
            assert!(
                matches!(Profile::from_json(bad, &config), Err(MproError::BadJson(_))),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn two_definitions_of_a_tool_with_different_data_conflict() {
        let mut p = supplier();
        p.tools.push(Tool {
            number: 1,
            diameter_mm: 0.9,
            note: String::new(),
        });
        assert_eq!(codes(&p), vec![Code::ConflictingTool]);
        // the same definition twice is not a conflict
        let mut q = supplier();
        q.tools.push(q.tools[0].clone());
        assert!(codes(&q).is_empty());
        let mut r = supplier();
        r.tools[0].diameter_mm = 0.0;
        assert_eq!(codes(&r), vec![Code::BadLimit]);
    }

    #[test]
    fn a_restriction_of_a_supplier_names_its_source() {
        let mut p = supplier();
        p.source = String::new();
        p.limits.get_mut("min_track_mm").unwrap().source = String::new();
        let found = p.check(&Config::default());
        assert!(found
            .iter()
            .any(|f| f.code == Code::NoSource && f.subject == "min_track_mm"));
        assert!(found
            .iter()
            .any(|f| f.code == Code::NoSource && f.subject.is_empty()));
        // a machine does not need one, and the rule has a setting
        p.kind = Kind::Machine;
        assert!(codes(&p).is_empty());
        let open = Config {
            require_source: false,
            ..Config::default()
        };
        p.kind = Kind::Supplier;
        assert!(p.check(&open).is_empty());
    }

    #[test]
    fn the_same_content_has_the_same_hash_and_a_change_changes_it() {
        let config = Config::default();
        let a = r#"{"name":"m","kind":"machine","limits":{"max_a_mm":{"value":1},"min_b_mm":{"value":2}}}"#;
        let b = r#"{"limits":{"min_b_mm":{"value":2.0},"max_a_mm":{"value":1.0}},"kind":"machine","name":"m"}"#;
        let (a, b) = (
            Profile::from_json(a, &config).unwrap(),
            Profile::from_json(b, &config).unwrap(),
        );
        assert_eq!(a.hash(), b.hash());
        assert_eq!(a.hash().len(), 64);
        let mut c = a.clone();
        c.limits.get_mut("max_a_mm").unwrap().value = 1.5;
        assert_ne!(a.hash(), c.hash());
    }

    #[test]
    fn a_violation_names_the_limit_the_source_and_the_remedy() {
        let p = supplier();
        let mut m = BTreeMap::new();
        m.insert("track_mm".to_string(), 0.1);
        m.insert("layers_count".to_string(), 4.0);
        let v = p.verify(&m, &Config::default()).unwrap();
        assert_eq!(v.len(), 1);
        let text = v[0].explanation();
        assert!(
            text.contains("track_mm") && text.contains("min_track_mm"),
            "{text}"
        );
        assert!(
            text.contains("price list") && text.contains("widen the tracks"),
            "{text}"
        );
        m.insert("layers_count".to_string(), 6.0);
        let v = p.verify(&m, &Config::default()).unwrap();
        assert_eq!(
            v.iter().map(|x| x.limit.as_str()).collect::<Vec<_>>(),
            vec!["max_layers_count", "min_track_mm"]
        );
        assert!(v[0].explanation().contains("above the maximum"));
    }

    #[test]
    fn the_tolerance_and_the_unknown_measurements_follow_the_settings() {
        let p = supplier();
        let mut m = BTreeMap::new();
        m.insert("track_mm".to_string(), 0.148);
        assert_eq!(p.verify(&m, &Config::default()).unwrap().len(), 1);
        let loose = Config {
            tolerance_percent: 2.0,
            ..Config::default()
        };
        assert!(p.verify(&m, &loose).unwrap().is_empty());
        m.insert("weight_count".to_string(), 3.0);
        assert!(p.verify(&m, &Config::default()).is_ok());
        let strict = Config {
            reject_unknown_measurements: true,
            ..Config::default()
        };
        assert!(matches!(
            p.verify(&m, &strict),
            Err(MproError::UnknownMeasurement(q)) if q == "weight_count"
        ));
    }

    #[test]
    fn the_size_limits_apply() {
        let small = Config {
            max_file_bytes: 4,
            ..Config::default()
        };
        assert!(matches!(
            Profile::from_json(r#"{"name":"m","kind":"machine"}"#, &small),
            Err(MproError::TooLarge { .. })
        ));
        let few = Config {
            max_entries: 1,
            ..Config::default()
        };
        let text = serde_json::to_string(&supplier()).unwrap();
        assert!(matches!(
            Profile::from_json(&text, &few),
            Err(MproError::TooLarge { .. })
        ));
    }
}
