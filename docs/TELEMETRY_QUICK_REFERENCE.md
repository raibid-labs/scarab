# Telemetry Quick Reference

**Quick guide for enabling Scarab telemetry/logging**

## Environment Variables (Quick Debug)

```bash
# FPS logging every 5 seconds
SCARAB_LOG_FPS=5 cargo run -p scarab-daemon

# Pane lifecycle events
SCARAB_LOG_PANES=1 cargo run -p scarab-daemon

# Sequence number changes (requires debug logging)
RUST_LOG=debug SCARAB_LOG_SEQUENCE=1 cargo run -p scarab-daemon

# All telemetry
RUST_LOG=debug SCARAB_LOG_FPS=5 SCARAB_LOG_SEQUENCE=1 SCARAB_LOG_PANES=1 \
    cargo run -p scarab-daemon
```

## Configuration File

**Fusabi** (`~/.config/scarab/config.fsx`):
```fsharp
let telemetry = {
    FpsLogIntervalSecs = 5;      // 0 = disabled
    LogSequenceChanges = true;    // false = disabled
    LogDirtyRegions = false;      // not yet implemented
    LogPaneEvents = true          // false = disabled
}

{
    terminal = terminal;
    font = font;
    telemetry = telemetry
}
```

**TOML** (`~/.config/scarab/config.toml`):
```toml
[telemetry]
fps_log_interval_secs = 5
log_sequence_changes = true
log_dirty_regions = false
log_pane_events = true
```

## Logging Levels

```bash
# Errors only
RUST_LOG=error cargo run -p scarab-daemon

# Info + warnings + errors (recommended)
RUST_LOG=info cargo run -p scarab-daemon

# Debug (shows sequence changes if enabled)
RUST_LOG=debug cargo run -p scarab-daemon

# Filter by module
RUST_LOG=scarab_daemon=debug cargo run -p scarab-daemon
RUST_LOG=scarab_daemon::orchestrator=info cargo run -p scarab-daemon
```

## Expected Output

### FPS Logging
```
[INFO] Compositor: 60.2 fps (avg over 5s), 3012 frames
```

### Sequence Changes
```
[DEBUG] Sequence: 1234 -> 1235
```

### Pane Events
```
[INFO] PaneOrchestrator: Started with 1 panes
[INFO] PaneOrchestrator: Pane 1 created, spawning reader
```

## Common Use Cases

### Debugging Compositor Performance
```bash
RUST_LOG=info SCARAB_LOG_FPS=5 cargo run -p scarab-daemon
```

### Debugging Tab/Pane Flow
```bash
RUST_LOG=info SCARAB_LOG_PANES=1 cargo run -p scarab-daemon
```

### Full Debugging Session
```bash
RUST_LOG=debug SCARAB_LOG_FPS=5 SCARAB_LOG_SEQUENCE=1 SCARAB_LOG_PANES=1 \
    cargo run -p scarab-daemon
```

## Copy Example Config

```bash
# Fusabi (recommended)
cp examples/fusabi-config/telemetry.fsx ~/.config/scarab/config.fsx

# TOML (legacy)
cp examples/config-telemetry.toml ~/.config/scarab/config.toml
```

## Full Documentation

See [`TELEMETRY.md`](./TELEMETRY.md) for comprehensive documentation.

## Performance Impact

- **Disabled**: Zero overhead (default)
- **Enabled**: Negligible (<0.1% CPU)
