# Story 2.2: Resumable Transfers via Checksum Matching

Status: done

## Story

As the lead migration engineer,
I want the sync process to skip artifacts that are already fully transferred,
So that network interruptions don't force me to restart massive multi-GB file downloads from zero.

## Acceptance Criteria

1. **Given** a partially completed migration where some artifacts exist on the target
2. **When** `jfrog2nexus sync --resume-by-checksum` is executed
3. **Then** the system queries the local `.j2n/state.db` SQLite database to compare checksums
4. **And** skips identical files without requiring remote target API validation, downloading only missing artifacts.

## Tasks / Subtasks

- [x] Task 1: SQLite State Store Implementation (AC: 3)
  - [x] Initialize SQLite database at `.j2n/state.db`.
  - [x] Implement `StateStore` with `get_artifact_status` and `mark_complete`.
- [x] Task 2: Integrate into Transfer Engine (AC: 4)
  - [x] Check `StateStore` before initiating `transfer_artifact`.
  - [x] Mark artifact as complete in `StateStore` after successful transfer.
- [x] Task 3: CLI Integration (AC: 2)
  - [x] Add `--resume-by-checksum` flag to `sync` command.
- [x] Task 4: Validation (AC: 1, 4)
  - [x] Integration tests for resuming transfers.

## Dev Notes

- Use `rusqlite` or `sqlx` (architecture didn't specify exactly, but `sqlx` with `sqlite` is good for async).
- Actually, for a simple CLI tool, `rusqlite` is often easier but sync. `sqlx` is better for integration with `tokio`.
- Let's check if `sqlx` is in `Cargo.toml`.
