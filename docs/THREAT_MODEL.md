# EH3D-MANUFACTURING-PROFILE: threat model

Basic model for early code. The rows marked partial have a mitigation written and tests for it, run only on data made by the tests themselves; the rows marked planned have nothing yet. No row is assessed against real data.

| Threat | Planned mitigation | Planned test | State | Residual risk |
|---|---|---|---|---|
| A tampered profile that widens a limit | the hash of a profile changes with any change of its content, so a changed limit no longer matches the hash that was published | `the_same_content_has_the_same_hash_and_a_change_changes_it` | partial | the hash is not signed: a profile replaced together with its hash is not detected |
| A profile read as authority over the machine | documented as an input; the machine keeps its own limits | review | planned | not assessed: nothing is implemented |
| Corrupt or malicious project or input files (oversized entries, path tricks, deep nesting, resource exhaustion) | the reader refuses a number the format does not know and applies the size limits | `a_number_that_the_format_does_not_know_is_refused_when_reading`, `the_size_limits_apply` | partial | hostile samples written by hand; no fuzzing yet |
| Temporary files, logs and reports that reveal private designs | no private data in outputs; redaction | automated pattern search | planned | not assessed: nothing is implemented |
| Compromised dependency or binary | pinned versions, licence and advisory checks, software bill of materials for releases | CI checks | planned | not assessed: nothing is implemented |
| Power loss or forced close during a save | atomic write: temporary file, then rename; old file never deleted first | interruption tests at controlled points | planned | not assessed: nothing is implemented |

Minimum negative tests once there is code: hostile files and resource limits; secrets absent from logs, errors and reports; pinned dependencies and licence review before publishing. Report a vulnerability privately as described in SECURITY.md, without attaching private designs.
