#!/bin/bash
# Copilot-compatible wrapper for ICN environment setup

set -euo pipefail
cd /workspace/icn-core || exit 1

# Run the main setup script (already fully robust)
if [ -f "./scripts/setup.sh" ]; then
  chmod +x ./scripts/setup.sh
  ./scripts/setup.sh
else
  echo "❌ scripts/setup.sh not found!"
  exit 1
fi
