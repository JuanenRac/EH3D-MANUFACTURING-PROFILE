# EH3D-MANUFACTURING-PROFILE: dependencies

One row per relation with another repository. The kind is conceptual, compile-time or run-time. The state of a contract is: not written, drafted, accepted, frozen or published. A repository existing does not mean that its contract is ready: this project does not implement or consume a contract that is not accepted.

| From | To | Kind | Contract | Contract state | Minimum version | Tested version | Compatibility test | Change owner | What it blocks | Migration |
|---|---|---|---|---|---|---|---|---|---|---|
| EH3D-MANUFACTURING-PROFILE | EH3D-COMMON | compile | container, canonical JSON, diagnostic crate | drafted | not set | not set | to be written with the first code | the owner of the contract | all of it | to be defined |
| EH3D-MANUFACTURING-PROFILE | EH3D-SERVER | conceptual | machine files in use | drafted | not set | not set | to be written with the first code | the owner of the contract | the mapping of machine files | to be defined |
| EH3D-MANUFACTURING-PROFILE | EH3D-TOOL-MANAGER | conceptual | tool table | not written | not set | not set | to be written with the first code | the owner of the contract | the tool alignment | to be defined |
| EH3D-MANUFACTURING-PROFILE | EH3D-DESIGN-VERIFIER | conceptual | consumes profiles | not written | not set | not set | to be written with the first code | the owner of the contract | nothing here is blocked by it | to be defined |

Rules: repositories are built next to each other as sibling folders and each declares in its manifest the versions of the others that it needs, once those exist. A lower layer never depends on a higher one. The manifest field for build dependencies stays empty until code exists.
