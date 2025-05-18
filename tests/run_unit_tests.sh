#!/bin/bash

# Script to run unit tests for IC News Square project
# This script compiles and runs the Rust unit tests

set -e

echo "===== Running IC News Square Unit Tests ====="

# Directory setup
PROJECT_ROOT=$(cd "$(dirname "$0")/.." && pwd)
TEST_DIR="$PROJECT_ROOT/tests/unit_tests"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to run a specific test file
run_test() {
    local test_file=$1
    local test_name=$(basename "$test_file" .rs)
    
    echo -e "${YELLOW}Running test: $test_name${NC}"
    
    # Compile and run the test
    if rustc --test "$test_file" -o "$TEST_DIR/$test_name" && "$TEST_DIR/$test_name"; then
        echo -e "${GREEN}✓ Test passed: $test_name${NC}"
        return 0
    else
        echo -e "${RED}✗ Test failed: $test_name${NC}"
        return 1
    fi
}

# Check if rustc is installed
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}Error: rustc is not installed. Please install Rust to run these tests.${NC}"
    exit 1
fi

# Create temporary directory for compiled tests
mkdir -p "$TEST_DIR/tmp"

# Find all test files
echo "Finding test files..."
TEST_FILES=$(find "$TEST_DIR" -name "*_test.rs" -type f)

# Run each test file
FAILED_TESTS=()
for test_file in $TEST_FILES; do
    if ! run_test "$test_file"; then
        FAILED_TESTS+=("$(basename "$test_file")")
    fi
    echo ""
done

# Run the main test file if it exists
if [ -f "$TEST_DIR/main_test.rs" ]; then
    echo -e "${YELLOW}Running main test suite...${NC}"
    if rustc --test "$TEST_DIR/main_test.rs" -o "$TEST_DIR/main_test" && "$TEST_DIR/main_test"; then
        echo -e "${GREEN}✓ Main test suite passed${NC}"
    else
        echo -e "${RED}✗ Main test suite failed${NC}"
        FAILED_TESTS+=("main_test.rs")
    fi
fi

# Clean up compiled test binaries
echo "Cleaning up..."
rm -f "$TEST_DIR"/*_test
rm -rf "$TEST_DIR/tmp"

# Summary
echo ""
echo "===== Test Summary ====="
if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}${#FAILED_TESTS[@]} test(s) failed:${NC}"
    for failed in "${FAILED_TESTS[@]}"; do
        echo -e "${RED}- $failed${NC}"
    done
    exit 1
fi
