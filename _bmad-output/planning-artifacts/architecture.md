---
stepsCompleted: ['step-01-init', 'step-02-context', 'step-03-starter', 'step-04-decisions', 'step-05-patterns', 'step-06-structure', 'step-07-validation', 'step-08-complete']
inputDocuments: ['_bmad-output/planning-artifacts/prd.md', '_bmad-output/brainstorming/brainstorming-session-2026-02-22.md']
workflowType: 'architecture'
lastStep: 8
status: 'complete'
completedAt: '2026-02-22'
project_name: 'jfrog2nexus'
user_name: 'tiziano_di_gennaro'
date: '2026-02-22'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
The tool is a non-interactive CLI focusing heavily on configuration parsing (YAML), robust network execution (connection pools, resumes, throttles, proxy routing), data validation (SHA256), and observability (JSON logging, CSV audits, Prometheus metrics).

**Non-Functional Requirements:**
Strict constraints on memory (<512MB) and CPU (<5%) while sustaining high throughput (95MB/s). It requires graceful network connection failure recovery and 100% data integrity validation.

**Scale & Complexity:**
Medium complexity, driven primarily by the scale of the artifacts (100GB+ blobs) and the need for resilient, memory-safe data streaming over enterprise network proxies.

- Primary domain: Backend CLI Tool (Rust)
- Complexity level: Medium
- Estimated architectural components: ~6 (Config Parser, CLI Core, Network Client, Stream Processor/Hasher, Metrics Server, Audit Logger)

### Technical Constraints & Dependencies

- Must be completely scriptable for cron/CI/CD environments (zero interactivity).
- Must execute on constrained environments like PikaOS standard VMs.
- Strict adherence to HTTPS/TLS 1.2+ and stateless execution between runs.

### Cross-Cutting Concerns Identified

- **Streaming & Memory Management**: Handling huge artifacts without RAM overruns.
- **Error Handling & Resilience**: Exponential backoffs, proxy awareness, and checksum resumption state.
- **Observability & Auditing**: Uniform JSON logging, CSV generation, and real-time Prometheus stat updates.

## Starter Template Evaluation

### Primary Technology Domain
Backend CLI Tool (Rust)

### Starter Options Considered
We evaluated generic CLI boilerplates (e.g., `ladvoc/rust-cli-template`, `rust-starter`) versus composing a purpose-built foundation using industry-standard crates.

### Selected Starter: Purpose-Built Rust CLI Foundation
**Rationale for Selection:**
Generic boilerplates often include unnecessary bloat or outdated crate versions. Given the strict CPU/Memory and zero-interactivity requirements of `jfrog2nexus`, composing a clean foundation using exact, current versions of standard libraries ensures maximum performance and minimal attack surface.

**Initialization Command:**
```bash
cargo new jfrog2nexus --bin
cargo add tokio -F full
cargo add clap -F derive,env
cargo add reqwest -F rustls-tls,stream,json
cargo add serde serde_yaml
cargo add tracing tracing-subscriber -F json,env-filter
cargo add anyhow thiserror
```

**Architectural Decisions Provided by Starter:**
- **Language & Runtime:** Rust (latest stable), `tokio` for async I/O.
- **Styling Solution:** N/A (Non-interactive data tool).
- **Build Tooling:** Standard `cargo` with potential future `cargo-dist` for cross-platform CI release automation.
- **Testing Framework:** Native `cargo test` with `wiremock` for HTTP mocking.
- **Code Organization:** Standard idiomatic Rust `src/main.rs` (CLI entry) and `src/lib.rs` (Core engine logically separated).
- **Development Experience:** Immediate support for JSON logging, environment variable parsing, and async networking.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Data Checksum/Hashing Approach
- Metrics Server Endpoint
- Audit Report Generation

**Important Decisions (Shape Architecture):**
- Asynchronous Runtime (`tokio`)
- HTTP Client (`reqwest`)

**Deferred Decisions (Post-MVP):**
- Advanced Repository Filtering (Post-MVP capability)
- TUI / Real-time dashboard view (Phase 3 capability)

### Data Architecture

- **Decision**: Stream-based Hashing with `sha2` + `tokio::io::AsyncRead`.
- **Version**: `sha2` (latest stable).
- **Rationale**: Meets the strict <512MB RAM budget while supporting 100GB+ artifacts. Paged chunk reading guarantees memory safety.
- **Provided by Starter**: No (Industry standard crates to be explicitly added).

- **Decision**: Audit Report format via `csv` crate.
- **Version**: `csv` (latest stable).
- **Rationale**: Ensures robust output formatting (escaping, safe characters) for the FR17 compliance logs.
- **Provided by Starter**: No.

### Authentication & Security

