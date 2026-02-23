# Story 2.3: Connection Pooling and Exponential Backoff Retries

Status: done

## Story

As an operations engineer,
I want the tool to be resilient against transient network failures,
So that minor blips don't cause the entire migration to fail.

## Acceptance Criteria

1. **Given** a transient network failure during transfer
2. **When** the request fails with a 5xx or connection error
3. **Then** the transfer engine automatically retries up to 5 times
4. **And** uses exponential backoff (starting at 1s, doubling each time)
5. **And** optimizes throughput by reusing HTTP connections via persistent pooling.

## Tasks / Subtasks

- [x] Task 1: Configure Connection Pooling (AC: 5)
  - [x] Ensure `reqwest::Client` is reused and configured for persistent connections.
- [x] Task 2: Implement Exponential Backoff Retry Loop (AC: 1, 2, 3, 4)
  - [x] Implement a helper for retrying async operations.
  - [x] Apply retry logic to JFrog GET and Nexus PUT/POST calls.
- [x] Task 3: Validation (AC: 3, 4)
  - [x] Test retry behavior with mocked transient failures.

## Dev Notes

- Max retries: 5.
- Initial delay: 1s.
- Backoff: factor of 2.
