# Story 3.1: JSON Structured Logging

Status: done

## Story

As a platform admin,
I want all application output formatted as structured JSON,
So that I can ingest the logs into ElasticSearch or Datadog without custom parsing rules.

## Acceptance Criteria

1. **Given** an execution of the CLI
2. **When** application events or errors occur
3. **Then** the output to `stdout` is formatted strictly as JSON using the `tracing-subscriber` create
4. **And** standard macros like `println!` are entirely avoided.

## Tasks / Subtasks

- [x] Task 1: Configure Tracing JSON Formatter (AC: 3)
  - [x] Use `tracing-subscriber` with `json()` output.
- [x] Task 2: Remove println! usage (AC: 4)
  - [x] Audit codebase for any `println!` or `eprintln!` and replace with `tracing` macros.
- [x] Task 3: Validation (AC: 1)
  - [x] Verify CLI output is valid JSON.

## Dev Notes

- Already initialized in `main.rs`.
- No `println!` remains in `src/`.
