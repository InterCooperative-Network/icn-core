#!/bin/bash
# Run P2P+DAG End-to-End Integration Tests
# This script runs comprehensive tests for P2P networking and DAG storage integration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
VERBOSE=false
QUICK=false
SINGLE_TEST=""
FEATURES="enable-libp2p"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -q|--quick)
            QUICK=true
            shift
            ;;
        -t|--test)
            SINGLE_TEST="$2"
            shift 2
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -v, --verbose     Enable verbose output"
            echo "  -q, --quick       Run quick subset of tests"
            echo "  -t, --test NAME   Run specific test by name"
            echo "  --features FEAT   Specify features (default: enable-libp2p)"
            echo "  -h, --help        Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                                    # Run all P2P+DAG tests"
            echo "  $0 --verbose                          # Run with verbose output"
            echo "  $0 --quick                            # Run quick subset"
            echo "  $0 --test test_multi_node_dag_synchronization  # Run specific test"
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}=== ICN P2P+DAG End-to-End Integration Tests ===${NC}"
echo ""

# Check if running from correct directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "crates" ]]; then
    echo -e "${RED}Error: Please run this script from the icn-core project root${NC}"
    exit 1
fi

# Check if required features are available
if ! grep -q "enable-libp2p" Cargo.toml tests/Cargo.toml crates/*/Cargo.toml 2>/dev/null; then
    echo -e "${YELLOW}Warning: enable-libp2p feature may not be available${NC}"
fi

# Build first to catch compilation errors early
echo -e "${BLUE}Building project with features: ${FEATURES}${NC}"
if [[ "$VERBOSE" == "true" ]]; then
    cargo build --features "$FEATURES" --workspace
else
    cargo build --features "$FEATURES" --workspace >/dev/null 2>&1
fi

if [[ $? -ne 0 ]]; then
    echo -e "${RED}Build failed. Please fix compilation errors first.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Set up test environment
export RUST_LOG="${RUST_LOG:-info}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

# Cleanup any previous test artifacts
echo -e "${BLUE}Cleaning up previous test artifacts...${NC}"
rm -rf ./dag_* ./mana_* ./rep_* ./test_* 2>/dev/null || true

# Define test suite
if [[ "$QUICK" == "true" ]]; then
    TESTS=(
        "test_multi_node_dag_synchronization"
        "test_cross_node_receipt_anchoring"
    )
    echo -e "${YELLOW}Running quick test suite (${#TESTS[@]} tests)${NC}"
elif [[ -n "$SINGLE_TEST" ]]; then
    TESTS=("$SINGLE_TEST")
    echo -e "${YELLOW}Running single test: $SINGLE_TEST${NC}"
else
    TESTS=(
        "test_multi_node_dag_synchronization"
        "test_cross_node_receipt_anchoring"
        "test_dag_fork_resolution"
        "test_network_partition_recovery"
        "test_performance_under_load"
        "test_dag_integrity_validation"
        "test_comprehensive_p2p_dag_integration"
    )
    echo -e "${YELLOW}Running full test suite (${#TESTS[@]} tests)${NC}"
fi

echo ""

# Run the tests
TEST_RESULTS=()
FAILED_TESTS=()
START_TIME=$(date +%s)

for test in "${TESTS[@]}"; do
    echo -e "${BLUE}Running: $test${NC}"
    
    if [[ "$VERBOSE" == "true" ]]; then
        TEST_CMD="cargo test --package icn-integration-tests --features $FEATURES --test p2p_dag_e2e $test -- --nocapture"
    else
        TEST_CMD="cargo test --package icn-integration-tests --features $FEATURES --test p2p_dag_e2e $test"
    fi
    
    TEST_START=$(date +%s)
    
    if eval "$TEST_CMD" 2>&1; then
        TEST_END=$(date +%s)
        TEST_DURATION=$((TEST_END - TEST_START))
        echo -e "${GREEN}✓ $test (${TEST_DURATION}s)${NC}"
        TEST_RESULTS+=("✓ $test (${TEST_DURATION}s)")
    else
        TEST_END=$(date +%s)
        TEST_DURATION=$((TEST_END - TEST_START))
        echo -e "${RED}✗ $test (${TEST_DURATION}s)${NC}"
        TEST_RESULTS+=("✗ $test (${TEST_DURATION}s)")
        FAILED_TESTS+=("$test")
    fi
    
    echo ""
    
    # Small delay between tests to allow cleanup
    sleep 1
done

# Cleanup test artifacts
echo -e "${BLUE}Cleaning up test artifacts...${NC}"
rm -rf ./dag_* ./mana_* ./rep_* ./test_* 2>/dev/null || true

# Report results
END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))

echo -e "${BLUE}=== Test Results ===${NC}"
echo ""

for result in "${TEST_RESULTS[@]}"; do
    echo -e "  $result"
done

echo ""
echo -e "${BLUE}Total execution time: ${TOTAL_DURATION}s${NC}"

if [[ ${#FAILED_TESTS[@]} -eq 0 ]]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ ${#FAILED_TESTS[@]} test(s) failed:${NC}"
    for failed in "${FAILED_TESTS[@]}"; do
        echo -e "  ${RED}- $failed${NC}"
    done
    echo ""
    echo -e "${YELLOW}To debug a specific test, run:${NC}"
    echo -e "  cargo test --package icn-integration-tests --features $FEATURES --test p2p_dag_e2e <test_name> -- --nocapture"
    exit 1
fi 