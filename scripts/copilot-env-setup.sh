#!/bin/bash
# Entry script for Copilot Coding Agent

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SETUP_SCRIPT="$REPO_ROOT/scripts/setup.sh"
LOG_PATH="/tmp/icn-setup.log"

echo "🧠 Copilot setup executing in: $REPO_ROOT"

cd "$REPO_ROOT"

chmod +x "$SETUP_SCRIPT"
"$SETUP_SCRIPT"

if [ -f "$LOG_PATH" ]; then
  echo "📄 Last 20 lines of log:"
  tail -n 20 "$LOG_PATH"
fi
