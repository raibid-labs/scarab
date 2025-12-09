# Scarab Testing Guide

This guide outlines the testing strategies and commands for Scarab. We use `just` to automate testing workflows.

## Prerequisites

- **Rust**: 1.75+
- **Just**: `cargo install just`
- **Ratatui Testlib**: Enabled for specific smoke tests via `SCARAB_TEST_RTL=1`.

## Quick Start

Run the full CI validation suite (Formatting, Clippy, Unit Tests):

```bash
just ci
```

## Common Test Commands

| Command | Description |
|---------|-------------|
| `just test` | Run workspace unit tests. |
| `just test-all` | Run **all** test suites (unit, golden, e2e, headless). |
| `just check` | Run cargo check on the workspace. |
| `just lint` | Run clippy lints. |

## Integration & E2E Testing

Scarab has several layers of integration testing:

### 1. Ratatui Testlib (Smoke Tests)

These tests verify the terminal UI using `ratatui-testlib`. They are gated by an environment variable.

```bash
SCARAB_TEST_RTL=1 just rtl-smoke
```

### 2. Headless Harness

Tests the terminal logic without a graphical window.

```bash
just headless
```

### 3. Golden Tests (Visual Regression)

Snapshots are used to verify rendering consistency.

```bash
just golden
```

To update snapshots:

```bash
just golden-update
```

### 4. Navigation Tests

Verify input handling and pane navigation.

```bash
just nav-tests
```

## Performance Benchmarking

To run benchmarks:

```bash
just bench
```

## Plugin Development

For testing plugins, refer to the [Plugin Development Guide](docs/plugin-development/README.md).