# Architecture Overview

## Executive Summary
This document provides an overview of the architecture for `jfrog2nexus`, a monolithic Rust application designed to synchronize artifacts between JFrog Artifactory and Sonatype Nexus. It employs a CLI interface while building out backend services for operation.

## Technology Stack
- **Language**: Rust
- **Async Runtime**: Tokio
- **Web Framework**: Axum
- **HTTP Client**: Reqwest
- **CLI**: Clap
- **Database**: SQLx (SQLite)
- **Serialization**: Serde
- **Observability**: Tracing

## Architecture Pattern
The project utilizes a hybrid *Command-Line Interface* and *Service-Oriented Backend* pattern. The core logic executes procedurally via CLI commands, while internal state management and concurrent processing leverage robust backend paradigms (like `axum` and `sqlx`).

## Component Overview
- **CLI Adapter (`src/cli/`)**: Acts as the main entry point for user interaction, parsing arguments and dictating flow.
- **Sync Engine (`src/engine/`)**: The core domain logic that orchestrates API calls between JFrog (source) and Nexus (target).
- **Repositories (`src/admin_repo/`, `src/auth_repo/`, etc.)**: Data access layer abstracted over Redb/SQLx for persistence.
- **API Handlers (`src/api/handlers/`)**: Axum-based endpoints providing external observability or control.

## Data Architecture
Data is persisted locally using SQLite via `sqlx`. The models largely revolve around synchronization state, tracking which artifacts have been processed, user permissions, and audit logs.

*(Data models are fully detailed in `data-models-root.md`)*

## Testing Strategy
- **Unit Tests**: Handled inline within Rust modules (`#[cfg(test)]`).
- **Integration Tests**: Located under `tests/integration_test.rs`, these run against spun-up Docker containers representing JFrog and Nexus, using utilities from `tests/common/` and mock payloads from `tests/fixtures/`.
- **User Acceptance Tests**: Scripted E2E processes under `tests/uat/` to validate full user flows against a realistic environment configuration.
