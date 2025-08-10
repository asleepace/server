#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/vscode"

if ! command -v npx >/dev/null 2>&1; then
  echo "npx not found; please install Node.js" >&2
  exit 1
fi

npx --yes vsce package --no-yarn
echo "VSIX built under src/client/wc/vscode/*.vsix"


