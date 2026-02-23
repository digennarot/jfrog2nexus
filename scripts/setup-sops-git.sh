#!/bin/bash

# Configuration script to set up git filters for sops and age
scriptDir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cd "${scriptDir}/.." || exit 1

echo "Configuring Git for SOPS encryption..."

if ! command -v sops &> /dev/null; then
  echo "WARNING: sops is not installed on this system. You will need it to encrypt/decrypt secrets."
  echo "Install sops via github-to-sops or your package manager."
fi

if ! command -v github-to-sops &> /dev/null; then
  echo "WARNING: github-to-sops is not installed. Will not be able to fetch contributor public keys."
  echo "Install it via 'pip install github-to-sops' or 'uv pip install github-to-sops'"
else
  echo "Generating/Updating .sops.yaml from GitHub contributors..."
  # If a user is not logged in or token isn't provided, this works for public repos.
  # For private repos, ensure GITHUB_TOKEN is set.
  github-to-sops --github-users tizianodigennaro import-keys > .sops.yaml
  echo ".sops.yaml successfully generated!"
  
  # Ensure sops matches .env and .nexus_password which we set in .gitattributes
  # A basic regex covering all files for our simple usecase
  sed -i 's/path_regex: .*/path_regex: ".*\\.(env|nexus_password)$"/g' .sops.yaml
fi

# Configure Git filters
git config --local filter.sops.smudge "scripts/sops-decrypt.sh %f"
git config --local filter.sops.clean "scripts/sops-encrypt.sh %f"
git config --local filter.sops.required true

echo "Git filter 'sops' configured successfully."
echo "Any files matching 'filter=sops' in .gitattributes will now be transparently encrypted on commit and decrypted on checkout."

