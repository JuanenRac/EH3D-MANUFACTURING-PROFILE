// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the checks live in the other modules.
//!
//! Exit codes: 0 everything is as expected, 1 the profile has findings or the design has violations, 2 the input or the arguments are unusable.

use crate::config::Config;
use crate::error::MproError;
use crate::profile::Profile;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "\
eh3d-manufacturing-profile [--config FILE] <command>

  check <profile.json>                     list what is wrong: numbers without a unit, bad limits, conflicting tools, restrictions without a source
  hash <profile.json>                      print the SHA-256 that cites the content of the profile
  verify <profile.json> <measurements.json>
                                           compare the measurements of a design (a JSON object of quantity to number) with the limits

Exit codes: 0 nothing found, 1 findings or violations, 2 unusable input.
";

fn read_text(path: &Path, limit: u64) -> Result<String, MproError> {
    let io = |source| MproError::Io {
        path: path.to_path_buf(),
        source,
    };
    if fs::metadata(path).map_err(io)?.len() > limit {
        return Err(MproError::TooLarge { what: "input file" });
    }
    fs::read_to_string(path).map_err(io)
}

/// Run the command line with the given arguments (without the program name). Returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out) {
        Ok(code) => code,
        Err(None) => {
            let _ = err.write_all(USAGE.as_bytes());
            2
        }
        Err(Some(error)) => {
            let _ = writeln!(err, "error: {error}");
            2
        }
    }
}

fn dispatch(args: &[String], out: &mut dyn Write) -> Result<i32, Option<MproError>> {
    let mut args = args.to_vec();
    let mut config = Config::default();
    if args.first().map(String::as_str) == Some("--config") {
        if args.len() < 3 {
            return Err(None);
        }
        config = Config::parse(&read_text(Path::new(&args[1]), 1024 * 1024).map_err(Some)?)
            .map_err(Some)?;
        args.drain(..2);
    }
    let load = |path: &String| -> Result<Profile, Option<MproError>> {
        let text = read_text(Path::new(path), config.max_file_bytes).map_err(Some)?;
        Profile::from_json(&text, &config).map_err(Some)
    };
    let rest: Vec<&str> = args.iter().map(String::as_str).collect();
    match rest.as_slice() {
        ["check", _] => {
            let profile = load(&args[1])?;
            let findings = profile.check(&config);
            for f in &findings {
                let subject = if f.subject.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", f.subject)
                };
                let _ = writeln!(out, "{}{}: {}", f.code.as_str(), subject, f.message);
            }
            let _ = writeln!(out, "# {} findings", findings.len());
            Ok(if findings.is_empty() { 0 } else { 1 })
        }
        ["hash", _] => {
            let profile = load(&args[1])?;
            let _ = writeln!(out, "{}", profile.hash());
            Ok(0)
        }
        ["verify", _, _] => {
            let profile = load(&args[1])?;
            let text = read_text(Path::new(&args[2]), config.max_file_bytes).map_err(Some)?;
            let measurements: BTreeMap<String, f64> =
                serde_json::from_str(&text).map_err(|e| Some(MproError::BadJson(e.to_string())))?;
            let violations = profile.verify(&measurements, &config).map_err(Some)?;
            for v in &violations {
                let _ = writeln!(
                    out,
                    "{}: {}",
                    crate::Code::Violation.as_str(),
                    v.explanation()
                );
            }
            let _ = writeln!(out, "# {} violations", violations.len());
            Ok(if violations.is_empty() { 0 } else { 1 })
        }
        _ => Err(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(list: &[&str]) -> (i32, String, String) {
        let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    const GOOD: &str = r#"{"name":"Board house","kind":"supplier","source":"price list",
        "limits":{"min_track_mm":{"value":0.15,"remedy":"widen the tracks"}}}"#;
    const BAD: &str = r#"{"name":"Board house","kind":"supplier",
        "limits":{"min_track":{"value":0.15}}}"#;

    #[test]
    fn check_hash_and_verify_exit_codes() {
        let work = tempfile::tempdir().unwrap();
        let good = work.path().join("good.json");
        let bad = work.path().join("bad.json");
        let ok = work.path().join("ok.json");
        let low = work.path().join("low.json");
        fs::write(&good, GOOD).unwrap();
        fs::write(&bad, BAD).unwrap();
        fs::write(&ok, r#"{"track_mm":0.2}"#).unwrap();
        fs::write(&low, r#"{"track_mm":0.1}"#).unwrap();
        let (good, bad, ok, low) = (
            good.to_str().unwrap(),
            bad.to_str().unwrap(),
            ok.to_str().unwrap(),
            low.to_str().unwrap(),
        );
        assert_eq!(call(&["check", good]).0, 0);
        let (code, text, _) = call(&["check", bad]);
        assert_eq!(code, 1);
        assert!(
            text.contains("PHOENIX-MPRO-001") && text.contains("PHOENIX-MPRO-004"),
            "{text}"
        );
        let (code, hash, _) = call(&["hash", good]);
        assert_eq!((code, hash.trim().len()), (0, 64));
        assert_eq!(call(&["verify", good, ok]).0, 0);
        let (code, text, _) = call(&["verify", good, low]);
        assert_eq!(code, 1);
        assert!(
            text.contains("PHOENIX-MPRO-006") && text.contains("widen the tracks"),
            "{text}"
        );
    }

    #[test]
    fn bad_input_and_bad_arguments_exit_2() {
        let work = tempfile::tempdir().unwrap();
        let broken = work.path().join("broken.json");
        let numbers = work.path().join("numbers.json");
        fs::write(&broken, "{").unwrap();
        fs::write(&numbers, r#"{"track_mm":"wide"}"#).unwrap();
        let good = work.path().join("good.json");
        fs::write(&good, GOOD).unwrap();
        let (code, _, err) = call(&["check", broken.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("cannot be read"));
        assert_eq!(
            call(&["verify", good.to_str().unwrap(), numbers.to_str().unwrap()]).0,
            2
        );
        assert_eq!(call(&[]).0, 2);
        assert_eq!(call(&["check"]).0, 2);
        assert_eq!(call(&["check", "no-such-file.json"]).0, 2);
        assert_eq!(call(&["--config", "x"]).0, 2);
    }

    #[test]
    fn a_settings_file_makes_unknown_measurements_an_error() {
        let work = tempfile::tempdir().unwrap();
        let good = work.path().join("good.json");
        let extra = work.path().join("extra.json");
        let cfg = work.path().join("c.cfg");
        fs::write(&good, GOOD).unwrap();
        fs::write(&extra, r#"{"track_mm":0.2,"weight_count":3}"#).unwrap();
        fs::write(&cfg, "reject_unknown_measurements = on\n").unwrap();
        let (code, _, err) = call(&[
            "--config",
            cfg.to_str().unwrap(),
            "verify",
            good.to_str().unwrap(),
            extra.to_str().unwrap(),
        ]);
        assert_eq!(code, 2);
        assert!(err.contains("weight_count"), "{err}");
    }
}
