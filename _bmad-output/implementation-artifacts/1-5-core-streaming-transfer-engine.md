# Story 1.5: Core Streaming Transfer Engine

Status: done

## Story

As an operations engineer,
I want the tool to transfer artifacts from the source to the target repository,
So that my data is successfully migrated.

## Acceptance Criteria

1. **Given** a successful dry-run plan
2. **When** I execute `jfrog2nexus sync` (without dry-run)
3. **Then** the system streams the artifact bytes directly from JFrog to Nexus using a bounded `tokio::spawn` worker pool (max 50 concurrent)
4. **And** dynamically uses the correct target API mechanism (Docker v2 push vs Maven PUT) based on the repository mapping.

## Tasks / Subtasks

- [x] Task 1: Skeleton of the Transfer Engine (AC: 3)
  - [x] Define `TransferOrchestrator` in `src/engine/transfer.rs`.
  - [x] Implement a bounded worker pool (using `semaphore`) to limit concurrency to 50.
- [x] Task 2: Implement Streaming Transfer (AC: 3)
  - [x] Implement direct streaming from JFrog (GET) to Nexus (PUT) using `reqwest::Body::wrap_stream`.
  - [x] Ensure memory consumption is bounded (no `bytes()`/`collect()`).
- [x] Task 3: Repository Type Adapters (AC: 4)
  - [x] Handle Maven via standard PUT.
  - [x] Handle Docker via v2 push mechanism (POST initiate -> PUT blob).
- [x] Task 4: Integrate into `sync` Command (AC: 2)
  - [x] Update `main.rs` to call the transfer engine when not in dry-run mode.
- [x] Task 5: Validation (AC: 1, 3, 4)
  - [x] Unit tests for the transfer logic.


## Dev Notes

### Technical Requirements
- Use `reqwest::Response::bytes_stream()`.
- Use `reqwest::Body::wrap_stream(stream)` for the upload part.
- For Docker, we might need to handle the specific handshake (POST initiate, then upload).

### Architecture Compliance
- Memory < 512MB: Mandatory streaming.
- CPU < 5%: Efficient async I/O.
- HTTPS only: Enforced in config, but ensure engine respects it.

### References
- PRD: FR8.
- Architecture: [Source: architecture.md#The Engine]