- **Decision**: Environment Variable secrets injection.
- **Version**: `std::env` (native rust).
- **Rationale**: Decouples secrets from configuration files; adheres to CI/CD security standards to prevent leakage.
- **Provided by Starter**: Yes (Standard environment variables configuration).

### API & Communication Patterns

- **Decision**: Metrics Server Endpoint via `axum` + `metrics-exporter-prometheus`.
- **Version**: `axum` (latest stable).
- **Rationale**: Minimal overhead background task serving FR16. Integrates perfectly with the existing `tokio` asynchronous runtime.
- **Provided by Starter**: No.

### Application Architecture

- **Decision**: Declarative YAML parsing using `serde_yaml`.
- **Version**: `serde_yaml` (latest stable).
- **Rationale**: Provides strictly typed parsing structs, preventing malformed routing configurations.
- **Provided by Starter**: Yes.

### Decision Impact Analysis

**Implementation Sequence:**
1. Scaffold project and basic CLI command parsing (`clap`).
2. Implement declarative YAML config parsing and validation.
3. Build the core stream processing engine (`reqwest` + `sha2`) for throttled I/O.
4. Implement the proxy-aware target upload flow.
5. Overlay observability (`axum` metrics and JSON `tracing`).

**Cross-Component Dependencies:**
- The HTTP Client (`reqwest`) and Hashing Engine (`sha2`) must be tightly coupled to ensure we can hash while streaming the download chunk, saving memory and time by avoiding multiple passes over the artifact.

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:**
3 areas where AI agents could make different choices that would break the non-functional requirements.

### Naming Patterns

**Configuration Naming Conventions:**
- Base config file MUST be named `j2n.yaml`.
- All environment variables MUST be prefixed with `J2N_` (e.g., `J2N_JFROG_TOKEN`, `J2N_THROTTLE_MB`).

**Code Naming Conventions:**
- All async network functions MUST be suffixed with `_async` if a sync version exists, or otherwise just follow standard `snake_case` (e.g., `download_artifact`).
- Metric counters MUST follow Prometheus snake_case conventions prefixed with `j2n_` (e.g., `j2n_transfer_bytes_total`).

### Format Patterns

**Error Handling Formats:**
- ALL application-level functions MUST return `anyhow::Result<T>`.
- ALL library/core engine functions MUST return a strongly-typed `thiserror` enum (e.g., `Result<T, MigrationError>`).
- NEVER use `.unwrap()` or `.expect()` in production paths. Errors must propagate up to the CLI boundary to be formatted into JSON by `tracing`.

**Log Formats:**
- ALL stdout MUST be structured JSON via `tracing-subscriber`. No `println!` or `print!` macros are allowed anywhere in the codebase (they break programmatic JSON parsing).

### Communication & Process Patterns

**Stream Processing Patterns (CRITICAL for NFR2):**
- NEVER load full file bodies into memory (`reqwest::Response::bytes()` is forbidden).
- MUST use `reqwest::Response::bytes_stream()` paired with `tokio_util::io::StreamReader`.
- Hashing MUST occur incrementally over chunks as they are read from the stream, using `sha2::Digest::update`.

### 2. The Engine (`src/engine/`)

The core migration orchestrator executing the data plane.

- **TransferOrchestrator**: Consumes matched entries from the scanner, spawning bounded `tokio` tasks to transfer individual chunks.
- **StreamProcessor**: Handles the `reqwest` byte streams, pipelining data directly to the TargetClient while calculating `sha2` hashes on the fly.
- **RetryAgent**: Wraps API calls with `tokio-retry` exponential backoff strategies for 503/504 errors.
- **StateStore**: Manages a local SQLite database (`.j2n/state.db`) tracking artifact progress and checksums for instantaneous resumes without querying remote APIs.
- **TargetMapper**: Abstracts the differences between target proxy APIs (e.g., Docker v2 manifests vs Maven file uploads).

**Retry Patterns:**
- API calls MUST be wrapped in an exponential backoff retry loop using the `tokio-retry` or `reqwest-retry` patterns, specifically catching 503, 504, and network timeout errors.

### Enforcement Guidelines

**All AI Agents MUST:**
- Adhere to the strict memory streaming pattern for all network operations.
- Ensure all application output routes through the `tracing` JSON subscriber.
- Verify CLI inputs thoroughly before mutating state (`dry-run` first).

### Pattern Examples

**Good Examples:**
```rust
// GOOD: Proper streaming to avoid OOM
let mut stream = response.bytes_stream();
let mut hasher = Sha256::new();
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    hasher.update(&chunk);
    // write chunk to disk or proxy
}
```

**Anti-Patterns:**
```rust
// BAD: Will OOM on 10GB files
let body = response.bytes().await?; 
let hash = sha256_hash(&body);
```

## Project Structure & Boundaries

