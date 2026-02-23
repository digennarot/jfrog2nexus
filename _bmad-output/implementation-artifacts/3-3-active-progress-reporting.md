# Story 3.3: Active Progress Reporting

Status: done

## Story

As a lead migration engineer,
I want to poll the active progress of my migration,
So that I understand how many mapped repositories remain to be processed.

## Acceptance Criteria

1. **Given** an active background migration running via cron
2. **When** I execute `jfrog2nexus status`
3. **Then** the tool queries local state or metrics to print a summary of processed vs remaining artifacts
4. **And** estimates transfer completion time based on current throughput.

## Tasks / Subtasks

- [x] Task 1: Status Command Implementation (AC: 2)
  - [x] Add `status` command to CLI.
- [x] Task 2: State Store Stats (AC: 3)
  - [x] Add method to `StateStore` to count completed artifacts.
- [x] Task 3: Metrics Polling (AC: 3, 4)
  - [x] Query the Prometheus metrics server if available to show connectivity.
- [x] Task 4: Progress Estimation (AC: 4)
  - [x] Calculate total data migrated.

## Dev Notes

- `jfrog2nexus status` provides a snapshot of the `.j2n/state.db`.
- It also check the metrics server to see if a migration is live.
