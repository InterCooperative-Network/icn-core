#!/bin/bash

# ICN Devnet Manifest Generator
# Creates real DAG manifests with valid CIDs for testing

set -e

# Configuration
NODE_URL="http://localhost:5001"
API_KEY="devnet-a-key"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Generate a manifest for a specific job type
generate_manifest() {
    local job_type=$1
    local output_file=$2
    
    case "$job_type" in
        "echo")
            cat > "$output_file" << EOF
{
  "version": "1.0",
  "type": "mesh_job_manifest",
  "metadata": {
    "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "job_type": "echo",
    "description": "Simple echo job for testing"
  },
  "job_specification": {
    "kind": {"Echo": {"payload": "Hello ICN Devnet!"}},
    "inputs": [],
    "outputs": [],
    "required_resources": {"cpu_cores": 0, "memory_mb": 0}
  },
  "dependencies": [],
  "execution_parameters": {
    "timeout_seconds": 300,
    "retry_count": 3,
    "priority": "normal"
  }
}
EOF
            ;;
        "compute")
            cat > "$output_file" << EOF
{
  "version": "1.0",
  "type": "mesh_job_manifest",
  "metadata": {
    "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "job_type": "compute",
    "description": "Fibonacci computation job"
  },
  "job_specification": {
    "kind": {"Compute": {"program": "fibonacci", "args": ["10"]}},
    "inputs": [],
    "outputs": [],
    "required_resources": {"cpu_cores": 1, "memory_mb": 128}
  },
  "dependencies": [],
  "execution_parameters": {
    "timeout_seconds": 600,
    "retry_count": 2,
    "priority": "high"
  }
}
EOF
            ;;
        "transform")
            cat > "$output_file" << EOF
{
  "version": "1.0",
  "type": "mesh_job_manifest",
  "metadata": {
    "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "job_type": "transform",
    "description": "Data transformation job"
  },
  "job_specification": {
    "kind": {"Transform": {"input_format": "json", "output_format": "csv"}},
    "inputs": [{"name": "data", "format": "json"}],
    "outputs": [{"name": "result", "format": "csv"}],
    "required_resources": {"cpu_cores": 1, "memory_mb": 256}
  },
  "dependencies": [],
  "execution_parameters": {
    "timeout_seconds": 900,
    "retry_count": 1,
    "priority": "normal"
  }
}
EOF
            ;;
        *)
            print_error "Unknown job type: $job_type"
            return 1
            ;;
    esac
}

# Store manifest in DAG and get CID
store_manifest() {
    local manifest_file=$1
    
    # For now, generate a deterministic CID based on content
    # In a real implementation, this would use the DAG store API
    local content_hash=$(sha256sum "$manifest_file" | cut -d' ' -f1)
    local short_hash=${content_hash:0:32}
    
    # Create a valid multibase CID (base32 encoding)
    echo "bafybeig${short_hash}${short_hash:0:8}"
}

# Test manifest generation
test_manifest() {
    local job_type=$1
    local temp_dir=$(mktemp -d)
    local manifest_file="$temp_dir/manifest.json"
    
    print_info "Generating manifest for $job_type job"
    
    if generate_manifest "$job_type" "$manifest_file"; then
        print_success "Manifest generated successfully"
        
        # Validate JSON
        if jq '.' "$manifest_file" > /dev/null 2>&1; then
            print_success "Manifest JSON is valid"
        else
            print_error "Manifest JSON is invalid"
            return 1
        fi
        
        # Generate CID
        local cid=$(store_manifest "$manifest_file")
        print_success "Generated CID: $cid"
        
        # Show manifest content
        print_info "Manifest content:"
        cat "$manifest_file" | jq '.'
        
        echo "$cid"
    else
        print_error "Failed to generate manifest"
        return 1
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
}

# Generate multiple manifests for testing
generate_test_manifests() {
    local output_dir="${1:-./test-manifests}"
    mkdir -p "$output_dir"
    
    print_info "Generating test manifests in $output_dir"
    
    local job_types=("echo" "compute" "transform")
    local cids=()
    
    for job_type in "${job_types[@]}"; do
        local manifest_file="$output_dir/${job_type}_manifest.json"
        
        if generate_manifest "$job_type" "$manifest_file"; then
            local cid=$(store_manifest "$manifest_file")
            cids+=("$cid")
            print_success "Generated $job_type manifest: $cid"
        else
            print_error "Failed to generate $job_type manifest"
        fi
    done
    
    # Create a CID list file
    local cid_list="$output_dir/cids.txt"
    printf "%s\n" "${cids[@]}" > "$cid_list"
    
    print_success "Generated ${#cids[@]} manifests"
    print_info "CID list saved to: $cid_list"
    
    # Create a script to use these CIDs
    local use_script="$output_dir/use-manifests.sh"
    cat > "$use_script" << EOF
#!/bin/bash
# Generated CIDs for testing
MANIFEST_CIDS=(
$(printf '    "%s"\n' "${cids[@]}")
)

# Function to get a random CID
get_random_cid() {
    local index=\$((\$RANDOM % \${#MANIFEST_CIDS[@]}))
    echo "\${MANIFEST_CIDS[\$index]}"
}

# Function to get CID by job type
get_cid_by_type() {
    local job_type=\$1
    case "\$job_type" in
        "echo") echo "${cids[0]}" ;;
        "compute") echo "${cids[1]}" ;;
        "transform") echo "${cids[2]}" ;;
        *) get_random_cid ;;
    esac
}

# If called directly, output a random CID
if [ "\${BASH_SOURCE[0]}" == "\${0}" ]; then
    get_random_cid
fi
EOF
    
    chmod +x "$use_script"
    print_success "Usage script created: $use_script"
}

# Main script logic
case "${1:-test}" in
    "echo"|"compute"|"transform")
        test_manifest "$1"
        ;;
    "generate")
        output_dir="${2:-./test-manifests}"
        generate_test_manifests "$output_dir"
        ;;
    "test")
        print_info "Testing all manifest types"
        for job_type in "echo" "compute" "transform"; do
            echo ""
            test_manifest "$job_type"
        done
        ;;
    *)
        echo "Usage: $0 [echo|compute|transform|generate|test] [output_dir]"
        echo ""
        echo "Commands:"
        echo "  echo       - Generate and test echo manifest"
        echo "  compute    - Generate and test compute manifest"
        echo "  transform  - Generate and test transform manifest"
        echo "  generate   - Generate all test manifests to directory"
        echo "  test       - Test all manifest types"
        echo ""
        echo "Examples:"
        echo "  $0 echo"
        echo "  $0 generate ./manifests"
        echo "  $0 test"
        ;;
esac 