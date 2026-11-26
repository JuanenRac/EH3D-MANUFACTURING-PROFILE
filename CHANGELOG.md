# Changelog

All notable changes to this project are documented here.

## [Unreleased]
- **A limit or a measurement written twice is refused:** the last one took the place of the first in silence, so a looser bound or another measurement was compared and could pass. The profile and the measurements file now say which key is repeated.
- **A JSON array is no longer read as a profile:** only an object is.
- **11 more tests** of the Rust code: a profile cut at every place, bad measurement files, the units, the tolerance on both sides and a seeded campaign against the plain comparison of every bound.
- **More tests, for the shared tools of the base and for the profile and the settings:** `tools/test_shared_tools.py` adds 65 tests that run the lessons linter, the workflow check, the CHANGELOG check of a pull request and the banner check on small inputs made by the test itself (findings, baselines, exit codes and the shape of what they print). The Rust code gets 20 more: a text that is not a profile or has a field the format does not know is an error and never a panic, the hash does not depend on the order of the keys and survives writing and reading again, a measurement exactly on a limit passes, the violations and the unmeasured limits come sorted, a limit without a source takes the source of the profile, the most entries is read and one more is not, 20 000 measurements are compared, and the settings file reads comments, the three spellings of a switch, the line of an error and the range of the tolerance.
- **The register of requirements names a test for each row:** `docs/REQUIREMENTS.md` has one row for each capability with an identifier (`MPRO-001` on), its state (planned, prototype, implemented or validated), how far it is (done, partial, pending or blocked), its evidence (simulator, host-test or none), the test that shows it and its limits; a capability that is not written names the test that will show it. `tests/reqs.rs` reads the table and fails when a test does not exist, when a state does not agree with the progress or the evidence, when `docs/STATUS.md` does not say the same numbers, when a README does not link the documents or when a document holds a date or a private reference. `docs/SCOPE.md`, `docs/STATUS.md` and `docs/NOT_YET.md` are rewritten to the same shape and the eight READMEs link the five documents.
- The shared workflow checker (`tools/workflow_hygiene.py`) is the current one of the documentation hub: the filters under a trigger (branches, paths, types) are no longer read as events.

## [0.0.4] - Reversed limits and measures that are not numbers refused

- **A test fixes the known value of the hash of a fixed profile:** until now the tests compared two calculations of the same code, so an accidental change of the order of the fields or of the way the content is written would have changed every hash that was ever quoted without a test failing. The test writes the expected hash of a fixed case.

- **The threat model says what the code already does:** the rows of `docs/THREAT_MODEL.md` that have a mitigation in the code and a test for it are marked `partial` and name the tests and what is still not covered; the rows without either stay `planned`.

- **`verify` checks the profile first:** while the profile has findings it prints them, compares nothing and exits with 1, because a profile with a bad limit gives a verdict that means nothing. The setting `check_before_verify` (on by default) turns it off.
- **`verify` says which limits it could not evaluate:** a limit that no measurement answers (an empty measurements file, a misspelt quantity) was skipped in silence and the run ended with `# 0 violations`. It now also prints how many limits were not evaluated and which, and the new setting `require_all_limits_measured` (off by default, as before) makes that a failure with exit code 1. Tested.
- **The check names a minimum above the maximum of the same quantity:** `min_track_mm` of 0.2 with `max_track_mm` of 0.1 is reported as a bad limit (`PHOENIX-MPRO-002`), because no measurement can meet both; equal bounds are accepted.
- **`verify` refuses a measurement that is not a finite number:** a value that is not a number, or infinite, was below no minimum and above no maximum and so passed every limit; now it ends the run with an error that names the quantity (exit code 2).
- **The settings refuse `max_file_bytes = 0` and a unit written twice:** a limit of zero bytes would refuse every file, and `units = mm, rpm, MM` repeats a unit (the names are compared in lower case).

## [0.0.3] - Shared validation checks, split CI workflows and pinned Rust toolchain

- **The shared validation tools are updated:** the validator also runs the workflow and lessons checks of the suite with their latest rules, including a check to run before every push.
- **The CI validation checks the workflows and the lessons of past CI failures against a baseline:** `tools/workflow_hygiene.py` and `tools/lessons_lint.py` (copies of the shared tools) run in the validation, and only findings that are not in `tools/workflow_hygiene_baseline.json` and `tools/lessons_lint_baseline.json` fail it, so what exists today stays as it is and a new one is caught. `python tools/workflow_hygiene.py --root . --detail` and `python tools/lessons_lint.py --repo .` list the findings.
- **The validation of the CI is the reusable one of the documentation hub:** the job `validate` calls `validate.yml` of EH3D-DOCS (pinned by commit) and the check of the CHANGELOG is a job of its own.
- **The CI is split in two workflows:** the validation of the repository (manifest, documentation, pictures, tools) stays in `ci.yml` and the Rust jobs move to `rust.yml`, which does not run for a change that only touches Markdown files or pictures, so a change of the documentation no longer builds the code on two systems, and they keep a cache of the build. The names of the jobs are the same.
- **The Rust toolchain is pinned:** `rust-toolchain.toml` fixes the version of Rust (the same one in every repository of the suite) with `rustfmt` and `clippy`, the workflows install that version and not `stable`, so a new release of Rust cannot turn the CI red by itself.

## [0.0.2] - First release: the library, the command line, the stable error codes and the tests

- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.

- Add the library: profiles of a machine or a supplier with a tool table and limits named `min_<quantity>` or `max_<quantity>` where the quantity ends in its unit; the format refuses a field it does not know.
- Find and report, with stable codes `PHOENIX-MPRO-001` to `PHOENIX-MPRO-006`: numbers without a unit, bad limits, conflicting tool definitions and restrictions without a source.
- Cite a profile by the SHA-256 of its content and compare the measurements of a design with the limits, naming the limit, its source and the remedy.
- Add the command line (`check`, `hash`, `verify`) and a settings file; run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
