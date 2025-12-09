# Scarab Panes Test Suite

This directory contains comprehensive tests for the scarab-panes crate, which provides split pane management functionality for the Scarab terminal emulator.

## Test Coverage Summary

**Total Tests: 93 passing**

- **Unit Tests (in lib.rs)**: 14 tests
- **Async Integration Tests**: 20 tests
- **Complex Layout Tests**: 16 tests
- **Edge Cases Tests**: 24 tests
- **Pane Creation Tests**: 9 tests
- **Pane Layout Tests**: 10 tests

## Test Files

### `async_integration_tests.rs`
Tests async plugin behavior including command handling and event processing:
- Remote command execution (split, close, navigate, resize, zoom)
- Terminal resize handling
- Input processing (keyboard shortcuts)
- Sequential command workflows

### `complex_layout_tests.rs`
Tests complex multi-pane layouts and stress scenarios:
- Quad split layouts (2x2 grids)
- Deep horizontal and vertical nesting
- Mixed split directions
- Create and close sequences
- Terminal resize with complex layouts
- Navigation in complex grids
- Stress testing with many panes
- Real-world workflow simulations

### `edge_cases_tests.rs`
Tests edge cases and boundary conditions:
- Zero/minimal dimensions
- Maximum dimensions
- Odd-sized terminals
- Various common terminal sizes
- Metadata validation
- Command structure validation
- Serialization roundtrips
- Extreme position values

### `pane_creation_tests.rs`
Tests basic plugin creation and metadata:
- Plugin initialization with various sizes
- Metadata correctness
- Command registration
- Split direction serialization
- Default constructors

### `pane_layout_tests.rs`
Tests pane layout structures and calculations:
- Layout creation and cloning
- Serialization/deserialization
- Parent-child relationships
- Various split ratios
- Boundary positions
- Minimal and maximum dimensions

## Running Tests

Run all tests:
```bash
cargo test -p scarab-panes
```

Run specific test file:
```bash
cargo test -p scarab-panes --test async_integration_tests
cargo test -p scarab-panes --test complex_layout_tests
cargo test -p scarab-panes --test edge_cases_tests
cargo test -p scarab-panes --test pane_creation_tests
cargo test -p scarab-panes --test pane_layout_tests
```

Run tests with output:
```bash
cargo test -p scarab-panes -- --nocapture
```

Run specific test:
```bash
cargo test -p scarab-panes test_quad_split_layout
```

## Test Categories

### Pane Management
- Creating panes
- Splitting panes (horizontal/vertical)
- Closing panes
- Focusing panes
- Resizing panes
- Navigating between panes

### Layout Calculations
- Tree-based layout recalculation
- Space redistribution after closes
- Split ratio management
- Terminal resize handling

### Plugin Integration
- Command registration and execution
- Metadata management
- Async event handling
- Input processing

### Edge Cases
- Zero-size terminals
- Maximum-size terminals
- Single pane restrictions
- Deep nesting scenarios
- Stress testing

## Dependencies

The test suite requires:
- `tokio` (with "rt" and "macros" features) for async testing
- `serde_json` for serialization testing
- `scarab-plugin-api` for plugin trait and context

## Test Design Philosophy

The tests follow these principles:

1. **No External Dependencies**: Tests run without PTY, IPC, or UI dependencies
2. **Comprehensive Coverage**: Tests cover happy paths, error conditions, and edge cases
3. **Fast Execution**: All tests complete in < 1 second
4. **Clear Naming**: Test names describe what they verify
5. **Isolated**: Each test is independent and can run in any order
6. **Realistic**: Tests simulate real-world usage patterns

## Continuous Testing

These tests are designed to catch regressions in:
- Pane layout algorithms
- Tree structure maintenance
- Split ratio calculations
- Command handling
- Terminal resize logic
- Edge case handling

## Contributing

When adding new features to scarab-panes, please:
1. Add corresponding tests to the appropriate test file
2. Ensure all existing tests pass
3. Aim for >90% code coverage
4. Test both success and failure paths
5. Include edge cases and boundary conditions
