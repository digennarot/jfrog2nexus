# Sprint Change Proposal: Addressing Adversarial Review Findings

## Section 1: Issue Summary
An adversarial review of the `epics.md` and `architecture.md` documents revealed 10 critical gaps that pose significant implementation risks. These gaps encompass missing recovery behaviors, undefined throughput limits, incomplete pagination logic, and missing architectural specs for data persistence and targeted repository types. If left unaddressed, they jeopardize the success criteria for automated 1TB+ migrations.

## Section 2: Impact Analysis
- **Epic Impact**: 
    - **Epic 1** (Core Migration) is missing pagination logic (Story 1.4), distinct target API abstractions (Story 1.5), and concurrent connection caps.
    - **Epic 2** (Resilience) is missing a persistence strategy for resumes (Story 2.2), mismatch recovery behavior (Story 2.1), and token refresh logic.
    - **Epic 3** (Observability) is missing metrics security (Story 3.2) and file permission handling (Story 3.4).
- **Artifact Conflicts**:
    - **PRD**: Needs to refine MVP scope to explicitly include state persistence, rate limit bounds, and metrics security.
    - **Architecture**: Needs explicit components for a State Database (e.g., SQLite) and target-specific API mappers (Docker v2 vs Maven raw).
    - **Epics**: Requires additional stories and modified acceptance criteria for 8 existing stories.

## Section 3: Recommended Approach
**Option 1: Direct Adjustment (Hybrid)**
We will directly modify existing stories and add two new stories within the current epics to address the gaps. This maintains the project timeline while drastically reducing technical risk. The MVP scope (PRD) and Architecture will receive surgical updates to match.

- **Effort Estimate**: Medium
- **Risk Level**: Low (Reduces overall risk)

## Section 4: Detailed Change Proposals

### PRD Updates

**Location:** `prd.md` -> MVP - Phase 1
**Change:** Add explicit requirements to support the new resilience features.
```diff
 ### MVP - Phase 1
 - CLI synchronization for Docker and Maven artifacts via HTTPS APIs.
 - Resilience: `--resume-by-checksum` and `--throttle`.
+- Resilience: Persistent local state (SQLite) for immediate resumes and token refresh strategies.
 - Declarative YAML configuration (mappings, proxies).
 - `--dry-run` execution mode.
 - Prometheus `/metrics` endpoint.
```

### Architecture Updates

**Location:** `architecture.md` -> Application Core -> Components
**Change:** Add `StateStore` and `TargetMapper`.
```diff
 ### 2. The Engine (`src/engine/`)
 - **StreamProcessor**: Handles the `reqwest` byte streams, pipelining data directly to the TargetClient while calculating `sha2` hashes on the fly.
 - **RetryAgent**: Wraps API calls with `tokio-retry` exponential backoff strategies for 503/504 errors.
+- **StateStore**: Manages a local SQLite database (`.j2n/state.db`) tracking artifact progress and checksums for instantaneous resumes without querying remote APIs.
+- **TargetMapper**: Abstracts the differences between target proxy APIs (e.g., Docker v2 manifests vs Maven file uploads).
```

### Epic & Story Updates

**Location:** `epics.md` -> Epic 1
**Change:** Update Story 1.4 (Pagination) and Story 1.5 (Connections & Target APIs).

```diff
 ### Story 1.4: Artifact Traversal and Dry-Run Execution
 **Acceptance Criteria:**
 **Given** a source repository with Docker and Maven artifacts
-**When** I execute `jfrog2nexus sync --dry-run`
-**Then** the tool queries the JFrog API to list all artifacts matching the mapping
+**When** I execute `jfrog2nexus sync --dry-run`
+**Then** the tool queries the JFrog API using pagination tokens to recursively list all artifacts matching the mapping
 **And** prints a simulated plan of what would be transferred, without downloading any bytes.
 
 ### Story 1.5: Core Streaming Transfer Engine
 **Acceptance Criteria:**
 **Given** a successful dry-run plan
-**When** I execute `jfrog2nexus sync` (without dry-run)
-**Then** the system streams the artifact bytes directly from JFrog to Nexus
-**And** the transfer succeeds without loading the entire payload into RAM (memory remains <512MB).
+**When** I execute `jfrog2nexus sync` (without dry-run)
+**Then** the system streams the artifact bytes directly from JFrog to Nexus using a bounded `tokio::spawn` worker pool (max 50 concurrent)
+**And** dynamically uses the correct target API mechanism (Docker v2 push vs Maven PUT) based on the repository mapping.
```

