# Story 2.4: Transfer Rate Throttling

Status: done

## Story

As a network administrator,
I want to limit the bandwidth consumed by the migration tool,
So that I don't saturate the site-to-site VPN or affect other production services.

## Acceptance Criteria

1. **Given** a high-bandwidth migration job
2. **When** `jfrog2nexus sync --max-kbps 1000` is executed
3. **Then** the total egress bandwidth across all worker threads combined does not exceed the specified limit (1000 KB/s in this case).

## Tasks / Subtasks

- [x] Task 1: Global Rate Limiter Implementation (AC: 3)
  - [x] Use `governor` crate or similar to implement a global token bucket.
- [x] Task 2: Integrate into Streaming Engine (AC: 3)
  - [x] Apply rate limiting to the `HashingStream` or a dedicated `ThrottledStream`.
- [x] Task 3: CLI Integration (AC: 2)
  - [x] Add `--max-kbps` flag to `sync` command.
- [x] Task 4: Validation (AC: 1, 3)
  - [x] Performance tests or unit tests with timing assertions.

## Dev Notes

- Use `governor`.
- We already have `HashingStream`. We can add a `ThrottledStream` or integrate into it.
- Rate limiting should be global across all concurrent transfers.
