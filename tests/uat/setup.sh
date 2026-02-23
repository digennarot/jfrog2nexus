#!/bin/bash
set -e

echo "Waiting for Artifactory to be healthy..."
until [ "$(docker inspect -f '{{.State.Health.Status}}' artifactory-uat)" == "healthy" ]; do
    sleep 5
done
echo "Artifactory is ready."

echo "Waiting for Nexus to be healthy..."
until [ "$(docker inspect -f '{{.State.Health.Status}}' nexus-uat)" == "healthy" ]; do
    sleep 5
done
echo "Nexus is ready."

# Get Nexus admin password
NEXUS_PWD=$(docker exec nexus-uat cat /nexus-data/admin.password)
echo "Nexus Admin Password retrieved: $NEXUS_PWD"

# Change Nexus password to something standard for UAT (e.g., 'admin123')
# Note: Newer Nexus versions might need different API calls for this, 
# but we'll try to use the CLI tool or just stick with the generated one for the session.
echo "Setting up UAT repositories..."

# 1. Create a Maven repo in Artifactory (usually 'example-repo-local' exists in OSS)
# 2. Create a Maven repo in Nexus
# 3. Seed Artifactory with a test artifact

# Setup J2N config for UAT
cat <<EOF > tests/uat/uat_config.yaml
jfrog:
  url: "http://localhost:8081/artifactory"
nexus:
  url: "http://localhost:8083"
mappings:
  - source: "example-repo-local"
    target: "maven-releases"
    type: "maven"
EOF

echo "UAT Setup complete."
echo "Config generated at tests/uat/uat_config.yaml"
echo "Artifactory: http://localhost:8081/artifactory (admin/password)"
echo "Nexus: http://localhost:8083 (admin/$NEXUS_PWD)"
