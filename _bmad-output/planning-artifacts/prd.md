---
stepsCompleted: ['step-01-init', 'step-02-discovery', 'step-02b-vision', 'step-02c-executive-summary', 'step-03-success', 'step-04-journeys', 'step-05-domain', 'step-06-innovation', 'step-07-project-type', 'step-08-scoping', 'step-09-functional', 'step-10-nonfunctional', 'step-11-polish']
inputDocuments: ['_bmad-output/brainstorming/brainstorming-session-2026-02-22.md']
briefCount: 0
researchCount: 0
brainstormingCount: 1
projectDocsCount: 0
workflowType: 'prd'
classification:
  projectType: 'CLI Tool'
  domain: 'DevOps Tooling'
  complexity: 'Medium'
  projectContext: 'Greenfield'
---

# Product Requirements Document - jfrog2nexus

**Author:** Tiziano_di_gennaro
**Date:** 2026-02-22

## Executive Summary

`jfrog2nexus` is a Rust-based CLI tool automating zero-downtime artifact migrations from JFrog Artifactory to Sonatype Nexus. It solves migration brittleness via smart proxy integration and resumable transfers, eliminating manual restarts for 1TB+ Docker registries. Operations teams can flip DNS on Day 1, allowing data to chunk invisibly in the background until Artifactory is decommissioned without service outages.

### Core Differentiators
- **Resumable HTTPS Transfers**: Native `--resume-by-checksum` capabilities prevent data blindspots and restart failures.
- **Smart Proxy Awareness**: Architecture understands proxy topologies to circumvent common AQL property rejections.
- **Auditable & Configurable**: Verbose JSON logging and YAML-based configuration for granular repository mapping and targeted rate limits.

## Project Classification

- **Project Type**: CLI Tool (Rust, server-to-server migration)
- **Domain**: DevOps / Developer Tooling
- **Context**: Greenfield

## Success Criteria

### User Success
Operations engineers recover automatically from network interruptions during large transfers (e.g., 100GB Docker layers) via checksum validation, without manual intervention, while monitoring real-time progress.

### Business Success
- **6-Week Decommissioning**: 100% of required artifacts synced and verified via checksum matching.
- **Zero Prod Impact**: >95% proxy cache hit rates achieved, allowing legacy server shutdown with no business interruption.

### Technical Success
- **Performance**: Sustains 95MB/s transfer speeds.
- **Data Integrity**: 0 corruption events, guaranteed by post-transfer SHA256 validation.
- **Resource Efficiency**: <5% CPU utilization on standard PikaOS-like environments.

## Product Scope

### MVP - Phase 1
- CLI synchronization for Docker and Maven artifacts via HTTPS APIs.
- Resilience: `--resume-by-checksum` and `--throttle`.
- Resilience: Persistent local state (SQLite) for immediate resumes and token refresh strategies.
- Declarative YAML configuration (mappings, proxies).
- `--dry-run` execution mode.
- Prometheus `/metrics` endpoint.
- JSON stdout logging and CSV SHA256 audit exports.

### Growth Features - Phase 2 & 3
- Support for NPM and PyPI artifacts.
- Advanced repository filtering and exclusion rules.
- Dedicated AQL-to-CQL translation layer.
- Hosted rebuild manifests.

## User Journeys

### 1. The Lead Migration Engineer
**Goal**: Move 5TB of artifacts in 6 weeks.
**Journey**: Configures YAML mappings and runs `sync --dry-run` to validate. Schedules `sync --resume-by-checksum` via cron. Upon network interruption at 60% of a 100GB transfer, the next cron execution verifies checksums, skips transferred data, and completes the remaining 40GB automatically. DNS flips to Nexus at week 5.

### 2. The Platform Admin
**Goal**: Ensure infrastructure stability during migration.
**Journey**: Configures Prometheus to scrape `/metrics`. Sets Grafana alerts for cache hit rates (<90%) and CPU usage (>80%). Observes sustained 95MB/s transfers and <5% CPU load.

### 3. The Security & Audit Officer
**Goal**: Validate integrity of migrated compliance artifacts.
**Journey**: Generates CSV audit report post-migration. Validates 100% match between Artifactory and Nexus SHA256 hashes to sign off on legacy server decommissioning.

