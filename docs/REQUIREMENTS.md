# EH3D-MANUFACTURING-PROFILE: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `planned` today: nothing is implemented or verified. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| MPRO-001 | Profile data model with units in field names. | planned | automatic | No unit-less numeric field. | Not started: no limits known yet. |
| MPRO-002 | Map the machine files used by the server to profiles. | planned | automatic: reference files | Round trip keeps every mapped value. | Not started: no limits known yet. |
| MPRO-003 | Tool table consistent with the tool manager. | planned | automatic | No conflicting tool definition. | Not started: no limits known yet. |
| MPRO-004 | Supplier restrictions. | planned | automatic | A restriction names its source. | Not started: no limits known yet. |
| MPRO-005 | Versioned profiles cited by hash. | planned | automatic | Same content, same hash. | Not started: no limits known yet. |
| MPRO-006 | Explain a violation for the verifier. | planned | automatic | The explanation names the limit and the remedy. | Not started: no limits known yet. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
