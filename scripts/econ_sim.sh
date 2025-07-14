#!/bin/bash
# Run a simple economic simulation using the icn-economics example
# Usage: ./scripts/econ_sim.sh [iterations] [ledger_path]
set -euo pipefail
ITER=${1:-10}
LEDGER=${2:-/tmp/econ_sim_ledger.json}

cargo run --package icn-economics --example simulate -- ${LEDGER} ${ITER}
