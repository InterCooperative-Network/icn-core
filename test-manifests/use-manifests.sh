#!/bin/bash
# Generated CIDs for testing
MANIFEST_CIDS=(
    "bafybeigf525f8d264f8955cd6f139e5fbaaaa4cf525f8d2"
    "bafybeigf33150c7de0d868abeb99dba4ef5c31cf33150c7"
    "bafybeig8c645c932daa4a764a92ae52742256788c645c93"
)

# Function to get a random CID
get_random_cid() {
    local index=$(($RANDOM % ${#MANIFEST_CIDS[@]}))
    echo "${MANIFEST_CIDS[$index]}"
}

# Function to get CID by job type
get_cid_by_type() {
    local job_type=$1
    case "$job_type" in
        "echo") echo "bafybeigf525f8d264f8955cd6f139e5fbaaaa4cf525f8d2" ;;
        "compute") echo "bafybeigf33150c7de0d868abeb99dba4ef5c31cf33150c7" ;;
        "transform") echo "bafybeig8c645c932daa4a764a92ae52742256788c645c93" ;;
        *) get_random_cid ;;
    esac
}

# If called directly, output a random CID
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    get_random_cid
fi
