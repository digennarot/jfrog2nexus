#!/bin/bash

scriptDir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cd "${scriptDir}/.." || exit 1

if ! command -v sops &> /dev/null; then
  # If sops is missing, we just return the raw (encrypted) content
  cat
  exit 0
fi

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
  input_type="binary"
fi

# Git passes the encrypted content via stdin to be smudged (decrypted)
temp_file=$(mktemp)
cat > "$temp_file"

# Check if the file is actually sops encrypted (usually has sops metadata)
# For binary it's a JSON with "sops" key. For others it's inline.
if ! grep -q "sops" "$temp_file"; then
  # File is not encrypted, just pass it through
  cat "$temp_file"
  rm -f "$temp_file"
  exit 0
fi

sops --decrypt \
     --input-type "$input_type" \
     --output-type "$input_type" \
     "$temp_file"

exit_code=$?
rm -f "$temp_file"
# If decryption fails (e.g., no key), exit code will be > 0. Git will just fail the checkout or smudge.
exit $exit_code

