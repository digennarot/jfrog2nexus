# jfrog2nexus

`jfrog2nexus` is a high-performance CLI tool written in Rust designed to migrate artifacts from JFrog Artifactory to Sonatype Nexus. It supports resuming partial transfers, rate limiting, metrics gathering, state management, and configuration validation.

## Features

- **High Performance:** Concurrent transfers utilizing asynchronous I/O.
- **Resilience:** Resumes partially completed migrations using a local SQLite state database (`state.db`).
- **Dry-run Mode:** Preview the migration and view aggregate numbers without triggering actual transfers.
- **Throttling:** Throttle your transfer speed using a configurable `max_kbps` rate limiter.
- **Metrics Dashboard:** Integrates a metrics server to monitor live bytes transferred.
- **Reporting:** Generates CSV audit reports on migration status.
- **Configuration Validation:** Preflight checks for connection and upstream validation.

## Installation

Ensure you have Rust and Cargo installed. To build the project:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/jfrog2nexus`.

## Usage

```bash
jfrog2nexus [COMMAND]
```

### Commands

*   `sync`: Sync artifacts from JFrog to Nexus.
*   `status`: Get the current status of the migration (completed artifacts, total migrated size, and live metrics).
*   `report`: Generate CSV audit reports for your migration.
*   `config`: Validate configuration files and test upstream connectivity.
*   `generate-completions`: Generate shell autocompletion scripts for various shells (bash, zsh, fish, etc).
*   `help`: Print the help message.

### Sync Command Options

```bash
jfrog2nexus sync [OPTIONS]
```

*   `-c, --config <CONFIG_PATH>`: Path to matching configuration. (default: `.j2n/j2n.yaml`)
*   `--dry-run`: Run without performing actual transfers. Builds and outputs a sync plan.
*   `--resume-by-checksum`: Resume partially completed transfers based on local state database.
*   `--max-kbps <MAX_KBPS>`: Maximum transfer rate in KB/s (0 for unlimited). (default: 0)
*   `-n, --concurrency <CONCURRENCY>`: Number of concurrent transfers. (default: 50)
*   `--metrics-addr <ADDR>`: Address to bind metrics server to. (default: `127.0.0.1:9090`)

### Status Command

View the migration progress and state:

```bash
jfrog2nexus status [OPTIONS]
```

*   `--db-path <PATH>`: Path to the local state database. (default: `.j2n/state.db`)
*   `--metrics-url <URL>`: URL of the metrics server to query for real-time stats. (default: `http://127.0.0.1:9090`)

### Report Command

Generate audit reports from the internal state database:

```bash
jfrog2nexus report generate [OPTIONS]
```

*   `--db-path <PATH>`: Path to state database. (default: `.j2n/state.db`)
*   `-o, --output <FILE>`: Path to output CSV file. (default: `migration_report.csv`)

### Configuration

By default, the application looks for a configuration file located at `.j2n/j2n.yaml`. You can validate your YAML configuration and upstream connectivity using the `config` command:

```bash
jfrog2nexus config validate --config .j2n/j2n.yaml
```

**Example `.j2n/j2n.yaml`:**
```yaml
jfrog:
  url: "https://jfrog.example.com"
  # Authentication can be provided via J2N_JFROG_TOKEN env var, 
  # or through a file supporting dynamic token refresh:
  token_file: "/etc/secrets/jfrog_token"
nexus:
  url: "https://nexus.example.com"
  # Authentication can be provided via J2N_NEXUS_TOKEN env var,
  # or through a file supporting dynamic token refresh:
  token_file: "/etc/secrets/nexus_token"
proxy: # Optional
  url: "http://proxy.example.com:8080"
mappings:
  - source: "docker-local"
    target: "docker-hosted"
    type: "docker"
  - source: "maven-releases"
    target: "maven-releases"
    type: "maven"
```

### Environment Variables

*   `J2N_JFROG_TOKEN`: JFrog access token (if `token_file` is not configured).
*   `J2N_NEXUS_TOKEN`: Nexus access token (if `token_file` is not configured).
*   `J2N_ALLOW_HTTP`: Set to `true` to disable the default HTTPS requirement for upstream URLs (useful for local testing).

The state database is local to your environment and located at `.j2n/state.db` by default.

## License

This project is licensed under standard open-source terms. Check your internal licensing if applicable.
