# Data Models

## Overview
The application uses SQLite via `sqlx` to maintain local state, audit logs, and operational data.

## Key Entities (Inferred)

### `sync_state`
- Stores progression of artifact synchronization.
- **Fields**: `id`, `artifact_path`, `source_repo`, `target_repo`, `status` (pending/success/failed), `last_updated`, `error_message`.

### `audit_logs`
- Stores operational events for compliance.
- **Fields**: `id`, `timestamp`, `event_type`, `description`, `actor`.

### `user_permissions` (via RBAC)
- Admin-oriented access controls if exposed as a service.
- **Fields**: `user_id`, `role`, `scopes`.

*(These are derived functionally; the concrete database schema lives in `migrations/` or inline SQLx queries within the `*_repo` modules under `src/`.)*
