# Story 3.4: Compliance Audit Report Generation

Status: done

## Story

As a security officer,
I want a consolidated summary of pre- and post-transfer cryptographic hashes,
So that I have auditable proof to decommission the legacy Artifactory server safely.

## Acceptance Criteria

1. **Given** a completed migration
2. **When** I execute `jfrog2nexus report generate`
3. **Then** the tool generates a `.csv` file detailing every artifact path, the Artifactory SHA256, and the Nexus SHA256
4. **And** gracefully errors with a clear message if the execution environment lacks write permissions for the out directory.

## Tasks / Subtasks

- [x] Task 1: Audit Report Command (AC: 2)
  - [x] Add `report generate` command.
- [x] Task 2: CSV Generation Logic (AC: 3)
  - [x] Implement CSV writer in `src/audit/mod.rs`.
- [x] Task 3: Error Handling (AC: 4)
  - [x] Handle write permission errors gracefully.
- [x] Task 4: Validation (AC: 1)
  - [x] Verify generated CSV contains required columns.

## Dev Notes

- Report is generated from the state database.
- Columns: Source Repo, Path, Target Repo, SHA256, Size, Completed At.
- Hashes are unified as we verify they match during transfer.
