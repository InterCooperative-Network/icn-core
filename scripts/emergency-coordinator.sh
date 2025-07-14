#!/bin/bash
# Simple emergency coordination helper
# Lists registered aid resources and allows quick request submission

set -e

API_URL="${ICN_API_URL:-http://127.0.0.1:7845}"

function usage() {
    echo "Usage: $0 list|request <resource-id>"
}

case "$1" in
    list)
        curl -s "$API_URL/aid/resources" || echo "Failed to fetch resources"
        ;;
    request)
        if [ -z "$2" ]; then
            usage
            exit 1
        fi
        curl -s -X POST "$API_URL/aid/request" -d "{\"id\":\"$2\"}" \
            -H 'Content-Type: application/json'
        ;;
    *)
        usage
        exit 1
        ;;
esac
