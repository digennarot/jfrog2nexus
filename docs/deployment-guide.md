# Deployment Configuration

## Local/Dev Environments
- Managed largely through cargo profiles.
- Employs Docker containers for simulating Nexus and JFrog endpoints (`tests/docker-compose.yml`, `tests/uat/docker-compose.yml`).

## Production Environments
- Production binary is built via `cargo build --release`.
- Deployed as a standalone binary CLI tool or wrapped in custom deployment scripts alongside operational credentials.
- CI/CD pipelines typically manage the compilation, test verification, and binary deployment. Make sure `SOPS` secrets are safely exposed to the CI runner.
