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

### Docker Quick Start (Recommended)

Since `jfrog2nexus` is available as a Docker package (`ghcr.io/digennarot/jfrog2nexus`), the easiest way to configure and run it is using Docker Compose. Make sure you mount your config file and define your tokens. 

**Using a Token File (Most Secure)**: For production or environments where environment variables could leak or are immutable, mounting the tokens as a file is the recommended approach.

Here is a simple `docker-compose.yml` demonstrating a secure setup mapping a local `.j2n/` directory:

```yaml
services:
  jfrog2nexus:
    image: ghcr.io/digennarot/jfrog2nexus:latest
    volumes:
      - ./.j2n/j2n.yaml:/.j2n/j2n.yaml:ro
      - ./.j2n/jfrog_token:/run/secrets/jfrog_token:ro
      - ./.j2n/nexus_token:/run/secrets/nexus_token:ro
```

### Environment Variables Fallback 

For quick local tests without file mounting, you can still provide tokens directly as environment variables:

- `J2N_JFROG_TOKEN`: The secret token used for JFrog Artifactory authentication.
- `J2N_NEXUS_TOKEN`: The secret token used for Sonatype Nexus authentication.
- `J2N_ALLOW_HTTP`: **Optional**. Set to `true` to bypass HTTPS enforcement. This is primarily useful for local testing.

### Configuration File (`.j2n/j2n.yaml`)

By default, the application looks for a configuration file at `.j2n/j2n.yaml`.
This file maps your JFrog repositories to Nexus repositories and defines endpoints. 

#### Minimal Example

This relies on environment variables (`J2N_JFROG_TOKEN`, `J2N_NEXUS_TOKEN`) for authentication:

```yaml
jfrog:
  url: "https://jfrog.example.com"
nexus:
  url: "https://nexus.example.com"
mappings:
  - source: "docker-local"
    target: "docker-hosted"
    type: "docker"
```

#### Full Example (Secure Token Files & Proxies)

```yaml
jfrog:
  url: "https://jfrog.example.com"
  token_file: "/run/secrets/jfrog_token" # Most secure
nexus:
  url: "https://nexus.example.com"
  token_file: "/run/secrets/nexus_token" # Most secure
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

### Validating Configuration

To validate your configuration and ensure connections are correctly authenticated before actually running a sync, run:
```bash
jfrog2nexus config validate -c path/to/my-config.yaml
```

---

## 3. Usage & Commands

The CLI provides several subcommands for different operations. Under the recommended Docker setup, you'll prepend commands with `docker compose run jfrog2nexus`.

### 1. Test your configuration (Dry Run)

The core command is `sync`, which handles artifact synchronization based on your mappings. Always use `--dry-run` first. This triggers the artifact scanning engine to contact the repositories and output a `SyncPlan` showing exactly what *would* be moved, without making any modifications.

```bash
docker compose run jfrog2nexus sync --dry-run
```

### 2. Run the migration

Once you are satisfied with the dry run output, execute the actual migration:

```bash
docker compose run jfrog2nexus sync
```

**Common Sync Options:**
- `--config <CONFIG>`: Path to the configuration file (default: `.j2n/j2n.yaml`).
- `--resume-by-checksum`: Resume partially completed transfers. The tool utilizes an embedded SQLite state database to determine what's already been synced.

*(Run `docker compose run jfrog2nexus sync --help` to see all advanced options like rate limiting and concurrency tuning.)*

### 3. Check status and reports

You can query the real-time status of your ongoing migrations directly from the tracking database, or generate audit reports.

**Query Status:**
```bash
docker compose run jfrog2nexus status
```

**Generate CSV Report:**
```bash
docker compose run jfrog2nexus report generate -o migration_report.csv
```

---

## 4. Best Practices

1. **Dry Runs**: Always run a sync operation with the `--dry-run` flag when you first create or update the configuration mappings.
2. **Rate Limiting**: Use the `--max-kbps` and `--concurrency` flags to throttle requests in production environments, ensuring that both JFrog and Nexus nodes aren't overwhelmed with requests.
3. **Resuming Migrations**: If your script crashes or stops, simply rerun with `--resume-by-checksum` to quickly pick up where it left off, avoiding redundant artifact transfers.
