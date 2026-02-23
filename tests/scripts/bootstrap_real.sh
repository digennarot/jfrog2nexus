#!/bin/bash
# bootstrap_real.sh — Seeds Artifactory and Nexus with real test data for ALL
# supported package formats: Maven, Docker, PyPI, npm, NuGet, Helm, Go, Raw.
#
# Usage:
#   docker compose -f tests/docker-compose.real.yml up -d
#   bash tests/scripts/bootstrap_real.sh
set -euo pipefail

JFROG_URL=${JFROG_URL:-"http://localhost:8081"}
JFROG_USER=${JFROG_USER:-"admin"}
JFROG_PASS=${JFROG_PASS:-"password"}
NEXUS_URL=${NEXUS_URL:-"http://localhost:8083"}

# ─── Helpers ─────────────────────────────────────────────────────────────────

jfrog_put_repo() {
    local name=$1; local pkg=$2
    curl -sf -X PUT -u "${JFROG_USER}:${JFROG_PASS}" \
        "${JFROG_URL}/artifactory/api/repositories/${name}" \
        -H "Content-Type: application/json" \
        -d "{\"rclass\":\"local\",\"packageType\":\"${pkg}\"}" \
        || echo "  (${name} may already exist)"
}

jfrog_upload() {
    local repo=$1; local remote_path=$2; local local_file=$3
    curl -sf -u "${JFROG_USER}:${JFROG_PASS}" \
        -X PUT "${JFROG_URL}/artifactory/${repo}${remote_path}" \
        -T "${local_file}"
}

nexus_post_repo() {
    local format=$1; local kind=$2; local body=$3
    curl -sf -u "admin:${NEXUS_PWD}" \
        -X POST "${NEXUS_URL}/service/rest/v1/repositories/${format}/${kind}" \
        -H "Content-Type: application/json" -d "${body}" \
        || echo "  (may already exist)"
}

# ─── 1. Wait for Artifactory ──────────────────────────────────────────────────

echo "==> Waiting for Artifactory…"
until curl -sf "${JFROG_URL}/artifactory/api/system/ping" >/dev/null; do
    sleep 5; echo "   …"
done
echo "    Artifactory ready."

# ─── 2. Create JFrog local repos ─────────────────────────────────────────────

echo "==> Creating Artifactory repositories…"
jfrog_put_repo maven-local   maven
jfrog_put_repo docker-local  docker
jfrog_put_repo pypi-local    pypi
jfrog_put_repo npm-local     npm
jfrog_put_repo nuget-local   nuget
jfrog_put_repo helm-local    helm
jfrog_put_repo go-local      go
jfrog_put_repo raw-local     generic

# ─── 3. Seed JFrog — Maven ───────────────────────────────────────────────────

echo "==> Seeding Maven artifact…"
echo "maven-content" > /tmp/lib-1.0.jar
jfrog_upload maven-local /com/example/lib/1.0/lib-1.0.jar /tmp/lib-1.0.jar

# ─── 4. Seed JFrog — Docker ──────────────────────────────────────────────────

echo "==> Seeding Docker blob + manifest…"
echo "blob1-content" > /tmp/blob1
jfrog_upload docker-local /library/hello-world/_/sha256__abcd /tmp/blob1

cat > /tmp/manifest.json <<'EOF'
{"schemaVersion":2,"layers":[{"digest":"sha256:abcd","size":13,"mediaType":"application/octet-stream"}]}
EOF
jfrog_upload docker-local /library/hello-world/latest/manifest.json /tmp/manifest.json

# ─── 5. Seed JFrog — PyPI ────────────────────────────────────────────────────

echo "==> Seeding PyPI wheel…"
echo "fake-wheel-content" > /tmp/mylib-1.0-py3-none-any.whl
jfrog_upload pypi-local /packages/mylib/mylib-1.0-py3-none-any.whl /tmp/mylib-1.0-py3-none-any.whl

echo "==> Seeding PyPI sdist…"
echo "fake-sdist-content" > /tmp/mylib-1.0.tar.gz
jfrog_upload pypi-local /packages/mylib/mylib-1.0.tar.gz /tmp/mylib-1.0.tar.gz

# ─── 6. Seed JFrog — npm ─────────────────────────────────────────────────────

echo "==> Seeding npm tarball…"
echo "npm-tarball-content" > /tmp/mylib-1.0.0.tgz
jfrog_upload npm-local /@myorg/mylib/-/mylib-1.0.0.tgz /tmp/mylib-1.0.0.tgz

