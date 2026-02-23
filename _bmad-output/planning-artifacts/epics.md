---
stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-03-create-stories']
inputDocuments: ['_bmad-output/planning-artifacts/prd.md', '_bmad-output/planning-artifacts/architecture.md']
---

# jfrog2nexus - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for jfrog2nexus, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

- **FR1:** The system can parse a declarative YAML configuration file defining source and target repository mappings.
- **FR2:** The system can parse proxy endpoints and routing rules from the YAML configuration.
- **FR3:** The system can read authentication secrets exclusively from environment variables.
- **FR4:** The system can validate YAML configuration syntax and upstream connectivity without initiating data transfer.
- **FR5:** The system can authenticate to JFrog Artifactory via HTTPS API.
- **FR6:** The system can authenticate to Sonatype Nexus via HTTPS API.
- **FR7:** The system can traverse and list Docker and Maven artifacts within a source repository.
- **FR8:** The system can transfer artifacts from source to target repository.
- **FR9:** The system can execute a dry-run simulation that maps artifacts and validates connectivity without transferring data.
- **FR10:** The system can calculate and verify SHA256 checksums of artifacts on both source and target servers.
- **FR11:** The system can resume an interrupted transfer by comparing checksums and skipping identical destination files.
- **FR12:** The system can restrict transfer bandwidth based on a user-defined threshold limit.
- **FR13:** The system can implement connection pooling and retry logic with exponential backoff on API timeouts and 503 errors.
- **FR14:** The system can output structured JSON logs detailing operational events and errors to standard output.
- **FR15:** The system can generate a progress report detailing active migration state across mappings.
- **FR16:** The system can expose a `/metrics` HTTP endpoint serving Prometheus-compatible telemetry.
- **FR17:** The system can generate a CSV audit report containing pre- and post-transfer SHA256 hashes.
- **FR18:** The system can provide shell autocompletion definitions for `bash`, `zsh`, and `fish`.
- **FR19:** The system can provide command-line documentation describing available commands and flags.

### NonFunctional Requirements

- **NFR1:** The system shall sustain a transfer rate of 95MB/s on a gigabit network without throttling as measured by OS network metrics.
- **NFR2:** The system shall maintain a memory footprint under 512MB during continuous 100GB+ artifact transfers as measured by process monitoring.
- **NFR3:** The system shall utilize less than 5% CPU on a standard dual-core virtual machine during active transfers as measured by process monitoring.
- **NFR4:** The system shall restrict network bandwidth to the user-defined limit within 2 seconds of transfer initiation as measured by network metrics.
- **NFR5:** The system shall automatically resume an interrupted data transfer without payload corruption upon the next execution as measured by SHA256 validation.
- **NFR6:** The system shall achieve a proxy cache hit rate exceeding 95% for Docker layer blobs as measured by proxy access logs.
- **NFR7:** The system shall transmit all data exclusively via HTTPS/TLS 1.2+ protocols as measured by network inspection.
- **NFR8:** The system shall never output environment variable secrets to `stdout` or log files as measured by log auditing.
- **NFR9:** The system shall maintain a 0% false-positive rate for data corruption detection during post-transfer SHA256 validation as measured by integration testing.

### Additional Requirements

- **Starter Template**: Architecture specifies initializing via `cargo new jfrog2nexus --bin` and adding standard crates (`tokio`, `clap`, `reqwest`, `serde`, `serde_yaml`, `tracing`, `tracing-subscriber`, `anyhow`, `thiserror`). **This impacts Epic 1 Story 1.**
- **Infrastructure**: Must execute on strictly constrained environments like PikaOS standard VMs without interactive prompts.
- **Security**: Environment variable secrets injection (`J2N_` prefix). No secrets in output.
- **Communication Patterns**: Strict memory streaming pattern (`reqwest::Response::bytes_stream()` paired with `tokio_util::io::StreamReader`) and continuous hashing via `sha2::Digest::update`.
- **Project Structure**: Strict boundary enforcement between CLI (`src/cli/`), Engine (`src/engine/`), Observability (`src/observability/`), and Audit (`src/audit/`).
- **Error Handling**: All app-level functions return `anyhow::Result<T>`, core logic returns typed `thiserror` enums. No `.unwrap()`.

### FR Coverage Map

