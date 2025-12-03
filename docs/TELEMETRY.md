# Scarab Telemetry and Logging

This document describes the telemetry and logging configuration options in Scarab, designed to provide observability for development, debugging, and performance monitoring.

## Overview

Scarab's telemetry system is **opt-in by default** to avoid any performance impact on production use. When enabled, it provides detailed insights into:

- Compositor performance (FPS tracking)
- Shared memory synchronization (sequence number changes)
- Update patterns (dirty region sizes)
- Pane lifecycle management (orchestrator events)

## Configuration

Telemetry can be configured via:
1. **Configuration files** (`.fsx` or `.toml`)
2. **Environment variables** (override config file settings)

### Configuration File (Fusabi)

Add to `~/.config/scarab/config.fsx`:

```fsharp
let telemetry = {
    FpsLogIntervalSecs = 5;       // Log FPS every 5 seconds
    LogSequenceChanges = true;     // Log sequence number changes
    LogDirtyRegions = false;       // Log dirty region sizes
    LogPaneEvents = true           // Log pane lifecycle events
}

{
    terminal = terminal;
    font = font;
    colors = colors;
    ui = ui;
    telemetry = telemetry
}
```

### Configuration File (TOML)

Add to `~/.config/scarab/config.toml`:

```toml
[telemetry]
fps_log_interval_secs = 5
log_sequence_changes = true
log_dirty_regions = false
log_pane_events = true
```

### Environment Variables

Environment variables **override** config file settings, useful for quick debugging:

```bash
# Log FPS every 5 seconds
SCARAB_LOG_FPS=5 cargo run -p scarab-daemon

# Enable sequence number logging
SCARAB_LOG_SEQUENCE=1 cargo run -p scarab-daemon

# Enable dirty region logging
SCARAB_LOG_DIRTY=1 cargo run -p scarab-daemon

# Enable pane lifecycle logging
SCARAB_LOG_PANES=1 cargo run -p scarab-daemon

# Combine with Rust log level for detailed output
RUST_LOG=scarab_daemon=debug SCARAB_LOG_FPS=5 SCARAB_LOG_PANES=1 \
    cargo run -p scarab-daemon
```

## Telemetry Options

### 1. FPS Logging (`fps_log_interval_secs`)

**Purpose**: Monitor compositor performance
**Type**: `u64` (seconds, 0 = disabled)
**Default**: `0` (disabled)

Logs average FPS over the specified interval. Useful for:
- Validating 60fps target
- Detecting performance regressions
- Understanding compositor behavior

**Example Output**:
```
[INFO] Compositor: 60.2 fps (avg over 5s), 3012 frames
[INFO] Compositor: 59.8 fps (avg over 5s), 2990 frames
```

**Usage**:
```bash
# Config file
FpsLogIntervalSecs = 5

# Environment variable
SCARAB_LOG_FPS=5
```

### 2. Sequence Number Logging (`log_sequence_changes`)

**Purpose**: Debug shared memory synchronization
**Type**: `bool`
**Default**: `false`

Logs sequence number changes when the compositor blits to shared memory. Useful for:
- Debugging IPC synchronization issues
- Understanding update frequency
- Validating lock-free shared memory protocol

**Example Output**:
```
[DEBUG] Sequence: 1234 -> 1235
[DEBUG] Sequence: 1235 -> 1236
```

**Usage**:
```bash
# Config file
LogSequenceChanges = true

# Environment variable
SCARAB_LOG_SEQUENCE=1
```

### 3. Dirty Region Logging (`log_dirty_regions`)

**Purpose**: Monitor update patterns and performance
**Type**: `bool`
**Default**: `false`

Logs the size of dirty regions when blitting to shared memory. Useful for:
- Understanding which parts of the grid change
- Optimizing partial updates
- Performance profiling

**Note**: This feature is not yet fully implemented (requires modification to `blit_to_shm` function).

**Planned Output**:
```
[DEBUG] Blit: 847 dirty cells (4.2% of grid)
[DEBUG] Blit: 1200 dirty cells (6.0% of grid)
```