### Complete Project Directory Structure

```text
jfrog2nexus/
├── Cargo.toml            # Project dependencies and workspace definition
├── Cargo.lock
├── j2n.yaml.example      # Example configuration file
├── .env.example          # Example environment variables for secrets
├── .github/
│   └── workflows/
│       └── release.yml   # CI/CD pipelines (e.g., cargo-dist cross-compilation)
├── tests/                # Integration tests directory
│   ├── sync_test.rs      # End-to-end tests using mock networks
│   └── hash_test.rs      # Data integrity tests
└── src/
    ├── main.rs           # CLI entrypoint; parses args and delegates to library
    ├── lib.rs            # Library entrypoint; exposes public engine modules
    ├── cli/
    │   ├── mod.rs        # clap struct definitions
    │   └── commands.rs   # Entrypoints for `sync`, `validate`, `status`, `report`
    ├── config/
    │   ├── mod.rs        # YAML deserialization structs
    │   └── mapper.rs     # Source/Target repository translation logic
    ├── engine/           # Core domain logic
    │   ├── mod.rs
    │   ├── download.rs   # async tokio chunk reader
    │   ├── upload.rs     # reqwest poster (proxy aware)
    │   └── hasher.rs     # sha2 streaming implementation
    ├── observability/
    │   ├── mod.rs
    │   ├── server.rs     # axum metrics server background task
    │   └── metrics.rs    # Prometheus counter definitions
    └── audit/
        ├── mod.rs
        └── csv.rs        # CSV compliance log generation
```

### Architectural Boundaries

**CLI Boundary (`src/cli/`):**
- Strictly responsible for parsing `clap` arguments, returning `anyhow::Result`, and initiating `tracing` subscribers.
- NEVER contains HTTP intelligence or stream manipulation.

**Engine Boundary (`src/engine/`):**
- The heavy lifter. Owns the `reqwest` clients and connection pools.
- Must return `thiserror` typed enums to allow the CLI boundary to decide whether to exit(1) or retry.

**Metrics Boundary (`src/observability/`):**
- Exists as a decoupled background `tokio` task. The engine merely increments global atomic counters (`j2n_transfer_bytes_total`), and this boundary handles the HTTP serving.

### Requirements to Structure Mapping

**Feature Mapping:**
- **Configuration (FR1-FR4):** Handled entirely within `src/config/`.
- **Migration & Resilience (FR5-FR13):** Handled entirely within `src/engine/`.
- **Metrics (FR16):** `src/observability/server.rs`.
- **Auditing (FR17):** `src/audit/csv.rs`.

### File Organization Patterns

**Source Organization:**
Idiomatic Rust layout. `main.rs` is kept intentionally tiny, acting only as the bridge between user input/output and the core library logic contained inside `src/`.

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:**
All chosen crates (`tokio`, `reqwest`, `sha2`, `axum`) belong to the standard Rust async networking ecosystem and are guaranteed to interoperate without blocking the core event loop.

**Pattern Consistency:**
Strict streaming rules prevent OOM panics when handling 100GB+ artifacts. The requirement for `anyhow` vs `thiserror` ensures errors are strongly typed in the engine for easy retry logic, while providing rich, contextual JSON logs at the CLI boundary.

**Structure Alignment:**
The CLI/Engine/Metrics boundaries strictly enforce separation of concerns, ensuring `main.rs` remains thin while `lib.rs` handles testable business logic.

### Requirements Coverage Validation ✅

**Feature Coverage:**
All core migration features (FR1-13), observability (FR14-17), and CLI DX (FR18-19) are explicitly mapped to the project tree (`config/`, `engine/`, `observability/`, `audit/`).

**Non-Functional Requirements Coverage:**
- NFR1 & NFR4 (Performance & Throttling): Handled via `tokio` time/sleep integration inside the `reqwest` download streams.
- NFR2 (Memory <512MB): Handled via `bytes_stream()` chunk reading constraint.
- NFR9 (0% False Positive Hashes): Handled via incremental `sha2` cryptographic hashing over the data stream.

### Implementation Readiness Validation ✅

**Completeness:**
The foundation commands (`cargo add ...`) and project structure are perfectly defined for immediate translation into an Epic breakdown.

### Architecture Completeness Checklist

**✅ Requirements Analysis**
- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**✅ Architectural Decisions**
- [x] Critical decisions documented with versions
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [x] Performance considerations addressed

**✅ Implementation Patterns**
- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented

**✅ Project Structure**
- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION
**Confidence Level:** High. The constraints of the PRD are inherently solved by combining memory-safe Rust with declarative, standardized network streams.

### Implementation Handoff

**First Implementation Priority:**
Initialize the project using the standard `cargo` template defined in the "Starter Template Evaluation" section.
