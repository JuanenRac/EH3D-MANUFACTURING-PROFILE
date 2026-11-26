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
    #[serde(default, deserialize_with = "unique_map")]
    pub limits: BTreeMap<String, Limit>,
}

/// A JSON object read as a map that refuses a key written twice: without this the last one would silently take the place of the others.
fn unique_map<'de, D, V>(deserializer: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct Unique<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for Unique<V> {
        type Value = BTreeMap<String, V>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("an object")
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, V>()? {
                if out.contains_key(&key) {
                    return Err(serde::de::Error::custom(format!(
                        "the key `{key}` is written twice"
                    )));
                }
                out.insert(key, value);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_map(Unique(std::marker::PhantomData))
}

/// Read the measurements of a design: a JSON object of quantity to number. A quantity written twice is an error, because only one of the two would be compared.
pub fn measurements_from_json(text: &str) -> Result<BTreeMap<String, f64>, MproError> {
    let bad = |e: serde_json::Error| MproError::BadJson(e.to_string());
    let mut reader = serde_json::Deserializer::from_str(text);
    let measurements = unique_map(&mut reader).map_err(bad)?;
    reader.end().map_err(bad)?;
    Ok(measurements)
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
        // a JSON array would be read as a list of the fields in order, so only an object is a profile
        if !text.trim_start().starts_with('{') {
            return Err(MproError::BadJson("a JSON object was expected".into()));
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
        // a minimum above the maximum of the same quantity: nothing can meet both
        for (name, minimum) in &self.limits {
            let Some(quantity) = name.strip_prefix("min_") else {
                continue;
            };
            if let Some(maximum) = self.limits.get(&format!("max_{quantity}")) {
                if minimum.value.is_finite()
                    && maximum.value.is_finite()
                    && minimum.value > maximum.value
                {
                    out.push(f(
                        Code::BadLimit,
                        name,
                        format!(
                            "the minimum {} is above the maximum {} of `{quantity}`: no value meets both",
                            minimum.value, maximum.value
                        ),
                    ));
                }
            }
        }
        out.sort_by(|a, b| (a.code, &a.subject).cmp(&(b.code, &b.subject)));
        out
    }

    /// The limits that no measurement answers, by name and sorted: `verify` cannot say anything about them.
    pub fn unmeasured(&self, measurements: &BTreeMap<String, f64>) -> Vec<String> {
        self.limits
            .keys()
            .filter(|name| {
                split_limit(name).is_some_and(|(_, quantity)| !measurements.contains_key(quantity))
            })
            .cloned()
            .collect()
    }

    /// Compare the measurements of a design (`quantity` to value) with the limits. The violations come sorted by the name of the limit.
    pub fn verify(
        &self,
        measurements: &BTreeMap<String, f64>,
        config: &Config,
    ) -> Result<Vec<Violation>, MproError> {
        // a number that is not finite is below no minimum and above no maximum: it would pass every limit
        if let Some((quantity, _)) = measurements.iter().find(|(_, v)| !v.is_finite()) {
            return Err(MproError::NonFiniteMeasurement(quantity.clone()));
        }
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

    #[test]
    fn a_minimum_above_the_maximum_of_the_same_quantity_is_a_bad_limit() {
        let mut p = supplier();
        p.limits
            .insert("max_track_mm".to_string(), limit(0.10, "price list", ""));
        let findings = p.check(&Config::default());
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, Code::BadLimit);
        assert_eq!(findings[0].subject, "min_track_mm");
        assert!(findings[0].message.contains("no value meets both"));
        // equal bounds are one exact value, which is possible
        p.limits
            .insert("max_track_mm".to_string(), limit(0.15, "price list", ""));
        assert!(p.check(&Config::default()).is_empty());
        // a bound that is not finite is reported once, by its own rule
        p.limits.insert(
            "max_track_mm".to_string(),
            limit(f64::NAN, "price list", ""),
        );
        assert_eq!(p.check(&Config::default()).len(), 1);
    }

    #[test]
    fn a_measurement_that_is_not_finite_is_refused_and_not_taken_as_passing() {
        let p = supplier();
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut m = BTreeMap::new();
            m.insert("track_mm".to_string(), bad);
            match p.verify(&m, &Config::default()) {
                Err(MproError::NonFiniteMeasurement(q)) => assert_eq!(q, "track_mm"),
                other => panic!("{bad}: {other:?}"),
            }
        }
        let mut m = BTreeMap::new();
        m.insert("track_mm".to_string(), 0.2);
        assert!(p.verify(&m, &Config::default()).unwrap().is_empty());
    }

    #[test]
    fn the_limits_that_no_measurement_answers_are_listed() {
        let p = supplier();
        let mut m = BTreeMap::new();
        assert_eq!(p.unmeasured(&m), vec!["max_layers_count", "min_track_mm"]);
        // a misspelt quantity answers nothing
        m.insert("trackwidth_mm".to_string(), 0.05);
        assert_eq!(p.unmeasured(&m).len(), 2);
        m.insert("track_mm".to_string(), 0.2);
        assert_eq!(p.unmeasured(&m), vec!["max_layers_count"]);
        m.insert("layers_count".to_string(), 2.0);
        assert!(p.unmeasured(&m).is_empty());
    }

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
    fn the_hash_of_a_fixed_profile_is_the_known_one() {
        // a fixed vector: a change of the order of the fields or of the way the content is written changes every hash that was ever quoted
        let p = Profile::from_json(
            r#"{"name":"m","kind":"machine","limits":{"max_a_mm":{"value":1},"min_b_mm":{"value":2}}}"#,
            &Config::default(),
        )
        .unwrap();
        assert_eq!(
            p.hash(),
            "ee0a13e3b5700fde78d524d6531642f61e230592bf6ee500883b7d72cf7dee70"
        );
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

    #[test]
    fn a_text_that_is_not_a_profile_is_an_error_and_never_a_panic() {
        let config = Config::default();
        for text in [
            "",
            " ",
            "null",
            "[]",
            "{",
            r#"{"name":1,"kind":"machine"}"#,
            r#"{"name":"m","kind":"robot"}"#,
            r#"{"name":"m"}"#,
            "\u{feff}{\"name\":\"m\",\"kind\":\"machine\"}",
        ] {
            assert!(
                matches!(
                    Profile::from_json(text, &config),
                    Err(MproError::BadJson(_))
                ),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_field_the_format_does_not_know_is_refused_in_every_part_of_the_profile() {
        let config = Config::default();
        for text in [
            r#"{"name":"m","kind":"machine","speed":3}"#,
            r#"{"name":"m","kind":"machine","tools":[{"number":1,"diameter_mm":1.0,"length":5}]}"#,
            r#"{"name":"m","kind":"machine","limits":{"max_speed_rpm":{"value":1.0,"unit":"rpm"}}}"#,
        ] {
            assert!(
                matches!(
                    Profile::from_json(text, &config),
                    Err(MproError::BadJson(_))
                ),
                "{text}"
            );
        }
    }

    #[test]
    fn the_hash_does_not_depend_on_the_order_of_the_keys_of_the_file() {
        let config = Config::default();
        let a = Profile::from_json(
            r#"{"name":"m","kind":"machine","source":"manual"}"#,
            &config,
        )
        .unwrap();
        let b = Profile::from_json(
            r#"{"source":"manual","kind":"machine","name":"m"}"#,
            &config,
        )
        .unwrap();
        assert_eq!(a.hash(), b.hash());
    }

    #[test]
    fn the_hash_is_sixty_four_lower_case_hexadecimal_digits() {
        let hash = supplier().hash();
        assert_eq!(hash.len(), 64);
        assert!(hash
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    }

    #[test]
    fn a_profile_written_and_read_again_is_the_same_profile_with_the_same_hash() {
        let config = Config::default();
        let original = supplier();
        let text = serde_json::to_string(&original).unwrap();
        let again = Profile::from_json(&text, &config).unwrap();
        assert_eq!(again, original);
        assert_eq!(again.hash(), original.hash());
        let pretty = serde_json::to_string_pretty(&original).unwrap();
        assert_eq!(
            Profile::from_json(&pretty, &config).unwrap().hash(),
            original.hash()
        );
    }

    #[test]
    fn a_change_of_one_value_of_a_limit_changes_the_hash() {
        let mut p = supplier();
        let before = p.hash();
        p.limits.get_mut("min_track_mm").unwrap().value = 0.16;
        assert_ne!(p.hash(), before);
    }

    #[test]
    fn a_measurement_exactly_on_a_limit_passes() {
        let p = supplier();
        let mut m = BTreeMap::new();
        m.insert("track_mm".to_string(), 0.15);
        m.insert("layers_count".to_string(), 4.0);
        assert!(p.verify(&m, &Config::default()).unwrap().is_empty());
    }

    #[test]
    fn the_violations_come_sorted_by_the_name_of_the_limit() {
        let p = supplier();
        let mut m = BTreeMap::new();
        m.insert("track_mm".to_string(), 0.1);
        m.insert("layers_count".to_string(), 6.0);
        let v = p.verify(&m, &Config::default()).unwrap();
        let names: Vec<&str> = v.iter().map(|x| x.limit.as_str()).collect();
        assert_eq!(names, ["max_layers_count", "min_track_mm"]);
    }

    #[test]
    fn a_limit_without_a_source_takes_the_source_of_the_profile() {
        let mut p = supplier();
        p.limits
            .insert("min_track_mm".to_string(), limit(0.15, "", ""));
        let mut m = BTreeMap::new();
        m.insert("track_mm".to_string(), 0.1);
        let v = p.verify(&m, &Config::default()).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].source, "price list");
        assert_eq!(v[0].remedy, "");
    }

    #[test]
    fn the_sentence_of_a_violation_leaves_out_the_source_and_the_remedy_when_they_are_empty() {
        let v = Violation {
            limit: "min_track_mm".into(),
            quantity: "track_mm".into(),
            measured: 0.1,
            bound: 0.15,
            source: String::new(),
            remedy: String::new(),
        };
        assert_eq!(
            v.explanation(),
            "track_mm is 0.1, below the minimum 0.15 (min_track_mm)"
        );
    }

    #[test]
    fn a_limit_that_is_neither_a_minimum_nor_a_maximum_is_never_compared_nor_listed() {
        let mut p = supplier();
        p.limits.insert("weird".to_string(), limit(1.0, "x", "y"));
        let mut m = BTreeMap::new();
        m.insert("track_mm".to_string(), 0.1);
        m.insert("layers_count".to_string(), 2.0);
        assert!(p.unmeasured(&m).is_empty());
        let v = p.verify(&m, &Config::default()).unwrap();
        assert!(v.iter().all(|x| x.limit != "weird"));
    }

    #[test]
    fn the_limits_that_no_measurement_answers_come_sorted() {
        let p = supplier();
        let m = BTreeMap::new();
        assert_eq!(p.unmeasured(&m), ["max_layers_count", "min_track_mm"]);
    }

    #[test]
    fn a_very_large_set_of_measurements_is_compared_without_a_panic() {
        let p = supplier();
        let mut m = BTreeMap::new();
        for n in 0..20_000 {
            m.insert(format!("quantity_{n}_mm"), f64::from(n));
        }
        m.insert("track_mm".to_string(), 0.1);
        let v = p.verify(&m, &Config::default()).unwrap();
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn a_profile_with_exactly_the_most_entries_is_read_and_one_more_is_not() {
        let config = Config {
            max_entries: 2,
            ..Config::default()
        };
        let mut p = supplier();
        assert_eq!(p.limits.len(), 2);
        let text = serde_json::to_string(&p).unwrap();
        assert!(Profile::from_json(&text, &config).is_ok());
        p.limits
            .insert("max_speed_rpm".to_string(), limit(9000.0, "manual", ""));
        let text = serde_json::to_string(&p).unwrap();
        assert!(matches!(
            Profile::from_json(&text, &config),
            Err(MproError::TooLarge { .. })
        ));
    }

    #[test]
    fn a_limit_written_twice_is_refused_and_the_looser_one_never_wins_in_silence() {
        let text = r#"{"name":"m","kind":"machine","limits":{"min_track_mm":{"value":0.2},"min_track_mm":{"value":0.05}}}"#;
        match Profile::from_json(text, &Config::default()) {
            Err(MproError::BadJson(message)) => {
                assert!(message.contains("min_track_mm"), "{message}")
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_measurement_written_twice_is_refused() {
        assert!(matches!(
            measurements_from_json(r#"{"track_mm":0.01,"track_mm":9}"#),
            Err(MproError::BadJson(_))
        ));
        let read = measurements_from_json(r#"{"track_mm":0.2,"layers_count":2}"#).unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(read["layers_count"], 2.0);
    }

    #[test]
    fn measurements_that_are_not_an_object_of_numbers_are_an_error() {
        for text in [
            "",
            "[]",
            "null",
            "12",
            "{",
            r#"{"a_mm":"1"}"#,
            r#"{"a_mm":null}"#,
            r#"{"a_mm":1e999}"#,
            r#"{"a_mm":1} {"b_mm":2}"#,
        ] {
            assert!(
                matches!(measurements_from_json(text), Err(MproError::BadJson(_))),
                "{text:?}"
            );
        }
        assert!(measurements_from_json("{}").unwrap().is_empty());
    }

    #[test]
    fn a_json_array_is_not_a_profile() {
        for text in [r#"["m","machine"]"#, r#" ["m","machine","",[],{}]"#] {
            assert!(
                matches!(
                    Profile::from_json(text, &Config::default()),
                    Err(MproError::BadJson(_))
                ),
                "{text}"
            );
        }
    }

    #[test]
    fn a_profile_cut_at_any_place_is_an_error_and_never_a_panic() {
        let text = serde_json::to_string(&supplier()).unwrap();
        for end in (0..text.len()).filter(|i| text.is_char_boundary(*i)) {
            assert!(
                matches!(
                    Profile::from_json(&text[..end], &Config::default()),
                    Err(MproError::BadJson(_))
                ),
                "cut at {end}"
            );
        }
    }

    #[test]
    fn a_unit_needs_a_quantity_before_it() {
        for quantity in ["mm", "_mm", "track", "trackmm", "track_cm", "track_mm_"] {
            assert!(!has_unit(quantity, &Config::default()), "{quantity}");
        }
        for quantity in ["t_mm", "feed_mm_min", "speed_rpm", "angle_deg", "temp_c"] {
            assert!(has_unit(quantity, &Config::default()), "{quantity}");
        }
    }

    #[test]
    fn the_tolerance_widens_a_maximum_and_a_negative_bound_in_the_right_direction() {
        let mut p = supplier();
        p.limits.clear();
        p.limits
            .insert("max_speed_rpm".into(), limit(1000.0, "manual", ""));
        p.limits
            .insert("min_temp_c".into(), limit(-20.0, "manual", ""));
        let loose = Config {
            tolerance_percent: 5.0,
            ..Config::default()
        };
        let measured = |speed: f64, temp: f64| {
            BTreeMap::from([
                ("speed_rpm".to_string(), speed),
                ("temp_c".to_string(), temp),
            ])
        };
        assert!(p
            .verify(&measured(1050.0, -21.0), &loose)
            .unwrap()
            .is_empty());
        assert_eq!(p.verify(&measured(1050.1, -21.1), &loose).unwrap().len(), 2);
    }

    #[test]
    fn a_seeded_campaign_agrees_with_the_plain_comparison_of_every_bound() {
        let mut state = 7u64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            f64::from((state >> 33) as u32 % 41) - 20.0
        };
        for round in 0..300 {
            let (low, high, value) = (next(), next(), next());
            let mut p = supplier();
            p.limits.clear();
            p.limits.insert("min_x_mm".into(), limit(low, "s", ""));
            p.limits.insert("max_x_mm".into(), limit(high, "s", ""));
            let found = p
                .verify(
                    &BTreeMap::from([("x_mm".to_string(), value)]),
                    &Config::default(),
                )
                .unwrap();
            let names: Vec<&str> = found.iter().map(|v| v.limit.as_str()).collect();
            assert_eq!(names.contains(&"min_x_mm"), value < low, "round {round}");
            assert_eq!(names.contains(&"max_x_mm"), value > high, "round {round}");
            let impossible = p
                .check(&Config::default())
                .iter()
                .any(|f| f.code == Code::BadLimit);
            assert_eq!(impossible, low > high, "round {round}");
        }
    }

    #[test]
    fn a_tool_with_a_diameter_that_is_negative_or_not_finite_is_a_bad_limit() {
        for bad in [-1.0, 0.0, f64::NAN, f64::INFINITY] {
            let mut p = supplier();
            p.tools[0].diameter_mm = bad;
            let found = p.check(&Config::default());
            assert_eq!(found.len(), 1, "{bad}");
            assert_eq!(found[0].code, Code::BadLimit);
            assert_eq!(found[0].subject, "tool 1");
        }
    }

    #[test]
    fn a_change_of_a_tool_of_the_kind_or_of_the_source_changes_the_hash() {
        let base = supplier().hash();
        let mut tool = supplier();
        tool.tools[1].diameter_mm = 1.2;
        let mut kind = supplier();
        kind.kind = Kind::Machine;
        let mut source = supplier();
        source.source = "manual".into();
        let mut order = supplier();
        order.tools.reverse();
        for p in [tool, kind, source, order] {
            assert_ne!(p.hash(), base);
        }
    }
}
