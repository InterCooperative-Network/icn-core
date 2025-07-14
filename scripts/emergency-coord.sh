#!/bin/bash
# Simple helper to match aid requests with job templates using icn-cli

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <requests.json> <templates.json>"
    exit 1
fi

cargo run -p icn-cli --quiet -- emergency match "$1" "$2"
