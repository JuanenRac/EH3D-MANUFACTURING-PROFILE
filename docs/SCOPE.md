# EH3D-MANUFACTURING-PROFILE: scope

Status: **early code**. EH3D-MANUFACTURING-PROFILE is a library and a command line that check a JSON profile, cite it by the SHA-256 of its content and compare the measurements of a design with its limits. It has been tried on data made by its own tests only. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The manager of manufacturing profiles: the capabilities of the machinery of a workshop or a factory and the restrictions of a supplier (tools, diameters and cutting limits, tolerances, materials and thicknesses, isolation and clearance, drilling limits and assembly restrictions), stored so that a limit always carries its unit and another record can cite a profile by its hash.

## What it does

* Reads a profile (a JSON file within a size limit) whose limits are named `min_` or `max_` plus a quantity that ends in a unit from the settings, and refuses a field it does not know.
* Checks it: a limit without a unit or a bound prefix, a minimum above its maximum, a tool defined twice with different data and a supplier restriction without a source are findings.
* Cites a profile by the SHA-256 of its canonical content: the same content has the same hash and a change changes it.
* Compares the measurements of a design with the limits, refuses a measurement that is not finite, lists the limits that no measurement answers, and for each violation names the limit, its source and the remedy written in the profile.
* Takes the tolerance and the rule about unknown measurements from a settings file.

## What it will not do

* It is not a machine controller and never starts motion.
* The limits it stores are inputs for checks; the limits, interlocks and emergency stop of the machine stay authoritative.
* It does not read the machine files of the server or the tool manager yet (see `REQUIREMENTS.md`).
* It publishes no proprietary supplier data without permission.
* It does not claim that anything was proven with a real machine: the evidence of every capability is `host-test`.

## Where it sits

* **It reads:** a profile file (JSON), the measurements of a design (JSON) and a settings file (the tolerance, the units that are accepted, the size limits and what to do with unknown measurements).
* **It writes:** the hash, the findings and the comparison it is asked to print, to the standard output; it never writes to the profile it reads.
* **It is read by:** nothing reads it yet: the design verifier will read a profile to check a design and explain what to change.
* **It depends on:** nothing of the suite at build time; the container and the canonical JSON come from EH3D-COMMON.

## What it never decides

Whether a design can be made on a machine, and the limits, the interlocks and the emergency stop of that machine, belong to the machine and to the person who runs it. A profile is a statement about a machine that somebody wrote; this program checks its form and compares numbers with it, and cannot know that the statement is true. Nothing here starts motion.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