**Usage**:
```bash
# Config file
LogDirtyRegions = true

# Environment variable
SCARAB_LOG_DIRTY=1
```

### 4. Pane Events Logging (`log_pane_events`)

**Purpose**: Track pane lifecycle in the orchestrator
**Type**: `bool`
**Default**: `false`

Logs pane creation, destruction, and reader task lifecycle events. Useful for:
- Validating tab/pane multiplexing
- Debugging pane orchestrator issues
- Understanding parallel PTY reading

**Example Output**:
```
[INFO] PaneOrchestrator: Started with 1 panes
[INFO] PaneOrchestrator: Reader task spawned for pane 1
[INFO] PaneOrchestrator: Reader task started for pane 1
[INFO] PaneOrchestrator: Pane 2 created, spawning reader
[INFO] PaneOrchestrator: Reader task spawned for pane 2
[INFO] PaneOrchestrator: Pane 1 destroyed, stopping reader
[INFO] PaneOrchestrator: Reader task stopped for pane 1
```

**Usage**:
```bash
# Config file
LogPaneEvents = true

# Environment variable
SCARAB_LOG_PANES=1
```

## Logging Levels

Scarab uses the standard Rust `log` crate with `env_logger`. Control verbosity with `RUST_LOG`:

```bash
# Only show errors
RUST_LOG=error cargo run -p scarab-daemon

# Show warnings and errors
RUST_LOG=warn cargo run -p scarab-daemon

# Show info, warnings, and errors (default recommended)
RUST_LOG=info cargo run -p scarab-daemon

# Show debug messages (includes sequence changes if enabled)
RUST_LOG=debug cargo run -p scarab-daemon

# Show everything (very verbose)
RUST_LOG=trace cargo run -p scarab-daemon

# Filter by module
RUST_LOG=scarab_daemon=debug cargo run -p scarab-daemon
RUST_LOG=scarab_daemon::orchestrator=debug cargo run -p scarab-daemon
```

## Telemetry Output Levels

| Option | Level | Rationale |
|--------|-------|-----------|
| FPS logging | `INFO` | Summary metric, not per-frame |
| Sequence changes | `DEBUG` | Detailed sync info |
| Dirty regions | `DEBUG` | Performance details |
| Pane events (enabled) | `INFO` | Important lifecycle events |
| Pane events (disabled) | `DEBUG` | Background details |

## Performance Impact

All telemetry is designed to have **minimal performance impact** when disabled:

- FPS tracking: Zero overhead when `fps_log_interval_secs = 0`
- Sequence logging: Single boolean check per frame
- Dirty regions: Single boolean check per blit
- Pane events: Boolean check only on lifecycle events (rare)

When enabled, overhead is negligible:
- FPS tracking: Simple counter increment per frame
- Logging: Asynchronous, does not block compositor

## Example Use Cases

### Debugging Compositor Performance

```bash
# Monitor FPS and sequence changes
RUST_LOG=info SCARAB_LOG_FPS=5 SCARAB_LOG_SEQUENCE=1 \
    cargo run -p scarab-daemon
```

### Debugging Tab/Pane Multiplexing

```bash
# Monitor pane lifecycle with debug output
RUST_LOG=debug SCARAB_LOG_PANES=1 \
    cargo run -p scarab-daemon
```

### Full Debugging Session

```bash
# Enable all telemetry
RUST_LOG=debug \
SCARAB_LOG_FPS=5 \
SCARAB_LOG_SEQUENCE=1 \
SCARAB_LOG_PANES=1 \
    cargo run -p scarab-daemon
```

### Production Monitoring

Create `~/.config/scarab/config.fsx` with:

```fsharp
let telemetry = {
    FpsLogIntervalSecs = 30;  // Log FPS every 30 seconds
    LogSequenceChanges = false;
    LogDirtyRegions = false;
    LogPaneEvents = false
}
```

Then run with:
```bash
RUST_LOG=info cargo run -p scarab-daemon
```

## See Also

- [Configuration Documentation](./CUSTOMIZATION.md)
- [Example Configs](../examples/fusabi-config/)
- [Development Workflow](./plugin-development/setup/dev-workflow.md)
