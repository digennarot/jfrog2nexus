# Story 2.5: Dynamic Token Refresh

Status: done

## Story

As an operations engineer,
I want the tool to refresh API tokens if they expire during a multi-day transfer,
So that massive migrations don't fail halfway through.

## Acceptance Criteria

1. **Given** an active migration that exceeds the initial API token's Time-To-Live (TTL)
2. **When** the target API returns a `401 Unauthorized` mid-transfer
3. **Then** the system intercepts the error, attempts a generic token refresh routine (or prompts re-evaluation of env vars),
4. **And** resumes the transfer pool automatically.

## Tasks / Subtasks

- [x] Task 1: Robust Token Provider (AC: 3)
  - [x] Added `token_file` support to `JfrogConfig` and `NexusConfig` for Linux process isolation workaround.
- [x] Task 2: Intercept 401 Errors with Thundering Herd Protection (AC: 2, 3)
  - [x] Detect 401 status codes.
  - [x] Use a global `refresh_lock` (Mutex) and `last_refresh` timestamp (cooldown) to ensure only one refresh happens at a time.
- [x] Task 3: Validation (AC: 1, 4)
  - [x] Mock test `test_transfer_unauthorized_retry_success` verifies the 401 -> Refresh -> Success flow.

## Dev Notes

- **Linux Compatibility**: Standard environment variables are immutable at runtime. We now support an optional `token_file` that can be updated by external automation (e.g., a sidecar).
- **Concurrency**: Introduced `refresh_lock` and a 10-second cooldown per repository to prevent thundering herd when many tasks hit 401 at the same time.
- **Observability**: Added path and repository type to refresh logs for better debugging.
