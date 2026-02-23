# jfrog2nexus User Guide

Welcome to the `jfrog2nexus` User Guide! This document covers everything you need to know about installing, configuring, and running the `jfrog2nexus` CLI, a powerful tool for migrating and synchronizing artifacts effectively from JFrog Artifactory to Sonatype Nexus.

---

## 1. Installation

As `jfrog2nexus` is primarily a Rust application, you can compile it from source using Cargo:

```bash
git clone <repository_url> jfrog2nexus
cd jfrog2nexus
cargo build --release
```

The compiled binary will be located in `target/release/jfrog2nexus`. You can move this executable to your `PATH` for easier access.

To generate shell autocomplete scripts (e.g. for bash, zsh, fish), use the `generate-completions` command:
```bash
jfrog2nexus generate-completions bash > ~/.jfrog2nexus-completion.bash
source ~/.jfrog2nexus-completion.bash
```

---

## 2. Configuration

`jfrog2nexus` requires two main configuration components:

1. **Environment Variables**: For secrets and sensitive information.
2. **YAML Configuration File**: For mapping repositories and setting API endpoints.

### Environment Variables
The application uses the following environment variables. Note that API keys from earlier versions have been replaced with dedicated tokens:

- `J2N_JFROG_TOKEN`: **Required**. The secret token used for JFrog Artifactory authentication.
- `J2N_NEXUS_TOKEN`: **Required**. The secret token used for Sonatype Nexus authentication.
- `J2N_ALLOW_HTTP`: **Optional**. Set to `true` to bypass HTTPS enforcement. This is primarily useful for local testing setups or containers without TLS.

Example:
```bash
export J2N_JFROG_TOKEN="your-jfrog-token"
export J2N_NEXUS_TOKEN="your-nexus-token"
```

### Configuration File (`.j2n/j2n.yaml`)
By default, the application looks for a configuration file at `.j2n/j2n.yaml`.
This file maps your JFrog repositories to Nexus repositories and defines endpoints. 

Here is a comprehensive example configuration showcasing all available properties:

```yaml
jfrog:
  url: "https://jfrog.example.com"
  token_file: "/run/secrets/jfrog_token" # Optional: alternative to J2N_JFROG_TOKEN
nexus:
  url: "https://nexus.example.com"
  token_file: "/run/secrets/nexus_token" # Optional: alternative to J2N_NEXUS_TOKEN
mappings:
  - source: "docker-local"
    target: "docker-hosted"
    type: "docker"
  - source: "maven-releases"
    target: "maven-releases-hosted"
    type: "maven"
proxy: # Optional: HTTP proxy configuration
  url: "http://proxy.example.com:8080"
```

**Supported Repository Types:**
The `type` field in `mappings` must be precisely one of the following: `docker`, `maven`, `pypi`, `npm`, `nuget`, `helm`, `go`, or `raw`.

To validate your configuration and ensure connections are correctly authenticated, run:
```bash
jfrog2nexus config validate
# Or specify a custom config path:
jfrog2nexus config validate -c path/to/my-config.yaml
```

---

## 3. Usage & Commands

The CLI provides several subcommands for different operations. 

### `sync`
The core command that handles artifact synchronization based on your mappings.

```bash
jfrog2nexus sync [OPTIONS]
```

**Options:**
- `-c, --config <CONFIG>`: Path to the configuration file (default: `.j2n/j2n.yaml`).
- `--dry-run`: Run the sync process without actually moving any files, useful for testing out configs.
- `--resume-by-checksum`: Resume partially completed transfers. The tool utilizes an embedded SQLite state database to determine what's already been synced.
- `--max-kbps <MAX_KBPS>`: Maximum transfer rate in KB/s (default is `0` for unlimited).
- `-n, --concurrency <CONCURRENCY>`: Max number of concurrent asynchronous transfers (default: `50`).
- `--metrics-addr <METRICS_ADDR>`: Bind address for metrics server (default: `127.0.0.1:9090`).

### `status`
Query the real-time status of your ongoing migrations directly from the tracking database or the metrics server.

```bash
jfrog2nexus status [OPTIONS]
```

**Options:**
- `--db-path <DB_PATH>`: Path to the embedded SQLite state database (default: `.j2n/state.db`).
- `--metrics-url <METRICS_URL>`: URL of the metrics server for real-time Prometheus stats (default: `http://127.0.0.1:9090`).

### `report`
Generate audit and migration reports based on synchronization history.

```bash
jfrog2nexus report generate [OPTIONS]
```

**Options:**
- `--db-path <DB_PATH>`: Path to the state database to read from (default: `.j2n/state.db`).
- `-o, --output <OUTPUT>`: Output path for the CSV report (default: `migration_report.csv`).

### `config validate`
Check whether your configuration file is valid, verifying mappings, endpoints, and token availability without changing any data.

```bash
jfrog2nexus config validate -c path/to/cfg.yaml
```

---

## 4. Best Practices

1. **Dry Runs**: Always run a sync operation with the `--dry-run` flag when you first create or update the configuration mappings.
2. **Rate Limiting**: Use the `--max-kbps` and `--concurrency` flags to throttle requests in production environments, ensuring that both JFrog and Nexus nodes aren't overwhelmed with requests.
3. **Resuming Migrations**: If your script crashes or stops, simply rerun with `--resume-by-checksum` to quickly pick up where it left off, avoiding redundant artifact transfers.
