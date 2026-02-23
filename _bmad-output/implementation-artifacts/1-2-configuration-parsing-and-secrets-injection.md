# Story 1.2: Configuration Parsing and Secrets Injection

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As an operations engineer,
I want to define my mappings in a YAML file and provide credentials via environment variables,
so that I can securely configure the migration tool.

## Acceptance Criteria

1. **Given** a valid `j2n.yaml` with proxy endpoints and repository mappings
2. **When** I run `jfrog2nexus config validate` with `J2N_JFROG_TOKEN` and `J2N_NEXUS_TOKEN` environment variables set
3. **Then** the system parses the configuration into strongly typed Rust structs
4. **And** it exits with `0` without printing secrets to stdout.

## Tasks / Subtasks

- [x] Task 1: Define Configuration Models (AC: 1, 3)
  - [x] Implement `AppConfig` and related structs (RepositoryMapping, ProxyConfig) in `src/config/mod.rs`.
  - [x] Use `serde::Deserialize` for YAML mapping.
- [x] Task 2: Implement Secrets Injection (AC: 2, 4)
  - [x] Bind `J2N_JFROG_TOKEN` and `J2N_NEXUS_TOKEN` environment variables to the configuration.
  - [x] **Security Enhancement:** Recommend using the `secrecy` crate for token fields to prevent accidental exposure in `Debug` or `tracing` output.
- [x] Task 3: Implement `config validate` Command (AC: 3, 4)
  - [x] Update `src/cli/commands.rs` to handle the `config validate` logic.
  - [x] Use `serde_yaml` to parse the file at the provided path.
  - [x] **Validation Logic:** Verify that URLs are valid `url::Url` types and that tokens are not empty.
  - [x] Exit with `0` on successful parse and validation, or an `anyhow` error on failure.
- [x] Task 4: Unit Testing (AC: 3)
  - [x] Add tests in `src/config/mod.rs` for valid and invalid YAML configurations.
  - [x] Mock environment variables in tests to ensure proper injection.

## Dev Notes

### Technical Requirements
- **Env Vars:** MUST be prefixed with `J2N_`.
- **Secrets:** MUST NOT be printed to stdout or logs.
- **Error Handling:** Use `anyhow::Result` in `src/cli/commands.rs`.

### Architecture Compliance
- **Configuration Naming:** Base config file should be `j2n.yaml` (default). [Source: architecture.md#Naming Patterns]
- **Application Architecture:** Declarative YAML parsing using `serde_yaml`. [Source: architecture.md#Application Architecture]
- **Code Organization:** Models in `src/config/mod.rs`, execution in `src/cli/commands.rs`.

### Library / Framework Requirements
- `serde` / `serde_yaml` (v0.9+)
- `clap` (v4.5+) with `env` feature.

### Project Structure Notes
- Alignment with `src/config/` for all deserialization logic.
- `src/cli/commands.rs` delegates parsing to the config module.

### References
- PRD requirements: FR1, FR2, FR3.
- Architecture Decisions: [Source: architecture.md#Data Architecture]

## Dev Agent Record

### Agent Model Used

Antigravity

### Debug Log References

### Completion Notes List

- Implemented strongly-typed configuration models matching the PRD and Architecture requirements.
- Enforced security via `secrecy` crate for all API tokens.
- Bound `J2N_JFROG_TOKEN` and `J2N_NEXUS_TOKEN` as default values for configuration if omitted in YAML.
- Implemented `config validate` command that performs full deserialization and semantic validation (URL format, required tokens).
- Verified implementation with unit tests covering both valid config and missing environment secrets.
- Integrated `config validate` into the main CLI entry point.

### Review Follow-ups (AI)
- [x] [AI-Review][High] Flaky tests due to global env mutation. Fixed using `serial_test`. [src/config/mod.rs]
- [x] [AI-Review][Medium] Blocking I/O in async context. Replaced `std::fs` with `tokio::fs`. [src/config/mod.rs]
- [x] [AI-Review][Medium] Missing HTTPS protocol enforcement. Added scheme check in `validate()`. [src/config/mod.rs]
- [x] [AI-Review][Medium] Deprecated `serde_yaml`. Swapped for `serde_yml`. [Cargo.toml, src/config/mod.rs]
- [x] [AI-Review][Low] Redundant imports and unvalidated empty mappings. Fixed. [src/config/mod.rs]

### File List
- Cargo.toml
- src/config/mod.rs
- src/main.rs
- j2n.yaml (created for testing)