**Location:** `epics.md` -> Epic 2
**Change:** Update Story 2.1 (Mismatch Recovery), Story 2.2 (Data Persistence), Story 2.4 (Throttling Concurrency), and add Story 2.5 (Token Refresh).

```diff
 ### Story 2.1: Streaming Checksum Calculation
 **Acceptance Criteria:**
 **Given** an active artifact transfer from `jfrog2nexus sync`
 **When** the data streams to the destination
 **Then** the system concurrently calculates a SHA256 hash using memory-safe chunking
-**And** validates the final hash against the source metadata.
+**And** validates the final hash against the source metadata
+**And** automatically deletes the target file and requeues the transfer if a mismatch is detected.
 
 ### Story 2.2: Resumable Transfers via Checksum Matching
 **Acceptance Criteria:**
 **Given** a partially completed migration where some artifacts exist on the target
 **When** `jfrog2nexus sync --resume-by-checksum` is executed
-**Then** the system compares the checksums of files present on the target with the source
-**And** skips identical files, only downloading the remaining missing artifacts.
+**Then** the system queries the local `.j2n/state.db` SQLite database to compare checksums
+**And** skips identical files without requiring remote target API validation, downloading only missing artifacts.
+
+### Story 2.4: Transfer Rate Throttling
+**Acceptance Criteria:**
+**Given** a running `jfrog2nexus sync` operation with multiple concurrent workers
+**When** the user provides the `--throttle=<limit_mb_s>` flag
+**Then** the async stream processor restricts the I/O bytes read
+**And** the total network bandwidth consumed across the entire global token bucket drops to the specified limit within 2 seconds.
+
+### Story 2.5: Dynamic Token Refresh
+As an operations engineer,
+I want the tool to refresh API tokens if they expire during a multi-day transfer,
+So that massive migrations don't fail halfway through.
+
+**Acceptance Criteria:**
+**Given** an active migration that exceeds the initial API token's Time-To-Live (TTL)
+**When** the target API returns a `401 Unauthorized` mid-transfer
+**Then** the system intercepts the error, attempts a generic token refresh routine (or prompts re-evaluation of env vars),
+**And** resumes the transfer pool automatically.
```

**Location:** `epics.md` -> Epic 3
**Change:** Update Story 3.2 (Metrics Security) and Story 3.4 (Permissions).

```diff
 ### Story 3.2: Prometheus Metrics Server
 **Acceptance Criteria:**
 **Given** the CLI is actively syncing artifacts
 **When** I curl `http://localhost:9090/metrics`
-**Then** the system returns a Prometheus-compatible text payload via an internal `axum` server
+**Then** the system returns a Prometheus-compatible text payload via an internal `axum` server bound explicitly to `127.0.0.1` (no generic `0.0.0.0` exposure)
 **And** the payload includes counters for `j2n_transfer_bytes_total` and HTTP status codes.
 
 ### Story 3.4: Compliance Audit Report Generation
 **Acceptance Criteria:**
 **Given** a completed migration
 **When** I execute `jfrog2nexus report generate`
 **Then** the tool generates a `.csv` file detailing every artifact path, the Artifactory SHA256, and the Nexus SHA256
-**And** the CSV correctly escapes special characters and formats cleanly.
+**And** gracefully errors with a clear message if the execution environment lacks write permissions for the out directory.
```

## Section 5: Implementation Handoff

- **Scope Classification:** Moderate
- **Route To:** Product Owner / Scrum Master (for backlog changes), Architect (for architecture document updates).
- **Handoff Deliverables:** These accepted edits will be written directly into `prd.md`, `architecture.md`, and `epics.md`. The `sprint-status.yaml` (if generated) would reflect the new stories.