# ─── 7. Seed JFrog — NuGet ───────────────────────────────────────────────────

echo "==> Seeding NuGet package…"
echo "nupkg-content" > /tmp/mylib.1.0.0.nupkg
jfrog_upload nuget-local /mylib/1.0.0/mylib.1.0.0.nupkg /tmp/mylib.1.0.0.nupkg

# ─── 8. Seed JFrog — Helm ────────────────────────────────────────────────────

echo "==> Seeding Helm chart…"
echo "helm-chart-content" > /tmp/myapp-1.0.0.tgz
jfrog_upload helm-local /myapp-1.0.0.tgz /tmp/myapp-1.0.0.tgz

# ─── 9. Seed JFrog — Go ──────────────────────────────────────────────────────

echo "==> Seeding Go module zip…"
echo "go-module-content" > /tmp/v1.0.0.zip
jfrog_upload go-local /github.com/myorg/mylib/@v/v1.0.0.zip /tmp/v1.0.0.zip

echo "==> Seeding Go module info…"
echo '{"Version":"v1.0.0","Time":"2024-01-01T00:00:00Z"}' > /tmp/v1.0.0.info
jfrog_upload go-local /github.com/myorg/mylib/@v/v1.0.0.info /tmp/v1.0.0.info

# ─── 10. Seed JFrog — Raw / Generic ──────────────────────────────────────────

echo "==> Seeding Raw binary…"
echo "binary-content" > /tmp/myapp-linux-amd64
jfrog_upload raw-local /binaries/myapp-linux-amd64 /tmp/myapp-linux-amd64

# ─── 11. Wait for Nexus ───────────────────────────────────────────────────────

echo "==> Waiting for Nexus…"
until curl -sf "${NEXUS_URL}/service/rest/v1/status" >/dev/null; do
    sleep 5; echo "   …"
done
echo "    Nexus ready."

# Retrieve the auto-generated admin password (first boot only)
NEXUS_PWD=$(docker exec nexus cat /nexus-data/admin.password 2>/dev/null || true)
if [ -z "${NEXUS_PWD}" ]; then
    # Password was already changed; fall back to env var
    NEXUS_PWD=${NEXUS_PASS:-"admin123"}
fi
echo "    Using Nexus password: ${NEXUS_PWD}"

# ─── 12. Create Nexus hosted repos ───────────────────────────────────────────

echo "==> Creating Nexus repositories…"

nexus_post_repo maven hosted '{
  "name":"maven-target","online":true,
  "storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"ALLOW"},
  "maven":{"versionPolicy":"MIXED","layoutPolicy":"STRICT"}
}'

nexus_post_repo docker hosted '{
  "name":"docker-target","online":true,
  "storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"ALLOW"},
  "docker":{"v1Enabled":true,"forceBasicAuth":true,"httpPort":8084}
}'

nexus_post_repo pypi hosted '{
  "name":"pypi-target","online":true,
  "storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"ALLOW"}
}'

nexus_post_repo npm hosted '{
  "name":"npm-target","online":true,
  "storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"ALLOW"}
}'

nexus_post_repo nuget hosted '{
  "name":"nuget-target","online":true,
  "storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"ALLOW"},
  "nugetProxy":{"queryCacheItemMaxAge":3600}
}'

nexus_post_repo helm hosted '{
  "name":"helm-target","online":true,
  "storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"ALLOW"}
}'

nexus_post_repo raw hosted '{
  "name":"go-target","online":true,
  "storage":{"blobStoreName":"default","strictContentTypeValidation":false,"writePolicy":"ALLOW"}
}'

nexus_post_repo raw hosted '{
  "name":"raw-target","online":true,
  "storage":{"blobStoreName":"default","strictContentTypeValidation":false,"writePolicy":"ALLOW"}
}'

# ─── 13. Save credentials for tests ──────────────────────────────────────────

echo "admin"       > tests/.nexus_user
echo "${NEXUS_PWD}" > tests/.nexus_password

echo ""
echo "============================================================"
echo "  Bootstrap complete!"
echo ""
echo "  Artifactory : ${JFROG_URL}/artifactory"
echo "                user: ${JFROG_USER} / pass: ${JFROG_PASS}"
echo "  Nexus       : ${NEXUS_URL}"
echo "                user: admin / pass: ${NEXUS_PWD}"
echo ""
echo "  Seeded repos: maven-local, docker-local, pypi-local,"
echo "                npm-local, nuget-local, helm-local,"
echo "                go-local, raw-local"
echo "============================================================"
