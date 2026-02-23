# Story 2.1: Streaming Checksum Calculation

Status: done

## Story

As an operations engineer,
I want the tool to cryptographically verify data integrity during transfers,
So that I can be 100% certain my artifacts are not corrupted.

## Acceptance Criteria

1. **Given** an active artifact transfer from `jfrog2nexus sync`
2. **When** the data streams to the destination
3. **Then** the system concurrently calculates a SHA256 hash using memory-safe chunking
4. **And** validates the final hash against the source metadata
5. **And** automatically deletes the target file and requeues the transfer if a mismatch is detected.

## Tasks / Subtasks

- [x] Task 1: Implement Streaming Hasher (AC: 3)
  - [x] Use `sha2` crate to calculate hash over the byte stream.
  - [x] Integrate with `reqwest::Body::wrap_stream`.
- [x] Task 2: Hash Validation (AC: 4)
  - [x] Compare calculated hash with `artifact.sha256`.
- [x] Task 3: Error Handling and Cleanup (AC: 5)
  - [x] If hash mismatch, delete the target artifact.
  - [x] Requeue or retry the transfer.
- [x] Task 4: Integration (AC: 1, 2)
  - [x] Update `TransferOrchestrator` to use the hasher.

## Dev Notes

- Use `sha2::Sha256`.
- We need a way to tee the stream or calculate hash while reading.
- For `requeue`, we might need to update the `execute_plan` loop.
