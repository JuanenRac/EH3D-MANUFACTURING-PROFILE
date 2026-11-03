# EH3D-MANUFACTURING-PROFILE: scope

Status: **early code**. EH3D-MANUFACTURING-PROFILE has a library and a command line that check a JSON profile; most of what this page lists is still planned. This page says what it is meant to do, what it will not do, and where it sits. Nothing has been run on real data.

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

The package is a single crate (a library and a command line, no window) with `check`, `hash` and `verify`; a profile is a JSON file. The reading of the machine files of the server and of the tool manager is not written.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.