FR1: Epic 1 - The system can parse a declarative YAML configuration file defining source and target repository mappings.
FR2: Epic 1 - The system can parse proxy endpoints and routing rules from the YAML configuration.
FR3: Epic 1 - The system can read authentication secrets exclusively from environment variables.
FR4: Epic 1 - The system can validate YAML configuration syntax and upstream connectivity without initiating data transfer.
FR5: Epic 1 - The system can authenticate to JFrog Artifactory via HTTPS API.
FR6: Epic 1 - The system can authenticate to Sonatype Nexus via HTTPS API.
FR7: Epic 1 - The system can traverse and list Docker and Maven artifacts within a source repository.
FR8: Epic 1 - The system can transfer artifacts from source to target repository.
FR9: Epic 1 - The system can execute a dry-run simulation that maps artifacts and validates connectivity without transferring data.
FR10: Epic 2 - The system can calculate and verify SHA256 checksums of artifacts on both source and target servers.
FR11: Epic 2 - The system can resume an interrupted transfer by comparing checksums and skipping identical destination files.
FR12: Epic 2 - The system can restrict transfer bandwidth based on a user-defined threshold limit.
FR13: Epic 2 - The system can implement connection pooling and retry logic with exponential backoff on API timeouts and 503 errors.
FR14: Epic 3 - The system can output structured JSON logs detailing operational events and errors to standard output.
FR15: Epic 3 - The system can generate a progress report detailing active migration state across mappings.
FR16: Epic 3 - The system can expose a `/metrics` HTTP endpoint serving Prometheus-compatible telemetry.
FR17: Epic 3 - The system can generate a CSV audit report containing pre- and post-transfer SHA256 hashes.
FR18: Epic 4 - The system can provide shell autocompletion definitions for `bash`, `zsh`, and `fish`.
FR19: Epic 4 - The system can provide command-line documentation describing available commands and flags.

## Epic List

### Epic 1: Core Migration Execution
Users can define repository mappings, validate connectivity (dry-run), and successfully transfer Docker and Maven artifacts.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR8, FR9

### Story 1.1: Project Initialization and CLI Scaffolding

As a developer,
I want the Rust project properly scaffolded with `clap` and core directories,
So that I have a foundation to build the CLI commands.

**Acceptance Criteria:**

**Given** an empty repository
**When** the project is initialized with `cargo new --bin` and `clap` is configured for a non-interactive CLI
**Then** the directory structure contains `src/cli/`, `src/engine/` etc.
**And** the CLI accepts a `--help` flag and prints basic usage for a `sync` and `config validate` command (even if they do nothing yet).

### Story 1.2: Configuration Parsing and Secrets Injection

As an operations engineer,
I want to define my mappings in a YAML file and provide credentials via environment variables,
So that I can securely configure the migration tool.

**Acceptance Criteria:**

**Given** a valid `j2n.yaml` with proxy endpoints and repository mappings
**When** I run `jfrog2nexus config validate` with `J2N_JFROG_TOKEN` and `J2N_NEXUS_TOKEN` environment variables set
**Then** the system parses the configuration into strongly typed Rust structs
**And** it exits with `0` without printing secrets to stdout.

### Story 1.3: Upstream Connectivity and Authentication Validation

As an operations engineer,
I want the tool to independently verify it can reach both JFrog and Nexus,
So that I know my configuration and network are sound before starting a transfer.

**Acceptance Criteria:**

**Given** a valid configuration and network connection
**When** I run `jfrog2nexus config validate`
**Then** the system authenticates to both the JFrog and Nexus APIs
**And** returns a successful connectivity check message to stdout.

### Story 1.4: Artifact Traversal and Dry-Run Execution

As an operations engineer,
I want to simulate a transfer to map all artifacts without moving data,
So that I can safely verify what will be migrated.

**Acceptance Criteria:**

**Given** a source repository with Docker and Maven artifacts
**When** I execute `jfrog2nexus sync --dry-run`
**Then** the tool queries the JFrog API using pagination tokens to recursively list all artifacts matching the mapping
**And** prints a simulated plan of what would be transferred, without downloading any bytes.

### Story 1.5: Core Streaming Transfer Engine

As an operations engineer,
I want the tool to transfer artifacts from the source to the target repository,
So that my data is successfully migrated.

**Acceptance Criteria:**

**Given** a successful dry-run plan
**When** I execute `jfrog2nexus sync` (without dry-run)
**Then** the system streams the artifact bytes directly from JFrog to Nexus using a bounded `tokio::spawn` worker pool (max 50 concurrent)
**And** dynamically uses the correct target API mechanism (Docker v2 push vs Maven PUT) based on the repository mapping.

### Epic 2: Transfer Resilience & Throttling
Operations teams can safely migrate large-scale artifacts across unstable enterprise networks with resumable, checksum-validated transfers and strict bandwidth limits.
**FRs covered:** FR10, FR11, FR12, FR13

### Story 2.1: Streaming Checksum Calculation

As an operations engineer,
I want the tool to cryptographically verify data integrity during transfers,
So that I can be 100% certain my artifacts are not corrupted.

**Acceptance Criteria:**

**Given** an active artifact transfer from `jfrog2nexus sync`
**When** the data streams to the destination
**Then** the system concurrently calculates a SHA256 hash using memory-safe chunking
**And** validates the final hash against the source metadata
**And** automatically deletes the target file and requeues the transfer if a mismatch is detected.

### Story 2.2: Resumable Transfers via Checksum Matching

As the lead migration engineer,
I want the sync process to skip artifacts that are already fully transferred,
So that network interruptions don't force me to restart massive multi-GB file downloads from zero.

**Acceptance Criteria:**

