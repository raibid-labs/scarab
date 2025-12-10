//! Manual implementation of TestAuditor functionality
//!
//! Since ratatui-testlib v0.5.0 with TestAuditor is not yet available,
//! this module provides a basic implementation to scan for placeholder tests.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceholderReason {
    AlwaysPasses,        // assert!(true), assert_eq!(1, 1), etc.
    TodoOrUnimplemented, // todo!(), unimplemented!()
    EmptyBody,           // Test function with only whitespace/comments
    NoAssertions,        // No assert!() calls found
}

#[derive(Debug, Clone)]
pub struct PlaceholderTest {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub reason: PlaceholderReason,
}

pub struct TestAuditor {
    placeholder_tests: Vec<PlaceholderTest>,
}

impl TestAuditor {
    pub fn new() -> Self {
        Self {
            placeholder_tests: Vec::new(),
        }
    }

    /// Scan a directory for test files and identify placeholders
    pub fn scan_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(());
        }

        self.scan_directory_recursive(path)?;
        Ok(())
    }

    fn scan_directory_recursive(&mut self, path: &Path) -> Result<()> {
        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            self.scan_file(path)?;
        } else if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                self.scan_directory_recursive(&path)?;
            }
        }
        Ok(())
    }

    fn scan_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let file_str = path.display().to_string();

        // Look for test functions
        let mut in_test_fn = false;
        let mut test_name = String::new();
        let mut test_start_line = 0;
        let mut test_body = String::new();
        let mut brace_depth = 0;
        let mut found_test_attr = false;

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;
            let trimmed = line.trim();

            // Check for #[test] attribute
            if trimmed.starts_with("#[test]") || trimmed.starts_with("#[tokio::test]") {
                found_test_attr = true;
                continue;
            }

            // Check for function definition after #[test]
            if found_test_attr && trimmed.starts_with("fn ") {
                if let Some(name_end) = trimmed.find('(') {
                    test_name = trimmed[3..name_end].trim().to_string();
                    test_start_line = line_num;
                    in_test_fn = true;
                    found_test_attr = false;
                    test_body.clear();
                    brace_depth = 0;
                }
            }

            if in_test_fn {
                test_body.push_str(line);
                test_body.push('\n');

                // Track braces to find end of function
                for ch in line.chars() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                // End of test function
                                self.analyze_test_body(
                                    &test_name,
                                    &file_str,
                                    test_start_line,
                                    &test_body,
                                );
                                in_test_fn = false;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn analyze_test_body(&mut self, name: &str, file: &str, line: usize, body: &str) {
        // Skip ignored tests (they're often placeholders for a reason)
        if body.contains("#[ignore") {
            return;
        }

        // Check for assert!(true)
        if body.contains("assert!(true)") {
            self.placeholder_tests.push(PlaceholderTest {
                name: name.to_string(),
                file: file.to_string(),
                line,
                reason: PlaceholderReason::AlwaysPasses,
            });
            return;
        }

        // Check for assert_eq!(1, 1) or similar trivial checks
        if body.contains("assert_eq!(1, 1)")
            || body.contains("assert_eq!(0, 0)")
            || body.contains("assert_eq!(true, true)") {
            self.placeholder_tests.push(PlaceholderTest {
                name: name.to_string(),
                file: file.to_string(),
                line,
                reason: PlaceholderReason::AlwaysPasses,
            });
            return;
        }

        // Check for todo!() or unimplemented!()
        if body.contains("todo!") || body.contains("unimplemented!") {
            self.placeholder_tests.push(PlaceholderTest {
                name: name.to_string(),
                file: file.to_string(),
                line,
                reason: PlaceholderReason::TodoOrUnimplemented,
            });
            return;
        }

        // Check for empty body (only braces, whitespace, and comments)
        let body_without_comments = remove_comments(body);
        let body_code = body_without_comments
            .lines()
            .skip(1) // Skip function declaration
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && *l != "{" && *l != "}")
            .collect::<Vec<_>>()
            .join("");

        if body_code.is_empty() {
            self.placeholder_tests.push(PlaceholderTest {
                name: name.to_string(),
                file: file.to_string(),
                line,
                reason: PlaceholderReason::EmptyBody,
            });
            return;
        }

        // Check for no assertions (check for assert!, assert_eq!, assert_ne!, panic!, etc.)
        // Also check for custom assertion methods (assert_component_exists, assert_grid_snapshot, etc.)
        let has_assertions = body.contains("assert!(")
            || body.contains("assert_eq!(")
            || body.contains("assert_ne!(")
            || body.contains("assert_matches!(")
            || body.contains("panic!(")
            || body.contains("expect(")
            || body.contains("unwrap()")
            || body.contains("?") // Result-based error handling
            || body.contains("assert_") // Custom assertion methods (assert_component_exists, etc.)
            || body.contains("insta::") // Snapshot testing
            || body.contains("_snapshot!"); // Snapshot macros

        if !has_assertions {
            // Only flag if the test has some code but no assertions
            if body_code.len() > 10 {
                self.placeholder_tests.push(PlaceholderTest {
                    name: name.to_string(),
                    file: file.to_string(),
                    line,
                    reason: PlaceholderReason::NoAssertions,
                });
            }
        }
    }

    pub fn find_placeholder_tests(&self) -> &[PlaceholderTest] {
        &self.placeholder_tests
    }

    pub fn report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Test Quality Audit Report ===\n\n");

        if self.placeholder_tests.is_empty() {
            report.push_str("No placeholder tests found! All tests look good.\n");
            return report;
        }

        report.push_str(&format!(
            "Found {} placeholder test(s):\n\n",
            self.placeholder_tests.len()
        ));

        // Group by reason
        let mut by_reason: std::collections::HashMap<String, Vec<&PlaceholderTest>> =
            std::collections::HashMap::new();

        for test in &self.placeholder_tests {
            let reason_str = match test.reason {
                PlaceholderReason::AlwaysPasses => "Always Passes",
                PlaceholderReason::TodoOrUnimplemented => "TODO/Unimplemented",
                PlaceholderReason::EmptyBody => "Empty Body",
                PlaceholderReason::NoAssertions => "No Assertions",
            };
            by_reason
                .entry(reason_str.to_string())
                .or_insert_with(Vec::new)
                .push(test);
        }

        for (reason, tests) in by_reason.iter() {
            report.push_str(&format!("\n{}:\n", reason));
            for test in tests {
                report.push_str(&format!(
                    "  - {} ({}:{})\n",
                    test.name, test.file, test.line
                ));
            }
        }

        report.push_str("\n=== Recommendations ===\n");
        report.push_str("- Replace placeholder tests with meaningful assertions\n");
        report.push_str("- Verify that tests actually test the intended behavior\n");
        report.push_str("- Remove or implement tests marked with todo!()\n");

        report
    }

    pub fn generate_replacement_template(&self, test: &PlaceholderTest) -> String {
        let mut template = String::new();
        template.push_str(&format!("/// {}\n", test.name));
        template.push_str("///\n");
        template.push_str("/// TODO: Replace this placeholder test with meaningful assertions\n");
        template.push_str("///\n");
        template.push_str(&format!("/// Reason: {:?}\n", test.reason));
        template.push_str("#[test]\n");
        template.push_str(&format!("fn {}() -> Result<()> {{\n", test.name));
        template.push_str("    // TODO: Implement real test logic here\n");
        template.push_str("    // Examples:\n");
        template.push_str("    // - Create test data\n");
        template.push_str("    // - Execute the function/method being tested\n");
        template.push_str("    // - Assert expected outcomes\n");
        template.push_str("    \n");
        template.push_str("    // Replace this with real assertions:\n");
        template.push_str("    // assert_eq!(actual, expected);\n");
        template.push_str("    // assert!(condition);\n");
        template.push_str("    \n");
        template.push_str("    Ok(())\n");
        template.push_str("}\n");
        template
    }
}

