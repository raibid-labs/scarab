#!/usr/bin/env nu
# Navigation Smoke Test Script (Nushell version)
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
#
# Usage:
#   nu scripts/nav-smoke-test.nu

# Color utilities
def color-green [text: string] {
    $"(ansi green)($text)(ansi reset)"
}

def color-red [text: string] {
    $"(ansi red)($text)(ansi reset)"
}

def color-yellow [text: string] {
    $"(ansi yellow)($text)(ansi reset)"
}

def color-blue [text: string] {
    $"(ansi blue)($text)(ansi reset)"
}

# Test suite runner
def run-test-suite [
    name: string,
    command: string,
    exit_code: int
] {
    print $"(color-yellow $'Running: ($name)')"
    print $"(color-blue $'Command: ($command)')"
    print ""

    let result = (do -i { nu -c $command } | complete)

    if $result.exit_code == 0 {
        print $"(color-green $'✓ ($name) PASSED')"
        print ""
        { passed: true }
    } else {
        print $"(color-red $'✗ ($name) FAILED')"
        print $result.stderr
        print ""
        exit $exit_code
    }
}

# Main execution
def main [] {
    print (color-blue "======================================")
    print (color-blue "Navigation System Smoke Tests")
    print (color-blue "======================================")
    print ""

    # Test counters
    mut tests_run = 0
    mut tests_passed = 0

    # 1. Navigation unit tests
    $tests_run = $tests_run + 1
    let test1 = (run-test-suite
        "Navigation Unit Tests"
        "cargo test -p scarab-client --lib navigation::tests --quiet"
        1
    )
    if $test1.passed {
        $tests_passed = $tests_passed + 1
    }

    # 2. Golden tests (snapshot validation)
    $tests_run = $tests_run + 1
    let test2 = (run-test-suite
        "Golden Tests (Snapshot Validation)"
        "cargo test -p scarab-client --test golden_tests --quiet"
        2
    )
    if $test2.passed {
        $tests_passed = $tests_passed + 1
    }

    # 3. Headless harness tests
    $tests_run = $tests_run + 1
    let test3 = (run-test-suite
        "Headless Harness Tests"
        "cargo test -p scarab-client --test headless_harness --quiet"
        3
    )
    if $test3.passed {
        $tests_passed = $tests_passed + 1
    }

    # Summary
    print (color-blue "======================================")
    print (color-blue "Test Summary")
    print (color-blue "======================================")
    print $"Total test suites run: ($tests_run)"
    print $"(color-green $'Passed: ($tests_passed)')"
    print $"(color-red $'Failed: ($tests_run - $tests_passed)')"
    print ""

    if $tests_passed == $tests_run {
        print (color-green "All navigation tests passed!")
        print (color-green "Navigation system is validated and ready for CI.")
        exit 0
    } else {
        print (color-red "Some tests failed. Please review the output above.")
        exit 1
    }
}
