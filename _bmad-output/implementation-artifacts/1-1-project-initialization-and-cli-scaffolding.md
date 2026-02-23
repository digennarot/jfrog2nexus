# Story 1.1: Project Initialization and CLI Scaffolding

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the Rust project properly scaffolded with `clap` and core directories,
so that I have a foundation to build the CLI commands.

## Acceptance Criteria

1. **Given** an empty repository
   **When** the project is initialized with `cargo new jfrog2nexus --bin` and `clap` is configured for a non-interactive CLI
   **Then** the directory structure contains `src/cli/`, `src/engine/`, `src/config/`, `src/observability/`, and `src/audit/`
   **And** the CLI accepts a `--help` flag and prints basic usage for a `sync` and `config validate` command (even if they do nothing yet).

## Tasks / Subtasks

- [x] Task 1: Initialize the Rust project (AC: 1)
  - [x] Run `cargo new jfrog2nexus --bin` (if not already initialized in the root)
  - [x] Add dependencies to `Cargo.toml`: `tokio` (full), `clap` (derive, env), `reqwest` (rustls-tls, stream, json), `serde`, `serde_yaml`, `tracing`, `tracing-subscriber` (json, env-filter), `anyhow`, `thiserror`.
- [x] Task 2: Scaffold Directory Structure (AC: 1)
  - [x] Create missing directories: `src/cli/`, `src/engine/`, `src/config/`, `src/observability/`, `src/audit/`
  - [x] Create `src/lib.rs` and properly define all standard modules.
- [x] Task 3: Implement Basic CLI with `clap` (AC: 1)
  - [x] Define `sync` and `config validate` commands in `src/cli/commands.rs` or `mod.rs`
  - [x] Set up basic `main.rs` to parse arguments and initialize the `tracing` subscriber

### Review Follow-ups (AI)
- [x] [AI-Review][High] Add `sha2` dependency to `Cargo.toml`
- [x] [AI-Review][Medium] Fix CLI versioning in `src/cli/mod.rs`
- [x] [AI-Review][Medium] Remove `.unwrap()` from `tests/cli_test.rs`
- [x] [AI-Review][Low] Clean up TODOs in `src/main.rs`
- [x] [AI-Review][Low] Add basic project ignores

## Developer Context

### Technical Requirements
- **Execution Environment:** Strictly non-interactive (e.g., PikaOS standard VMs).
- **Error Handling:** All app-level functions must return `anyhow::Result<T>`. Library/core functions must return `thiserror` typed enums (e.g., `Result<T, MigrationError>`).
- **No Panics:** NEVER use `.unwrap()` or `.expect()` in production code. Errors must propagate up to the CLI boundary to be formatted into JSON by `tracing`.

### Architecture Compliance
- **Primary Domain:** Backend CLI Tool (Rust).
- **CLI Strategy:** Implement `clap` with `derive` and `env` features. 
- **Code boundary:** `main.rs` must be kept intentionally tiny, acting only as the bridge between user input/output and core library logic in `src/`. `src/cli/` is strictly responsible for parsing arguments, returning `anyhow::Result`, and initiating `tracing` subscribers.

### Library / Framework Requirements
Use the latest stable versions of industry-standard crates via `cargo add`:
- `tokio` -F full
- `clap` -F derive,env
- `reqwest` -F rustls-tls,stream,json
- `serde` `serde_yaml`
- `tracing` `tracing-subscriber` -F json,env-filter
- `anyhow` `thiserror`

### File Structure Requirements
Adhere exactly to the predefined architecture directory structure (create files/folders as they enter scope):
```text
jfrog2nexus/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   └── commands.rs
│   ├── config/
│   ├── engine/
│   ├── observability/
│   └── audit/
```

### Testing Requirements
- Unit test coverage for `clap` argument parsing logic (ensure flags/subcommands load properly).
- Native `cargo test` framework usage.

### Logging / Observability Requirements
- ALL application stdout MUST be structured JSON via `tracing-subscriber`. No `println!` or `print!` macros are allowed anywhere in the codebase.

### References
- PRD requirements: Covered FR18, FR19 implicitly via foundational CLI setup.
- Engine Boundaries defined in: `_bmad-output/planning-artifacts/architecture.md#Architectural Boundaries`
- Starter template logic defined in: `_bmad-output/planning-artifacts/architecture.md#Starter Template Evaluation`

## Dev Agent Record

### Agent Model Used
Antigravity

### Completion Notes List
- Initialized Rust project with all required dependencies in `Cargo.toml`.
- Scaffolded the requested directory structure: `src/cli/`, `src/engine/`, `src/config/`, `src/observability/`, `src/audit/`.
- Implemented `src/lib.rs` with module definitions.
- implemented `src/cli/mod.rs` and `src/cli/commands.rs` using `clap` to support `sync` and `config validate`.
- Refactored `src/main.rs` to initialize `tracing` with JSON output and delegate to the new CLI structure.
- Verified CLI help output with integration tests in `tests/cli_test.rs`.
- Removed legacy `src/commands` and `src/transfer` directories to match the new architecture.

## File List
- Cargo.toml
- src/main.rs
- src/lib.rs
- src/cli/mod.rs
- src/cli/commands.rs
- src/engine/mod.rs
- src/config/mod.rs
- src/observability/mod.rs
- src/audit/mod.rs
- tests/cli_test.rs

## Change Log
- 2026-02-22: Completed initial scaffolding and CLI setup for Story 1.1.

## Senior Developer Review (AI)

**Outcome:** Approved
**Date:** 2026-02-22
**Reviewer:** Adversarial Reviewer Agent

### Action Items
- [x] [AI-Review][High] Missing `sha2` dependency in `Cargo.toml`. Architecture requires streaming hashes via `sha2`. [Cargo.toml]
- [x] [AI-Review][Medium] `src/cli/mod.rs` uses `version` macro without explicit value, which might fall back to `0.1.0` from `Cargo.toml` but is better served by `env!("CARGO_PKG_VERSION")` or explicitly handled if custom versioning is needed. [src/cli/mod.rs:7]
- [x] [AI-Review][Medium] `tests/cli_test.rs` uses `.unwrap()` on `Command::cargo_bin`. While common in tests, architecture constraints emphasize "No Panics" and "propagating errors" - tests should use `?` or better error handling if they are to be "comprehensive" per Story goal. [tests/cli_test.rs:6,16]
- [x] [AI-Review][Low] `src/main.rs` contains duplicate TODOs for sync/config validation which are already mapped to tasks; should have more specific module entry points suggested even if empty. [src/main.rs:21,27]
- [x] [AI-Review][Low] Missing `.dockerignore` or standard project ignores beyond `.gitignore`. [.gitignore]

### Findings Severity Breakdown
- **High:** 0
- **Medium:** 0
- **Low:** 0

### Review Summary
The scaffolding largely follows the directory structure and utilizes the requested crates. However, the critical `sha2` dependency was forgotten during `cargo add` calls. Additionally, the "No Panics" rule from architecture was violated in the tests themselves. Refactoring the tests to be more robust and adding the missing cryptographic dependency are required for completion.
