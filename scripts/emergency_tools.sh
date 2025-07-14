#!/bin/bash
set -euo pipefail
if [ $# -lt 1 ]; then
    echo "Usage: $0 register <file>|list" >&2
    exit 1
fi
cmd=$1
shift
case "$cmd" in
    register)
        cargo run -p icn-cli -- emergency register-resource "${1:--}"
        ;;
    list)
        cargo run -p icn-cli -- emergency list-resources
        ;;
    *)
        echo "Unknown command: $cmd" >&2
        exit 1
        ;;
esac
