# Story 3.2: Prometheus Metrics Server

Status: done

## Story

As a platform admin,
I want a `/metrics` HTTP endpoint serving real-time telemetry,
So that I can set up Grafana alerts for cache hit rates and transfer speeds.

## Acceptance Criteria

1. **Given** the CLI is actively syncing artifacts
2. **When** I curl `http://localhost:9090/metrics`
3. **Then** the system returns a Prometheus-compatible text payload via an internal `axum` server bound explicitly to `127.0.0.1` (no generic `0.0.0.0` exposure)
4. **And** the payload includes counters for `j2n_transfer_bytes_total` and HTTP status codes.

## Tasks / Subtasks

- [x] Task 1: Setup Prometheus Recorder (AC: 4)
  - [x] Integrate `metrics` and `metrics-exporter-prometheus` crates.
- [x] Task 2: Implement Metrics Server (AC: 3)
  - [x] Create an `axum` server task that serves the metrics endpoint.
- [x] Task 3: Instrument Transfer Engine (AC: 4)
  - [x] Add real-time counters for bytes transferred.
  - [x] Add counters for HTTP status codes encountered.
- [x] Task 4: CLI Integration (AC: 3)
  - [x] Add `--metrics-addr` flag.
- [x] Task 5: Validation (AC: 1, 2)
  - [x] Manual or automated check of the `/metrics` endpoint.

## Dev Notes

- Metrics server runs in a background tokio task.
- Uses `127.0.0.1:9090` by default.
