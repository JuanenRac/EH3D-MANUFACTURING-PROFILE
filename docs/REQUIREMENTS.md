# EH3D-MANUFACTURING-PROFILE: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Five requirements are `prototype` (code and tests exist, only on data made by the tests) and one is `planned` (MPRO-002); none has been used on real data or validated. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| MPRO-001 | Profile data model with units in field names. | prototype | automatic | No unit-less numeric field. | Limits are named min_/max_ plus a quantity that ends in a unit from the settings; the format refuses a field it does not know, so a number with another name is an error when reading. |
| MPRO-002 | Map the machine files used by the server to profiles. | planned | automatic: reference files | Round trip keeps every mapped value. | Not started: the machine files of the server and of the tool manager have not been read yet. |
| MPRO-003 | Tool table consistent with the tool manager. | prototype | automatic | No conflicting tool definition. | Checks the tool table of the profile for the same number defined twice with different diameters; the comparison with the tool manager itself is not written. |
| MPRO-004 | Supplier restrictions. | prototype | automatic | A restriction names its source. | A supplier profile or a restriction without a source is flagged (a setting can switch the rule off); whether the source is real cannot be known. |
| MPRO-005 | Versioned profiles cited by hash. | prototype | automatic | Same content, same hash. | The hash is the SHA-256 of the canonical JSON of the content; versions and the way records cite a hash are not written. |
| MPRO-006 | Explain a violation for the verifier. | prototype | automatic | The explanation names the limit and the remedy. | The explanation names the limit, the source and the remedy written in the profile; it cannot invent a remedy a person did not write. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
