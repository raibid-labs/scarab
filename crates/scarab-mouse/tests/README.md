# Scarab Mouse - Test Suite

This directory contains comprehensive integration tests for the scarab-mouse crate.

## Test Files

### 1. `click_handler_tests.rs` (19 tests)
Tests for click detection and mouse event sequence generation:
- Single, double, and triple-click detection
- Click timeout and distance thresholds
- Mouse sequence generation (SGR format)
- Cursor positioning sequences
- Modifier key handling (Shift, Ctrl, Alt)
- Button type support (Left, Right, Middle, ScrollUp, ScrollDown)

### 2. `selection_tests.rs` (26 tests)
Tests for text selection functionality:
- Character, word, line, and block selection types
- Selection normalization (forward/backward)
- Selection containment checks
- Word boundary detection and expansion
- Text extraction from selections
- Multi-line and rectangular block selections
- Empty and sparse grid handling

### 3. `mode_tests.rs` (42 tests)
Tests for mouse mode detection and management:
- ANSI escape sequence detection (X10, SGR, urxvt modes)
- Mode switching (Normal vs Application)
- Application heuristics (vim, tmux, htop, etc.)
- Mode persistence across sequences
- Case-insensitive app detection
- Description lookup for known applications

### 4. `context_menu_tests.rs` (36+ tests)
Tests for context menu functionality:
- Menu creation and item management
- Standard, URL, and file-specific menus
- Menu navigation (next/previous with separator skipping)
- Item enable/disable states
- Keyboard shortcut handling
- Selection-dependent menu states

### 5. `types_tests.rs` (39 tests)
Tests for core mouse types:
- Position creation and distance calculation
- Modifier combinations and equality
- Mouse button and event variants
- Click type enumeration
- Mouse mode state
- Serialization/deserialization roundtrips
- Hash and Copy trait implementations

### 6. `integration_tests.rs` (19 tests)
Comprehensive end-to-end workflow tests:
- Complete click-to-select workflows
- Drag selection scenarios
- Context menu integration with selections
- Mouse mode changes with vim
- Ctrl+Click on URLs
- Triple-click line selection
- Scroll wheel in application mode
- Multi-line and block selections
- Word expansion on double-click
- Complex modifier combinations
- Backward selection normalization

## Running Tests

Run all tests:
```bash
cargo test -p scarab-mouse
```

Run specific test file:
```bash
cargo test -p scarab-mouse --test click_handler_tests
cargo test -p scarab-mouse --test selection_tests
cargo test -p scarab-mouse --test mode_tests
cargo test -p scarab-mouse --test context_menu_tests
cargo test -p scarab-mouse --test types_tests
cargo test -p scarab-mouse --test integration_tests
```

Run unit tests only (in src/ modules):
```bash
cargo test -p scarab-mouse --lib
```

Run all integration tests:
```bash
cargo test -p scarab-mouse --tests
```

## Test Coverage

The test suite provides comprehensive coverage of:
- **Click handling**: Detection, timing, distance thresholds
- **Selection**: All selection types, text extraction, boundaries
- **Mode detection**: ANSI sequences, app heuristics
- **Context menus**: Creation, navigation, state management
- **Type safety**: Serialization, equality, hashing
- **Integration**: Real-world workflows and edge cases

Total: **181+ tests** covering all major functionality without external dependencies.

## Test Philosophy

These tests are designed to:
1. Run without external dependencies (no X11, Wayland, clipboard)
2. Use mock data structures for terminal grids
3. Test behavior, not implementation details
4. Cover edge cases and error conditions
5. Provide clear documentation through test names
6. Enable confident refactoring and feature additions

## Mock Structures

The integration tests use `MockTerminalGrid` to simulate terminal content without requiring a real terminal emulator. This allows fast, deterministic testing of selection and text extraction logic.

## Future Enhancements

Potential additions to the test suite:
- Performance benchmarks for selection on large grids
- Fuzz testing for escape sequence parsing
- Property-based testing for selection operations
- Visual regression tests for rendering (requires Bevy context)
