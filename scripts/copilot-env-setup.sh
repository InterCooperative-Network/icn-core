#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SETUP_SCRIPT="$REPO_ROOT/scripts/setup.sh"
LOG_PATH="/tmp/icn-setup.log"

echo "🧠 Copilot setup starting from $0"
echo "📁 Repo root: $REPO_ROOT"

cd "$REPO_ROOT"

if [ ! -x "$SETUP_SCRIPT" ]; then
  echo "🔧 Making setup script executable"
  chmod +x "$SETUP_SCRIPT"
fi

echo "🚀 Running ICN setup script..."
"$SETUP_SCRIPT"

if [ -f "$LOG_PATH" ]; then
  echo "📄 Setup log tail:"
  tail -n 20 "$LOG_PATH"
fi
