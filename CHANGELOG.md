# Changelog

All notable changes to this project are documented here.

## [Unreleased]
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.

- Add the library: profiles of a machine or a supplier with a tool table and limits named `min_<quantity>` or `max_<quantity>` where the quantity ends in its unit; the format refuses a field it does not know.
- Find and report, with stable codes `PHOENIX-MPRO-001` to `PHOENIX-MPRO-006`: numbers without a unit, bad limits, conflicting tool definitions and restrictions without a source.
- Cite a profile by the SHA-256 of its content and compare the measurements of a design with the limits, naming the limit, its source and the remedy.
- Add the command line (`check`, `hash`, `verify`) and a settings file; run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
