# Story 1.3: Upstream Connectivity and Authentication Validation

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As an operations engineer,
I want the tool to independently verify it can reach both JFrog and Nexus,
so that I know my configuration and network are sound before starting a transfer.

## Acceptance Criteria

1. **Given** a valid configuration and network connection
2. **When** I run `jfrog2nexus config validate`
3. **Then** the system authenticates to both the JFrog and Nexus APIs
4. **And** returns a successful connectivity check message to stdout.

## Tasks / Subtasks

- [x] Task 1: Implement JFrog Connectivity Check (AC: 1, 3)
  - [x] Implement `check_connectivity` method for `JfrogConfig` in `src/config/mod.rs` (or a dedicated client module).
  - [x] Use `reqwest` to call a lightweight endpoint (e.g., `/api/system/ping`).
- [x] Task 2: Implement Nexus Connectivity Check (AC: 1, 3)
  - [x] Implement `check_connectivity` method for `NexusConfig`.
  - [x] Use `reqwest` to call a lightweight endpoint (e.g., `/service/rest/v1/status`).
- [x] Task 3: Integrate Connectivity Checks into `config validate` (AC: 2, 3, 4)
  - [x] Update `load_config` or the CLI handler to call these checks after parsing.
  - [x] Ensure errors from connectivity checks are reported clearly.
- [x] Task 4: Integration Testing (AC: 3)
  - [x] Mock the JFrog and Nexus APIs using `wiremock` to test connectivity success/failure scenarios.

## Dev Notes

### Technical Requirements
- **Authentication:** Use the provided tokens in headers.
- **Async I/O:** All network calls must be async.
- **Client Configuration:** Use a shared `reqwest::Client` if possible.

### Architecture Compliance
- **Communication Patterns:** Multi-tenant connection pooling (managed by `reqwest::Client`).
- **Error Handling:** Use `thiserror` for client-specific errors if necessary, but `anyhow` for top-level.

### Library / Framework Requirements
- `reqwest` (v0.12+)
- `wiremock` (for testing)

### Project Structure Notes
- Interaction logic should probably move to a new `src/client/` or `src/engine/` module if it gets complex.

### References
- PRD: FR4, FR5, FR6.
- Architecture: [Source: architecture.md#Network Architecture]
