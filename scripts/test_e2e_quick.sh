#!/bin/bash

# Quick E2E test runner
echo "🚀 Running ICN Comprehensive E2E Test..."

# Run the test with timeout
timeout 300s cargo test --features enable-libp2p -p icn-integration-tests --test comprehensive_e2e comprehensive_mesh_job_e2e_test -- --nocapture 2>&1

# Capture the exit code
exit_code=$?

if [ $exit_code -eq 124 ]; then
    echo "⏰ Test timed out after 300 seconds"
elif [ $exit_code -eq 0 ]; then
    echo "✅ Test passed successfully!"
else
    echo "❌ Test failed with exit code: $exit_code"
fi

exit $exit_code 