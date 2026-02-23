# Story 4.2: Comprehensive Command-Line Documentation

Status: done

## Story

As an operations engineer,
I want built-in help text explaining flags and subcommands,
So that I understand how to format mapping arguments or timeout parameters correctly.

## Acceptance Criteria

1. **Given** a terminal session
2. **When** I execute `jfrog2nexus --help` or `jfrog2nexus sync --help`
3. **Then** the tool outputs detailed, human-readable instructions describing all available arguments, environment variables (`J2N_*`), and configuration paths.

## Tasks / Subtasks

- [x] Task 1: Enrich Help Text (AC: 3)
  - [x] Use `clap` attributes to add detailed descriptions to all commands and flags.
- [x] Task 2: Environment Variable Documentation (AC: 3)
  - [x] Ensure `env` attributes are used so `clap` shows them in help.
- [x] Task 3: Validation (AC: 1, 2)
  - [x] Audit `--help` output for completeness.

## Dev Notes

- All commands have descriptions.
- Environment variables are documented via `env` flags in `SyncArgs` or via `Config` command notes.
