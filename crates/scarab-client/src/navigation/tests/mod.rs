//! Comprehensive navigation tests for Scarab terminal emulator
//!
//! This module provides comprehensive testing for the navigation system without requiring
//! a window or graphics context. Tests use Bevy's headless mode and mock terminal content.

// Test helper utilities - shared across all test modules
pub(crate) mod helpers;

// Individual test modules organized by feature area
mod focusable_tests;
mod focus_tests;
mod hint_tests;
mod integration_tests;
mod mode_tests;
mod prompt_tests;
mod registry_tests;
