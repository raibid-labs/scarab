#!/bin/bash
# Navigation Smoke Test Script
#
# This script runs comprehensive navigation system tests in headless mode
# for CI validation. It tests:
# - Navigation unit tests (mode switching, focus management)
# - Golden tests (snapshot validation)
# - Headless harness tests (E2E navigation workflows)
#
# Exit codes:
#   0 - All tests passed
#   1 - Navigation unit tests failed
#   2 - Golden tests failed
#   3 - Headless harness tests failed
#   4 - Build failed
#
# Usage:
#   ./scripts/nav-smoke-test.sh

set -e  # Exit on first error

# Color output for better readability
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Navigation System Smoke Tests${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

# Test counter
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test suite
run_test_suite() {
    local name="$1"
    local command="$2"
    local exit_code="$3"

    echo -e "${YELLOW}Running: ${name}${NC}"
    echo -e "${BLUE}Command: ${command}${NC}"
    echo ""

    TESTS_RUN=$((TESTS_RUN + 1))

    if eval "${command}"; then
        echo -e "${GREEN}✓ ${name} PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        echo ""
        return 0
    else
        echo -e "${RED}✗ ${name} FAILED${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        echo ""
        exit "${exit_code}"
    fi
}

# 1. Run navigation unit tests
# Note: We run tests directly without a separate build step since cargo test will build automatically
run_test_suite \
    "Navigation Unit Tests" \
    "cargo test -p scarab-client --lib navigation::tests --quiet" \
    1

# 2. Run golden tests (snapshot validation)
run_test_suite \
    "Golden Tests (Snapshot Validation)" \
    "cargo test -p scarab-client --test golden_tests --quiet" \
    2

# 3. Run headless harness tests
run_test_suite \
    "Headless Harness Tests" \
    "cargo test -p scarab-client --test headless_harness --quiet" \
    3

# Summary
echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}======================================${NC}"
echo -e "Total test suites run: ${TESTS_RUN}"
echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"
echo ""

if [ "${TESTS_FAILED}" -eq 0 ]; then
    echo -e "${GREEN}All navigation tests passed!${NC}"
    echo -e "${GREEN}Navigation system is validated and ready for CI.${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Please review the output above.${NC}"
    exit 1
fi
