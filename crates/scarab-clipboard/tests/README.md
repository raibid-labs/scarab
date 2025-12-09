# scarab-clipboard Tests

This directory contains integration tests for the scarab-clipboard crate.

## Test Organization

### `clipboard_tests.rs`
Tests for clipboard operations including:
- Clipboard manager initialization and availability checking
- Copy/paste operations for standard clipboard
- Linux-specific primary selection support
- Multiline and large text handling
- Unicode text support
- Clipboard clearing functionality
- Paste confirmation mode management
- Error handling for unavailable clipboards

**Note**: Many tests are marked with `#[ignore]` because they require a display server (X11/Wayland) to be running. To run these tests:

```bash
cargo test -p scarab-clipboard --test clipboard_tests -- --ignored
```

### `selection_tests.rs`
Tests for text selection functionality including:
- Selection modes (Character, Word, Line, Block)
- Selection region calculations and normalization
- Region containment checks
- Width and height calculations
- Selection state management and lifecycle
- Edge cases and boundary conditions

All tests in this file run without external dependencies.

### `plugin_tests.rs`
Tests for the ClipboardPlugin integration including:
- Plugin initialization and metadata
- Plugin command registration
- Command uniqueness and descriptions
- Platform-specific command availability (e.g., primary selection on Linux)
- Multiple plugin instance independence

## Running Tests

### Run all tests (excluding ignored tests):
```bash
cargo test -p scarab-clipboard
```

### Run only integration tests:
```bash
cargo test -p scarab-clipboard --tests
```

### Run tests that require a display server:
```bash
cargo test -p scarab-clipboard --test clipboard_tests -- --ignored
```

### Run a specific test file:
```bash
cargo test -p scarab-clipboard --test selection_tests
cargo test -p scarab-clipboard --test clipboard_tests
cargo test -p scarab-clipboard --test plugin_tests
```

### Run a specific test:
```bash
cargo test -p scarab-clipboard --test selection_tests test_selection_region_normalize
```

## Test Coverage

The tests cover:

1. **Unit tests** (in `src/` modules):
   - Basic word boundary detection
   - Paste confirmation logic
   - Selection region operations
   - Clipboard manager basics

2. **Integration tests** (in this directory):
   - Full clipboard copy/paste workflows
   - Selection state management
   - Plugin API integration
   - Platform-specific features
   - Edge cases and error handling

## Platform-Specific Tests

### Linux
Tests related to X11 primary selection are conditionally compiled and only run on Linux:
- Primary selection copy/paste
- Primary and standard clipboard independence
- Primary selection clearing

These tests are in `linux_specific` modules and use `#[cfg(target_os = "linux")]`.

## Test Categories

### Tests requiring display server (#[ignore])
These tests interact with the system clipboard and require:
- X11 or Wayland display server running
- Clipboard access permissions
- They are ignored by default to allow CI/headless testing

### Tests that always run
These tests use mocking or test the logic without system interaction:
- Selection calculations
- State management
- Input validation
- Error handling
- Plugin metadata

## Adding New Tests

When adding new tests:

1. Place integration tests in this `tests/` directory
2. Use `#[ignore]` for tests requiring display servers
3. Test both success and error paths
4. Include edge cases and boundary conditions
5. Document any platform-specific behavior
6. Use descriptive test names that explain what is being tested

## CI Considerations

The test suite is designed to run in CI environments without a display server:
- Core functionality tests run without `#[ignore]`
- Display-dependent tests are marked `#[ignore]`
- Error paths are tested for unavailable clipboards
- All state management logic is fully testable without system resources
