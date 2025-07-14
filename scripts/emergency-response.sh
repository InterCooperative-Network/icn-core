#!/bin/bash
# Simple helper for coordinating emergency aid via the ICN CLI

set -euo pipefail

CMD=${1:-help}
API_URL=${ICN_API_URL:-"http://127.0.0.1:7845"}

case "$CMD" in
  list)
    icn-cli --api-url "$API_URL" emergency list
    ;;
  request)
    shift
    PAYLOAD=${1:-"-"}
    icn-cli --api-url "$API_URL" emergency request "$PAYLOAD"
    ;;
  *)
    echo "Usage: $0 [list|request <json-or-'-'>]" >&2
    exit 1
    ;;
esac
