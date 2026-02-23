#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." >/dev/null 2>&1 && pwd)"
TEST_DIR=$(mktemp -d)

echo "Running SOPS integration test in $TEST_DIR"
cd "$TEST_DIR"
git init --initial-branch=main >/dev/null 2>&1 || git init >/dev/null

mkdir scripts
cp "$REPO_ROOT"/scripts/sops-* scripts/

echo ".env filter=sops" > .gitattributes
git add .gitattributes
git commit -m "Add attributes" >/dev/null

# Mock setup rather than running the script which expects github connectivity
git config --local filter.sops.smudge "scripts/sops-decrypt.sh %f"
git config --local filter.sops.clean "scripts/sops-encrypt.sh %f"
git config --local filter.sops.required true

if command -v age-keygen &> /dev/null; then
  mkdir -p secrets
  age-keygen -o secrets/age-key.txt 2>/dev/null
  PUB_KEY=$(grep "public key:" secrets/age-key.txt | cut -d: -f2 | tr -d ' ')
  cat <<EOF > .sops.yaml
creation_rules:
  - key_groups:
      - age:
        - $PUB_KEY
EOF
fi

echo "SUPER_SECRET=12345" > .env
git add .env

if ! command -v sops &> /dev/null || ! command -v github-to-sops &> /dev/null; then
  git commit -m "Add secret" >/dev/null
  echo "SOPS/github-to-sops missing, verifying fallback behavior..."
  if git cat-file -p HEAD:.env | grep -q "SUPER_SECRET=12345"; then
    echo "Fallback successful: passed unstructured"
    exit 0
  else
    echo "Fallback failed"
    exit 1
  fi
else
  # If we have sops installed but no age, we can't test encryption properly
  if ! command -v age-keygen &> /dev/null; then
    echo "Requires age-keygen to mock local keys. Skipping full encryption test."
    exit 0
  fi
  
  git commit -m "Add secret" >/dev/null
  
  echo "Verifying encryption in Git object..."
  # The file in Git shouldn't contain the raw secret
  if git cat-file -p HEAD:.env | grep -q "SUPER_SECRET=12345"; then
    echo "Test failed: Git object is in plaintext! Filter didn't run or failed."
    exit 1
  # The file in Git should contain sops metadata
  elif git cat-file -p HEAD:.env | grep -q "sops"; then
    echo "Test passed: Git object is SOPS-encrypted."
  else
    echo "Test passed conditionally (encrypted, no sops tag)."
  fi
  
  # The local file should STILL be plaintext
  if grep -q "SUPER_SECRET=12345" .env; then
    echo "Local workspace file behaves normally."
  else
    echo "Test failed: Local file got encrypted!"
    exit 1
  fi
fi

rm -rf "$TEST_DIR"
echo "SOPS filter tested successfully!"
