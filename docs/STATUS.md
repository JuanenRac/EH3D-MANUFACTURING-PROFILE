# EH3D-MANUFACTURING-PROFILE: status

## Current state

EH3D-MANUFACTURING-PROFILE is early code. Version 0.0.1 holds only the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks) and the written scope, requirements, dependencies, threat model and decisions. It also has a Rust library and a command line that read a JSON profile whose limits always carry their unit, check it, cite it by the SHA-256 of its content and compare the measurements of a design with its limits. They have run only on data made by their own tests. Nothing has been run on a machine with real data, and no requirement is validated.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 6 requirements: 5 `prototype` and 1 `planned` (MPRO-002). This page never states a state that REQUIREMENTS.md does not.

## Open questions

* Which machine file formats the first version maps.

## Needs from other repositories

* EH3D-COMMON: container and canonical JSON

## Known technical debt and omitted tests

14 tests of the Rust code and the 3 of the base; the Rust tests build their own profiles and never touch real data. Not tested: very large profiles. `cargo deny` is not run yet.
