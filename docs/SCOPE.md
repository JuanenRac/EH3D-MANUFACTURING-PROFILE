# EH3D-MANUFACTURING-PROFILE: scope

Status: **planned**. EH3D-MANUFACTURING-PROFILE has no code. This page says what it is meant to do, what it will not do, and where it sits. Nothing here has been built or run.

## Purpose

EH3D-MANUFACTURING-PROFILE is planned as a manager of profiles of the capabilities of factory machinery and the restrictions of suppliers.

Manager of manufacturing profiles: capabilities of workshop and factory machinery and the restrictions of suppliers.

## What it is planned to do

* A profile per machine or supplier: tools, diameters and cutting limits, tolerances, materials and thicknesses, isolation and clearance, drilling limits and assembly restrictions.
* Alignment with the machine files the server already uses and with the tool manager.
* Versioned profiles that other records can cite by hash.
* Let the verifier check a design against a profile and explain what to change.

## What it will not do

* It is not a machine controller and never starts motion.
* The limits it stores are inputs for checks; the limits, interlocks and emergency stop of the machine stay authoritative.
* No proprietary supplier data is published without permission.

## Planned shape

A library without a window, written in Rust (planned), with a command-line interface and, where it applies, a kind of task for the server. Programs of the suite exchange work through project files or through the server, never by depending on each other.

Crates, names and file layout will be decided together with the first code; no crate exists yet.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.
