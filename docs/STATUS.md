# EH3D-MANUFACTURING-PROFILE: status

## Current state

EH3D-MANUFACTURING-PROFILE is early code on a computer: a Rust library and a command line (`check`, `hash`, `verify`) that read a JSON profile whose limits always carry their unit, check it, cite it by the SHA-256 of its content and compare the measurements of a design with its limits. All of it has run only on data made by its own tests. The mapping of the machine files of the server and of the tool manager is not written, nothing has been run with the profile of a real machine and no requirement is validated.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 21 requirements: 15 implemented, 5 prototype, 1 planned and 0 validated. By progress: 15 done, 5 partial, 1 pending and 0 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* Which machine file formats the first version maps.
* How a profile records its versions and how other records cite one.

## What it needs from other repositories

* EH3D-COMMON: the container and the canonical JSON.
* EH3D-SERVER: the machine files it uses, to be mapped.
* EH3D-TOOL-MANAGER: the tool table to agree with.

## Known technical debt and omitted tests

* 63 tests of the Rust code (nine of them keep this register true) and the 68 of the base (65 of them test the shared tools of the base); the Rust tests build their own profiles and never touch real data.
* Not tested: very large profiles.
* `cargo deny` is not run yet.