### 4. The CI/CD Integrator
**Goal**: Automate migrations safely in pipelines.
**Journey**: Integrates CLI into GitLab CI. Injects dynamic YAML config and API key environment variables. Executes `sync --dry-run` as a non-blocking test stage to validate mappings prior to destructive execution.

## Project-Type Requirements (CLI)

### Execution Model
Strictly non-interactive to guarantee zero-touch execution in automated environments (e.g., GitLab CI). `--dry-run` flag required to verify destructive or impactful operations prior to execution.

### Configuration & Secrets
Declarative YAML configuration defines repository mappings and operational boundaries. Secrets (`NEXUS_TOKEN`, `JFROG_TOKEN`) are injected exclusively via environment variables.

### Command Structure
- `sync`: Core migration engine (`--dry-run`, `--resume-by-checksum`, `--throttle=<limit>`, `--proxy-endpoint`).
- `config validate`: Lints YAML configuration and validates upstream/proxy connectivity.
- `status`: Provides polling of active migration progress.
- `report generate`: Outputs cache hits, transfer statistics, and checksum results.

## Functional Requirements

### 1. Configuration & Mapping
- **FR1:** The system can parse a declarative YAML configuration file defining source and target repository mappings.
- **FR2:** The system can parse proxy endpoints and routing rules from the YAML configuration.
- **FR3:** The system can read authentication secrets exclusively from environment variables.
- **FR4:** The system can validate YAML configuration syntax and upstream connectivity without initiating data transfer.

### 2. Migration Execution
- **FR5:** The system can authenticate to JFrog Artifactory via HTTPS API.
- **FR6:** The system can authenticate to Sonatype Nexus via HTTPS API.
- **FR7:** The system can traverse and list Docker and Maven artifacts within a source repository.
- **FR8:** The system can transfer artifacts from source to target repository.
- **FR9:** The system can execute a dry-run simulation that maps artifacts and validates connectivity without transferring data.

### 3. Resilience & Control
- **FR10:** The system can calculate and verify SHA256 checksums of artifacts on both source and target servers.
- **FR11:** The system can resume an interrupted transfer by comparing checksums and skipping identical destination files.
- **FR12:** The system can restrict transfer bandwidth based on a user-defined threshold limit.
- **FR13:** The system can implement connection pooling and retry logic with exponential backoff on API timeouts and 503 errors.

### 4. Observability & Auditing
- **FR14:** The system can output structured JSON logs detailing operational events and errors to standard output.
- **FR15:** The system can generate a progress report detailing active migration state across mappings.
- **FR16:** The system can expose a `/metrics` HTTP endpoint serving Prometheus-compatible telemetry.
- **FR17:** The system can generate a CSV audit report containing pre- and post-transfer SHA256 hashes.

### 5. Developer Experience
- **FR18:** The system can provide shell autocompletion definitions for `bash`, `zsh`, and `fish`.
- **FR19:** The system can provide command-line documentation describing available commands and flags.

## Non-Functional Requirements

### Performance & Resource Efficiency
- **NFR1:** The system shall sustain a transfer rate of 95MB/s on a gigabit network without throttling as measured by OS network metrics.
- **NFR2:** The system shall maintain a memory footprint under 512MB during continuous 100GB+ artifact transfers as measured by process monitoring.
- **NFR3:** The system shall utilize less than 5% CPU on a standard dual-core virtual machine during active transfers as measured by process monitoring.
- **NFR4:** The system shall restrict network bandwidth to the user-defined limit within 2 seconds of transfer initiation as measured by network metrics.

### Reliability & Resilience
- **NFR5:** The system shall automatically resume an interrupted data transfer without payload corruption upon the next execution as measured by SHA256 validation.
- **NFR6:** The system shall achieve a proxy cache hit rate exceeding 95% for Docker layer blobs as measured by proxy access logs.

### Security
- **NFR7:** The system shall transmit all data exclusively via HTTPS/TLS 1.2+ protocols as measured by network inspection.
- **NFR8:** The system shall never output environment variable secrets to `stdout` or log files as measured by log auditing.
- **NFR9:** The system shall maintain a 0% false-positive rate for data corruption detection during post-transfer SHA256 validation as measured by integration testing.
