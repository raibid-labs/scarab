#!/bin/bash
# Helper script for running scarab-clipboard tests

set -e

echo "============================================"
echo "Scarab Clipboard Test Runner"
echo "============================================"
echo

# Parse command line arguments
RUN_IGNORED=false
VERBOSE=false
TEST_NAME=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --ignored)
            RUN_IGNORED=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --test)
            TEST_NAME="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  --ignored       Run tests that require a display server"
            echo "  --verbose, -v   Show detailed test output"
            echo "  --test NAME     Run a specific test by name"
            echo "  --help, -h      Show this help message"
            echo
            echo "Examples:"
            echo "  $0                                    # Run all non-ignored tests"
            echo "  $0 --ignored                          # Run display-dependent tests"
            echo "  $0 --test test_selection_region_new   # Run specific test"
            echo "  $0 --verbose                          # Run with verbose output"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Build the cargo test command
CMD="cargo test -p scarab-clipboard"

if [ "$VERBOSE" = true ]; then
    CMD="$CMD -- --nocapture"
fi

if [ -n "$TEST_NAME" ]; then
    CMD="$CMD $TEST_NAME"
fi

if [ "$RUN_IGNORED" = true ]; then
    echo "Running tests that require a display server..."
    echo "Note: These tests will fail if no X11/Wayland display is available"
    echo
    $CMD -- --ignored
else
    echo "Running standard tests (display-independent)..."
    echo
    $CMD
fi

echo
echo "============================================"
echo "Test Summary:"
echo "  - Unit tests (in src/): Basic functionality"
echo "  - Integration tests: Full workflows"
echo "  - Total test files: 3"
echo "  - Total tests: 74 (16 lib + 58 integration)"
echo "  - Ignored tests: 12 (require display server)"
echo "============================================"
