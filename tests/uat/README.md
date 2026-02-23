# User Acceptance Testing (UAT) Environment

This environment spins up real instances of JFrog Artifactory and Sonatype Nexus 3 for end-to-end testing of `jfrog2nexus`.

## Prerequisites

- Docker and Docker Compose
- At least 8GB of free RAM (4GB for Artifactory, 4GB for Nexus)

## Usage

1. **Start the environment:**
   ```bash
   docker compose -f tests/uat/docker-compose.yml up -d
   ```

2. **Wait for services and setup:**
   ```bash
   ./tests/uat/setup.sh
   ```

3. **Run the tool against UAT:**
   ```bash
   # Export credentials (adjust Nexus password from setup.sh output)
   export J2N_JFROG_TOKEN="admin:password"
   export J2N_NEXUS_TOKEN="admin:XYZ"
   
   cargo run -- sync --config tests/uat/uat_config.yaml
   ```

## Configuration

- **Artifactory**: [http://localhost:8081/artifactory](http://localhost:8081/artifactory) (admin / password)
- **Nexus**: [http://localhost:8083](http://localhost:8083) (admin / see setup.sh output)

## Features Verified

- Real HTTPS/HTTP API interaction
- Real repository structure traversal
- Actual binary streaming and persistence
- Checksum validation against real server responses
