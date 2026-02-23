# jfrog2nexus Integration Tests

This directory contains the integration tests for the `jfrog2nexus` Rust CLI. The tests utilize a Docker Compose environment to spin up realistic, lightweight simulations of JFrog Artifactory and Sonatype Nexus using Wiremock.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) or [Podman](https://podman.io/) (with docker-compose compat)
- Rust toolchain (`cargo`)

## Setup Instructions

1. Copy the example environment file if you haven't already:
   ```bash
   cp .env.example .env
   ```

2. Start the simulation environment:
   ```bash
   make test-env-up
   # OR: docker compose -f tests/docker-compose.yml up -d
   ```

## Running Tests

To run the integration tests locally against the mocked environment:

```bash
make test-integration
# OR: cargo test --test integration_test
```

When you are finished testing, bring down the environment:

```bash
make test-env-down
# OR: docker compose -f tests/docker-compose.yml down
```

## Architecture Overview

- **`docker-compose.yml`**: Configures the Wiremock services.
- **`fixtures/`**: Contains data factories, mock API responses, and dummy files to speed up test writing.
- **`common/mod.rs`**: Contains shared setup code, including HTTP pollers that wait for the Docker services to become healthy before the tests start.

## Best Practices

- **Isolation**: Each integration test should aim to use unique mock paths or parameters to avoid state collisions if tests run in parallel.
- **Cleanup**: Integration tests should rely on the `setup()` function from `common::mod` to guarantee environment readiness.
