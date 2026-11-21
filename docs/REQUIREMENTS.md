# EH3D-MANUFACTURING-PROFILE: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run with the profile of a real machine or a real supplier.

The register has 21 requirements: 15 implemented, 5 prototype, 1 planned and 0 validated. By progress: 15 done, 5 partial, 1 pending and 0 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `MPRO-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a simulated source of data, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## The profile

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| MPRO-001 | Profile data model with the unit in the name of every limit. | prototype | partial | host-test | `src/profile::a_limit_without_a_unit_or_a_bound_prefix_is_flagged` | A limit is named `min_` or `max_` plus a quantity that ends in a unit from the settings, and a number with another name is an error when reading; the units are those of the settings, not a catalogue. |
| MPRO-002 | Map the machine files used by the server to profiles. | planned | pending | none | `planned:machine_files::a_machine_file_of_the_server_round_trips_through_a_profile` | Not written: the machine files of the server and of the tool manager have not been read yet. |
| MPRO-003 | The tool table is consistent with the tool manager. | prototype | partial | host-test | `src/profile::two_definitions_of_a_tool_with_different_data_conflict` | It finds the same tool number defined twice with different data inside the profile; the comparison with the tool manager itself is not written. |
| MPRO-004 | Supplier restrictions name their source. | prototype | partial | host-test | `src/profile::a_restriction_of_a_supplier_names_its_source` | A restriction without a source is flagged (a setting can switch the rule off); whether the source is real cannot be known. |
| MPRO-005 | Versioned profiles cited by hash. | prototype | partial | host-test | `src/profile::the_same_content_has_the_same_hash_and_a_change_changes_it` | The hash is the SHA-256 of the canonical JSON of the content; versions and the way records cite a hash are not written. |
| MPRO-006 | Explain a violation for the verifier: the limit, the source and the remedy. | prototype | partial | host-test | `src/profile::a_violation_names_the_limit_the_source_and_the_remedy` | It names the remedy that a person wrote in the profile and cannot invent one. |
| MPRO-007 | A clean profile has no findings. | implemented | done | host-test | `src/profile::a_clean_profile_has_no_findings` | - |
| MPRO-008 | A minimum above the maximum of the same quantity is a bad limit. | implemented | done | host-test | `src/profile::a_minimum_above_the_maximum_of_the_same_quantity_is_a_bad_limit` | - |
| MPRO-009 | A number that the format does not know is refused when reading. | implemented | done | host-test | `src/profile::a_number_that_the_format_does_not_know_is_refused_when_reading` | - |
| MPRO-010 | The hash of a fixed profile is a known value, so a change of the format cannot go unnoticed. | implemented | done | host-test | `src/profile::the_hash_of_a_fixed_profile_is_the_known_one` | - |
| MPRO-011 | The size limits apply. | implemented | done | host-test | `src/profile::the_size_limits_apply` | - |

## The comparison with a design

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| MPRO-012 | A measurement that is not finite is refused and never taken as passing. | implemented | done | host-test | `src/profile::a_measurement_that_is_not_finite_is_refused_and_not_taken_as_passing` | - |
| MPRO-013 | The limits that no measurement answers are listed. | implemented | done | host-test | `src/profile::the_limits_that_no_measurement_answers_are_listed` | - |
| MPRO-014 | The tolerance and the unknown measurements follow the settings. | implemented | done | host-test | `src/profile::the_tolerance_and_the_unknown_measurements_follow_the_settings` | - |
| MPRO-015 | The limits without a measurement are said and can make `verify` fail. | implemented | done | host-test | `src/cli::the_limits_without_a_measurement_are_said_and_can_make_verify_fail` | - |
| MPRO-016 | `verify` compares nothing while the profile has findings. | implemented | done | host-test | `src/cli::verify_compares_nothing_while_the_profile_has_findings` | - |

## The command line and the settings

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| MPRO-017 | `check`, `hash` and `verify` exit with the code that tells the result. | implemented | done | host-test | `src/cli::check_hash_and_verify_exit_codes` | - |
| MPRO-018 | Bad input and bad arguments exit with 2. | implemented | done | host-test | `src/cli::bad_input_and_bad_arguments_exit_2` | - |
| MPRO-019 | A settings file makes unknown measurements an error. | implemented | done | host-test | `src/cli::a_settings_file_makes_unknown_measurements_an_error` | - |
| MPRO-020 | The defaults are conservative and a bad line of the settings is refused. | implemented | done | host-test | `src/config::lines_change_values_and_bad_lines_are_refused` | - |
| MPRO-021 | A limit of zero bytes and a unit written twice are refused. | implemented | done | host-test | `src/config::a_limit_of_zero_bytes_and_a_unit_written_twice_are_refused` | - |
