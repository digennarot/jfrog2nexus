# API Contracts

## Overview
This application functions primarily as a CLI synchronization tool but exposes observability and administrative endpoints via `axum`.

## Endpoints

### Health / Metrics
- **GET /health**
  - **Description**: Returns basic application health status.
  - **Response**: `200 OK` (JSON)
- **GET /metrics**
  - **Description**: Exposes Prometheus-formatted metrics.
  - **Response**: `200 OK` (Text)

*(Note: API integration points for JFrog and Nexus are outbound HTTP client calls rather than exposed incoming routes, driven by the core sync engine in `src/engine/` using `reqwest`.)*
