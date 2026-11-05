<p align="center">
  <img src="images/EH3D_MANUFACTURING_PROFILE_BANNER.svg" alt="EH3D-MANUFACTURING-PROFILE banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-MANUFACTURING-PROFILE

<p align="center">🇺🇸 <b>English</b> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | <a href="README_ita.md">🇮🇹 Italiano</a> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### EH3D-MANUFACTURING-PROFILE: profiles of the capabilities of workshop and factory machinery and of the restrictions of suppliers. Early code.

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Markdown%20%2B%20Python-blue.svg" alt="Markdown and Python">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-3-2EA043.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.2-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Honesty check - what actually runs today:** a Rust library and a command line that read a JSON profile whose limits always carry their unit, find what is wrong with it (numbers without a unit, bad limits, conflicting tool definitions, restrictions without a source), cite it by the SHA-256 of its content and compare the measurements of a design with its limits, naming the limit, the source and the remedy. All of it has run only on data made by its own tests. What does not exist: the reading of the machine files that the server and the tool manager use, and the link with the verifier.

---

> **Safety and security status:** this repository is at its first steps and is not a safety system. Nothing written here replaces the limits, interlocks and emergency stop of the machine and its controller.

## 📋 At a glance

| | |
|---|---|
| Version | `0.0.2` |
| Maturity | Scaffolding |
| Tests | 3 |
| Stack | Markdown, Python 3.10+ |
| On real hardware | Not run on hardware yet |

**Jump to:** [Overview](#-overview) · [Architecture](#-architecture) · [Quick start](#-quick-start) · [Testing](#-testing) · [Repository Structure](#-repository-structure) · [Related Projects](#-related-projects)

## 🚦 Project status

Each line is one part of this repository and the light says how far it is today. It is written from what the repository can show (tests, builds, runs), not from what is planned.

🟢 Done and verified · 🟡 Works, with known limits · 🟠 Partly done or not verified yet · 🔴 Not done or not working

| Area | State | What it means here |
|---|---|---|
| Repository checks | 🟡 | The repository check, the check of the pictures and the 3 tests of the base run in CI and pass. |
| Tests | 🟡 | 14 tests of the Rust code and the 3 of the base pass on a computer. They use only data made by the tests themselves. |
| Real hardware | 🟠 | Not run on any machine or board: nothing here has been tried on hardware. |
| Planned function | 🟠 | Written: the profile with units in every name, the tool table, supplier restrictions with their source, the hash that cites a profile and the explanation of a violation. Not written: the mapping of the machine files of the server and of the tool manager. |
| Requirements | 🟠 | 5 of the 6 requirements are at prototype; the mapping of the machine files (MPRO-002) is only planned. None is validated. |

## 🎯 Overview

* **What exists:** a library and a command line, without a window, that read a JSON profile, check it (`check`), print its hash (`hash`) and compare the measurements of a design with it (`verify`). **Planned:** the reading of the machine files of the server and of the tool manager, the link with the verifier and a kind of server task.

## 🧩 Role in EH3D

EH3D-MANUFACTURING-PROFILE is part of the PHOENIX family of the EH3D Fabrication Suite. What it will do, what it will not do yet, its requirements and its relations with other repositories are written in the `docs/` folder.

## 🏛️ Architecture

<p align="center">
  <img src="images/architecture.svg" alt="EH3D-MANUFACTURING-PROFILE architecture: who uses it, what it holds and what it depends on" width="100%">
</p>

How the parts of this repository fit together: who uses it, what it holds and what it depends on. The picture is drawn from `docs/architecture.json` and checked against the code.

## 🚀 Quick start

You need Python 3.10 or newer; there is nothing to install.

```bash
python -I tools/ci_validate.py
python -I tools/make_architecture.py --check
python -I tools/make_structure.py --check --readmes
python -I -m unittest discover -s tools -p "test_*.py"
```

## 🧪 Testing

The base has 3 tests (`python -I -m unittest discover -s tools -p "test_*.py"`), and they pass. The CI runs the repository check, the check of the pictures and those tests.

## 🏗️ Build & Release

The version lives in the `VERSION` file and in `eh3d.project.json`, and each version has its entry in [CHANGELOG.md](CHANGELOG.md). There is nothing to build; releases are published on GitHub.

## 📂 Repository Structure

<p align="center">
  <img src="images/repository-structure.svg" alt="Repository structure of EH3D-MANUFACTURING-PROFILE" width="100%">
</p>

<details>
<summary>Text version of the tree</summary>

```text
EH3D-MANUFACTURING-PROFILE/
├── docs/
│   ├── architecture.json structure.json         the data of the two pictures
│   ├── SCOPE.md REQUIREMENTS.md                 what is planned and its requirements
│   ├── DEPENDENCIES.md THREAT_MODEL.md          relations with other repositories and the threat model
│   ├── DECISIONS.md STATUS.md                   decisions that apply here and the honest status
│   ├── NOT_YET.md RULES.md                      ideas left for later and rules for writing code
│   └── adr/                                     the accepted decision records
├── AGENTS.md                                    rules for anyone who changes the repository
├── src/                                         the library and the command line
├── Cargo.toml Cargo.lock deny.toml              Rust package and policy of the dependencies
├── tools/                                       the repository check, its tests and the picture tools
├── images/                                      banner and the Architecture and Repository Structure pictures
├── .github/                                     CI and dashboard workflows, issue and pull-request templates
├── eh3d.project.json                            manifest: version, role, maturity and build command
├── VERSION                                      the version of the repository
├── CHANGELOG.md                                 what changed in each version
├── SECURITY.md SUPPORT.md                       how to report a vulnerability and where to ask
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   how to contribute, the code of conduct and the licence
```

</details>

## 🗺️ Roadmap & Known limits

**Known limits today**

* Early code: it has only run on data made by its own tests; the machine files of the server and of the tool manager are not read yet.

**Next**

* Wait until the contracts this project depends on are accepted (see `docs/DEPENDENCIES.md`), then write the first version with tests and replace the status rows with real results.

## 🌍 Languages

README and texts exist in English, Spanish, French, Italian, German, Chinese, Japanese and European Portuguese.

## 🔗 Related Projects

* [EH3D-DOCS](https://github.com/JuanenRac/EH3D-DOCS): the map of the suite, its architecture, the conventions and the shared checks
* Planned relations, not built yet (EH3D-COMMON, EH3D-SERVER, EH3D-TOOL-MANAGER, EH3D-DESIGN-VERIFIER): see [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)

## 📚 Documentation

* [The conventions of the suite](https://github.com/JuanenRac/EH3D-DOCS/blob/main/docs/CONVENTIONS.md)
* [Changelog](CHANGELOG.md)
* [Scope, requirements, dependencies, threat model and decisions](docs/SCOPE.md)

## 🤝 Contributing & Community

* Before a pull request run `python -I tools/ci_validate.py` and the tests of the base: all must pass. See [CONTRIBUTING.md](CONTRIBUTING.md) and the [code of conduct](CODE_OF_CONDUCT.md).
* The README exists in eight languages: a change to the text is made in all eight files, and the two pictures are drawn from `docs/architecture.json` and `docs/structure.json` by `tools/make_architecture.py` and `tools/make_structure.py`.
* Report a vulnerability privately: [SECURITY.md](SECURITY.md). Questions, ideas and reports: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTHOR

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENSE

Software: GPL-3.0-or-later. See [LICENSE](LICENSE). Hardware designs of the suite use CERN OHL-S.
