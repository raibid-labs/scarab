//! Test audit runner for Issue #173
//!
//! This test scans the codebase for placeholder tests and generates a report.

mod test_auditor_impl;

use anyhow::Result;
use test_auditor_impl::TestAuditor;

#[test]
fn audit_daemon_tests() -> Result<()> {
    let mut auditor = TestAuditor::new();

    // Scan daemon tests directory
    let daemon_tests_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scarab-daemon/tests");
    auditor.scan_directory(daemon_tests_path)?;

    let placeholders = auditor.find_placeholder_tests();

    if !placeholders.is_empty() {
        println!("\n{}", auditor.report());
        println!("\nFound {} placeholder tests in scarab-daemon", placeholders.len());

        for test in placeholders {
            println!("\nReplacement template for '{}':", test.name);
            println!("{}", auditor.generate_replacement_template(test));
        }
    } else {
        println!("No placeholder tests found in scarab-daemon - all tests look good!");
    }

    Ok(())
}

#[test]
fn audit_client_tests() -> Result<()> {
    let mut auditor = TestAuditor::new();

    // Scan client tests directory (but exclude our own test files)
    let client_tests_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");
    auditor.scan_directory(client_tests_path)?;

    // Filter out our own test auditor files
    let placeholders: Vec<_> = auditor
        .find_placeholder_tests()
        .iter()
        .filter(|t| !t.file.contains("test_auditor") && !t.file.contains("run_test_audit"))
        .cloned()
        .collect();

    if !placeholders.is_empty() {
        println!("\nFound {} placeholder tests in scarab-client:", placeholders.len());

        for test in &placeholders {
            println!("  - {} ({}:{}) - {:?}", test.name, test.file, test.line, test.reason);
        }

        println!("\nGenerate reports with: cargo test audit_client_tests -- --nocapture");
    } else {
        println!("No placeholder tests found in scarab-client - all tests look good!");
    }

    Ok(())
}

#[test]
fn audit_all_tests() -> Result<()> {
    let mut auditor = TestAuditor::new();

    // Scan both daemon and client tests
    let daemon_tests = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scarab-daemon/tests");
    let client_tests = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");

    auditor.scan_directory(daemon_tests)?;
    auditor.scan_directory(client_tests)?;

    // Filter out our own test auditor files
    let all_placeholders: Vec<_> = auditor
        .find_placeholder_tests()
        .iter()
        .filter(|t| {
            !t.file.contains("test_auditor")
                && !t.file.contains("run_test_audit")
                && !t.file.contains("ratatui_testlib_test_auditor")
        })
        .cloned()
        .collect();

    if all_placeholders.is_empty() {
        println!("\n=== SUCCESS ===");
        println!("No placeholder tests found in the Scarab codebase!");
        println!("All tests have meaningful assertions.");
    } else {
        println!("\n=== TEST QUALITY AUDIT REPORT ===\n");
        println!("Found {} placeholder tests that need attention:\n", all_placeholders.len());

        for test in &all_placeholders {
            println!("  - {} ({}:{}) - {:?}", test.name, test.file, test.line, test.reason);
        }

        println!("\n=== RECOMMENDATIONS ===");
        println!("1. Review each placeholder test");
        println!("2. Replace with meaningful assertions");
        println!("3. Verify tests actually test intended behavior");
        println!("\nRun individual audit tests with --nocapture to see replacement templates");
    }

    // Don't fail the test - this is informational
    Ok(())
}
