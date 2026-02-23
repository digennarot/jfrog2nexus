#!/bin/bash

# Ensure we are in the project root
scriptDir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cd "${scriptDir}/.." || exit 1

if ! command -v sops &> /dev/null; then
  echo "WARNING: sops is not installed. Passing file unencrypted." >&2
  cat
  exit 0
fi

# The filename is passed by git as the first argument
filename="$1"
extension="${filename##*.}"
filename_base="${filename##*/}"

if [ "$extension" == "env" ] || [[ "$filename_base" == *".env"* ]]; then
  input_type="dotenv"
elif [ "$extension" == "json" ]; then
  input_type="json"
elif [ "$extension" == "yaml" ] || [ "$extension" == "yml" ]; then
  input_type="yaml"
else
  # Default to binary for unstructured files like .nexus_password
  input_type="binary"
fi

if [ ! -f ".sops.yaml" ]; then
  echo "ERROR: .sops.yaml not found! Run setup-sops-git.sh first." >&2
  exit 1
fi

# Extract age keys from .sops.yaml since sops < 3.10 doesn't support --filename for stdin cleanly
SOPS_AGE_RECIPIENTS=$(grep -Eo 'age1[a-z0-9]{58}' .sops.yaml | tr '\n' ',' | sed 's/,$//')

if [ -z "$SOPS_AGE_RECIPIENTS" ]; then
    echo "ERROR: No age recipients found in .sops.yaml! Cannot encrypt $filename" >&2
    exit 1
fi

# We must read from stdin, because git filter passes the file content via stdin.
temp_file=$(mktemp)
cat > "$temp_file"

sops --encrypt \
     --input-type "$input_type" \
     --output-type "$input_type" \
     --age "${SOPS_AGE_RECIPIENTS}" \
     "$temp_file"

exit_code=$?
rm -f "$temp_file"
exit $exit_code

