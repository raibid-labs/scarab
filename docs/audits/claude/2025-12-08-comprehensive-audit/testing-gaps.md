# Testing Infrastructure Gaps - Scarab Audit December 2025

## Overview

The Scarab project has partially developed testing infrastructure with significant coverage gaps.

## Crates Without Dedicated Test Directories

| Crate | Lines of Code | Risk Level |
|-------|---------------|------------|
| scarab-clipboard | ~1,400 | HIGH |
| scarab-mouse | ~2,000+ | HIGH |
| scarab-panes | 1,067 | HIGH |
| scarab-plugin-compiler | Unknown | CRITICAL |
| scarab-nav | 294 | MEDIUM |
| scarab-palette | ~800 | MEDIUM |
| scarab-tabs | 449 | MEDIUM |
| scarab-telemetry-hud | ~700 | MEDIUM |

## Untested Critical Files

### Daemon
| File | Lines | Impact |
|------|-------|--------|
| ipc.rs | 1,014 | Critical IPC handling |
| profiling.rs | ~200 | Metrics collection |
| main.rs | ~500 | Initialization |

### Client
| File | Lines | Impact |
|------|-------|--------|
| graphics_inspector.rs | ~400 | Debug tooling |
| integration.rs | ~400 | Core integration |
| ipc.rs | ~420 | Client IPC |
| plugin_inspector.rs | 918 | Plugin debugging |
| prompt_markers.rs | ~300 | Shell integration |
| safe_state.rs | ~200 | Memory safety |
| telemetry_integration.rs | ~150 | Observability |
| zones.rs | ~400 | Zone management |

## Placeholder Tests

The following smoke tests are just `assert!(true)`:

### scarab-daemon/tests/smoke_tests.rs
- `test_daemon_terminal_processing`
- `test_daemon_shared_memory`
- `test_daemon_plugin_management`
- `test_daemon_session_management`
- `test_daemon_tab_management`
- `test_daemon_pane_management`
- `test_daemon_profiling`
- `test_daemon_performance`
- `test_daemon_cleanup`

### scarab-client/tests/smoke_tests.rs
- `test_client_shared_memory`
- `test_client_terminal_display`
- `test_client_input_handling`
- `test_client_scrollback`
- `test_client_plugin_rendering`
- `test_client_performance`
- `test_client_cleanup`
- `test_client_theming`
- `test_client_resize`

## Missing Test Categories

### 1. End-to-End Tests
- Client + Daemon + IPC integration
- `/tests/e2e/` contains only templates/placeholders

### 2. Regression Tests
- No git-based test history
- No known-bug reproduction tests

### 3. Security Tests
- No input validation tests
- No privilege escalation tests
- No buffer overflow tests
- No untrusted input handling tests

### 4. Performance Regression
- Benchmarks exist but no baseline comparisons
- No memory regression tests
- No latency regression tests

### 5. Accessibility Tests
- No screen reader integration tests
- Keyboard navigation tests incomplete

### 6. Platform-Specific Tests
- Linux tests: Present but minimal
- macOS tests: Minimal
- Windows tests: Minimal

## CI/CD Issues

### Coverage Not Enforced
```yaml
# ci.yml
fail_ci_if_error: false  # Coverage failures don't break build
```

### RTL Tests Continue on Error
```yaml
# test-rtl.yml
continue-on-error: true  # Test failures ignored
```

### Coverage Excludes Integration Tests
```bash
cargo tarpaulin --workspace --exclude-files 'tests/*'
# This excludes 13,080 lines of test code from measurement
```

## Untested Critical Paths

| Critical Path | Status | Impact |
|---------------|--------|--------|
| Daemon startup | Partial | Server may fail to initialize |
| Session creation | Partial | Users can't create sessions reliably |
| PTY I/O pipeline | Partial | Terminal output may be corrupted |
| Plugin loading | Partial | Plugins may not load correctly |
| IPC message routing | Untested | Messages may be lost |
| Client-daemon sync | Partial | UI may fall out of sync |
| Clipboard operations | Inline only | Clipboard may be unreliable |
| Mouse interactions | Inline only | Mouse events may not work |
| Config hot-reload | Untested | Config changes may not apply |
| Theme switching | Partial | Themes may not apply |
| Multi-pane operations | Untested | Pane operations may fail |
| SSH session management | Ignored | SSH sessions untested |

## Recommendations

### Phase 1 (Week 1)
1. Create `/tests/` for scarab-clipboard, scarab-mouse
2. Implement real smoke tests
3. Add ipc.rs inline tests
4. Fix coverage calculation

### Phase 2 (Week 2-3)
1. Create E2E test suite
2. Implement PTY/terminal test harness
3. Add plugin loader tests
4. Create config hot-reload tests

### Phase 3 (Week 4-6)
1. Add security/input validation tests
2. Implement platform-specific suites
3. Create performance regression tests
4. Add accessibility tests

## Artifacts Needed

- `tests/fixtures/` - Test data directory
- `.cargo/config.toml` - Test profiles
- `docs/testing.md` - Test infrastructure docs
- `benchmarks/baselines/` - Performance baseline data
- `scripts/test-setup.sh` - Environment setup
