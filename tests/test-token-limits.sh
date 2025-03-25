#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Testing plz-cli token limits${NC}\n"

# Function to test command and check output
test_command() {
    local desc="$1"
    local cmd="$2"
    local expected_status="$3"

    echo -e "${BLUE}Testing: ${desc}${NC}"
    echo "Command: $cmd"

    eval "$cmd" > /tmp/plz_test_output 2>&1
    local status=$?

    if [ $status -eq "$expected_status" ]; then
        echo -e "${GREEN}✓ Test passed${NC}"
    else
        echo -e "${RED}✗ Test failed - Expected status $expected_status, got $status${NC}"
        echo -e "${RED}Output:${NC}"
        cat /tmp/plz_test_output
    fi
    echo
}

# Test 1: Simple command (should work in default mode)
test_command "Simple command with default tokens" \
    "cargo run -- -y 'show me the current directory contents'" \
    0

# Test 2: Moderately complex command (should work in default mode)
test_command "Moderate complexity with default tokens" \
    "cargo run -- -y 'create a script that finds all markdown files in the current directory, extracts their headers, and creates a table of contents'" \
    0

# Test 3: Complex command without --extended (testing token limit)
test_command "Complex command without extended tokens" \
    "cargo run -- -y 'write a script that lists all files recursively, calculates their size, shows file types, creates a summary grouped by extension, sorts by size, and displays the top 10 largest files with percentage of total disk usage'" \
    0

# Test 4: Simple long command with --extended
test_command "Simple command with extended tokens" \
    "cargo run -- -y --extended 'write a script that lists all files recursively, calculates their size, shows file types, creates a summary grouped by extension, sorts by size, and displays the top 10 largest files with percentage of total disk usage'" \
    0

# Clean up
rm /tmp/plz_test_output

echo -e "${BLUE}All tests completed${NC}"