fn remove_comments(code: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = code.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                result.push(ch);
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if ch == '"' && !in_string {
            in_string = true;
            result.push(ch);
            continue;
        }

        if ch == '"' && in_string {
            in_string = false;
            result.push(ch);
            continue;
        }

        if !in_string {
            if ch == '/' && chars.peek() == Some(&'/') {
                in_line_comment = true;
                chars.next();
                continue;
            }

            if ch == '/' && chars.peek() == Some(&'*') {
                in_block_comment = true;
                chars.next();
                continue;
            }
        }

        result.push(ch);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auditor_creation() {
        let auditor = TestAuditor::new();
        assert_eq!(auditor.find_placeholder_tests().len(), 0);
    }

    #[test]
    fn test_report_with_no_placeholders() {
        let auditor = TestAuditor::new();
        let report = auditor.report();
        assert!(report.contains("No placeholder tests found"));
    }

    #[test]
    fn test_remove_comments_line() {
        let code = "let x = 5; // comment\nlet y = 10;";
        let result = remove_comments(code);
        assert!(result.contains("let x = 5;"));
        assert!(result.contains("let y = 10;"));
        assert!(!result.contains("// comment"));
    }

    #[test]
    fn test_remove_comments_block() {
        let code = "let x = 5; /* block comment */ let y = 10;";
        let result = remove_comments(code);
        assert!(result.contains("let x = 5;"));
        assert!(result.contains("let y = 10;"));
        assert!(!result.contains("block comment"));
    }
}
