# Development Instructions

## Prerequisites
- Rust (use `rustup` to manage toolchains, `1.70+` recommended)
- `docker` and `docker-compose` (for integration testing with actual services)
- `SOPS` and `age` (for secret management during development)

## Setup
Ensure that any required `.env` file parameters are populated or use the config files. Look at `.sops.yaml` for encryption keys. 

```bash
cargo check
cargo build
```

## Running the Application
```bash
cargo run -- --help
```

## Testing
Run unit tests:
```bash
cargo test
```

For integration testing against spin-up containers, consult `tests/README.md`.