**Given** a partially completed migration where some artifacts exist on the target
**When** `jfrog2nexus sync --resume-by-checksum` is executed
**Then** the system queries the local `.j2n/state.db` SQLite database to compare checksums
**And** skips identical files without requiring remote target API validation, downloading only missing artifacts.

### Story 2.3: Connection Pooling and Exponential Backoff Retries

As a platform admin,
I want the tool to gracefully handle 503 proxy errors and timeouts,
So that transient network drops don't fail long-running cron jobs.

**Acceptance Criteria:**

**Given** an unstable network connection
**When** the `reqwest` client encounters a timeout, 503, or 504 error during a transfer
**Then** the engine automatically retries the failing chunk using an exponential backoff strategy
**And** continues the transfer once connectivity returns.

### Story 2.4: Transfer Rate Throttling

As a platform admin,
I want to restrict the CLI's bandwidth usage,
So that massive automated migrations don't saturate our corporate firewalls or VPN links.

**Acceptance Criteria:**

**Given** a running `jfrog2nexus sync` operation
**When** the user provides the `--throttle=<limit_mb_s>` flag
**Then** the async stream processor restricts the I/O bytes read
**And** the total network bandwidth consumed across the entire global token bucket drops to the specified limit within 2 seconds.

### Story 2.5: Dynamic Token Refresh

As an operations engineer,
I want the tool to refresh API tokens if they expire during a multi-day transfer,
So that massive migrations don't fail halfway through.

**Acceptance Criteria:**

**Given** an active migration that exceeds the initial API token's Time-To-Live (TTL)
**When** the target API returns a `401 Unauthorized` mid-transfer
**Then** the system intercepts the error, attempts a generic token refresh routine (or prompts re-evaluation of env vars),
**And** resumes the transfer pool automatically.

### Epic 3: Observability & Compliance Auditing
Platform admins and security officers can monitor migration health in real-time via integrations (Prometheus) and export cryptographic proof of data integrity for compliance and sign-off.
**FRs covered:** FR14, FR15, FR16, FR17

### Story 3.1: JSON Structured Logging

As a platform admin,
I want all application output formatted as structured JSON,
So that I can ingest the logs into ElasticSearch or Datadog without custom parsing rules.

**Acceptance Criteria:**

**Given** an execution of the CLI
**When** application events or errors occur
**Then** the output to `stdout` is formatted strictly as JSON using the `tracing-subscriber` create
**And** standard macros like `println!` are entirely avoided.

### Story 3.2: Prometheus Metrics Server

As a platform admin,
I want a `/metrics` HTTP endpoint serving real-time telemetry,
So that I can set up Grafana alerts for cache hit rates and transfer speeds.

**Acceptance Criteria:**

**Given** the CLI is actively syncing artifacts
**When** I curl `http://localhost:9090/metrics`
**Then** the system returns a Prometheus-compatible text payload via an internal `axum` server bound explicitly to `127.0.0.1` (no generic `0.0.0.0` exposure)
**And** the payload includes counters for `j2n_transfer_bytes_total` and HTTP status codes.

### Story 3.3: Active Progress Reporting

As a lead migration engineer,
I want to poll the active progress of my migration,
So that I understand how many mapped repositories remain to be processed.

**Acceptance Criteria:**

**Given** an active background migration running via cron
**When** I execute `jfrog2nexus status`
**Then** the tool queries local state or metrics to print a summary of processed vs remaining artifacts
**And** estimates transfer completion time based on current throughput.

### Story 3.4: Compliance Audit Report Generation

As a security officer,
I want a consolidated summary of pre- and post-transfer cryptographic hashes,
So that I have auditable proof to decommission the legacy Artifactory server safely.

**Acceptance Criteria:**

**Given** a completed migration
**When** I execute `jfrog2nexus report generate`
**Then** the tool generates a `.csv` file detailing every artifact path, the Artifactory SHA256, and the Nexus SHA256
**And** gracefully errors with a clear message if the execution environment lacks write permissions for the out directory.

### Epic 4: Developer Experience & CLI Polish
CI/CD integrators and engineers can easily discover tool capabilities, get shell hints, and automate execution safely.
**FRs covered:** FR18, FR19

### Story 4.1: Shell Autocompletion Generation

As a developer or operations engineer,
I want the CLI to provide tab-completion for shell environments,
So that I can quickly construct valid commands without referencing the manual.

**Acceptance Criteria:**

**Given** the compiled `jfrog2nexus` binary
**When** I execute `jfrog2nexus generate-completions [bash/zsh/fish]`
**Then** the tool outputs standard shell completion scripts derived directly from the `clap` command definitions.

### Story 4.2: Comprehensive Command-Line Documentation

As an operations engineer,
I want built-in help text explaining flags and subcommands,
So that I understand how to format mapping arguments or timeout parameters correctly.

**Acceptance Criteria:**

**Given** a terminal session
**When** I execute `jfrog2nexus --help` or `jfrog2nexus sync --help`
**Then** the tool outputs detailed, human-readable instructions describing all available arguments, environment variables (`J2N_*`), and configuration paths.

