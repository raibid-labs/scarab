# Scarab Plugin Inspector

A visual debugging and monitoring tool for Fusabi plugin developers.

## Overview

The Plugin Inspector provides a comprehensive GUI for debugging, monitoring, and managing plugins in the Scarab terminal emulator. It's designed to give plugin developers real-time insight into plugin behavior, performance, and errors.

## Features

### 1. Plugin List View
- **Status Indicators**: Color-coded dots showing plugin health
  - Green: Enabled and working
  - Red: Enabled with failures
  - Gray: Disabled
- **Quick Info**: Version and failure count at a glance
- **Search/Filter**: Find plugins by name, description, or author
- **Real-time Updates**: Automatically reflects plugin state changes

### 2. Overview Tab
- **Status Dashboard**: Current enabled/disabled state
- **Failure Tracking**: Number of consecutive failures
- **Performance Metrics**:
  - Total hook executions
  - Total execution time
  - Average execution time per hook
- **Error Display**: Last error message with prominent visibility
- **Quick Actions**: Enable/Disable/Reload buttons with live feedback

### 3. Metadata Tab
Displays complete plugin information:
- Name, version, and description
- Author and homepage (with clickable link)
- API version compatibility
- Minimum Scarab version required

### 4. Hooks Tab
- **Execution History**: Last 100 hook invocations
- **Statistics Summary**:
  - Total executions
  - Success vs failed count
  - Average execution time
- **Detailed Records**:
  - Timestamp (relative time ago)
  - Hook type (on_output, on_input, etc.)
  - Execution duration
  - Error messages for failures
- **Visual Feedback**: Green checkmarks for success, red X for failures

### 5. Logs Tab
- **Real-time Log Streaming**: View plugin output as it happens
- **Log Levels**: Trace, Debug, Info, Warn, Error with color coding
- **Filtering**: Search logs by content or plugin name
- **Auto-scroll**: Option to stick to bottom for live viewing
- **Timestamp Display**: Relative time for each log entry
- **Plugin Attribution**: See which plugin generated each log

### 6. Source Tab
Placeholder for future features:
- Plugin source code viewing (.fsx files)
- Compiled bytecode information (.fzb files)
- Configuration inspection
- Dependency tree visualization

## Usage

### Opening the Inspector

**Keyboard Shortcut**: `Ctrl + Shift + P`

The inspector will:
1. Open as a resizable window overlay
2. Automatically request the current plugin list from the daemon
3. Display real-time updates as plugins execute

### Navigating the Interface

**Sidebar (Left)**:
- Browse all loaded plugins
- Use the filter box to search
- Click a plugin to view details

**Toolbar (Top)**:
- View statistics: Total, Enabled, Failed plugins
- **Refresh**: Request updated plugin list
- **Clear Logs**: Remove all log entries
- **Export Debug Info**: Save all data to a file
- **Close**: Close the inspector

**Details Panel (Right)**:
- Five tabs with different views
- Scrollable content for large data sets
- Interactive controls for plugin management

### Managing Plugins

From the Overview tab, you can:

1. **Enable/Disable**: Toggle plugin activation
   - Click "Enable" or "Disable" button
   - State change is sent to daemon immediately
   - UI updates when confirmation received

2. **Reload**: Restart a plugin without restarting Scarab
   - Useful for development/debugging
   - Preserves execution metrics
   - Clears failure count on success

### Monitoring Performance

Track plugin performance impact:

**Per-Plugin Metrics**:
- Total execution time across all hooks
- Average time per hook execution
- Identify slow plugins causing latency

**Hook History**:
- See which hooks are called most frequently
- Identify performance bottlenecks
- Debug intermittent failures

### Debugging Errors

When a plugin fails:

1. **Visual Indicators**:
   - Red dot in plugin list
   - "Failures: N" count in sidebar
   - Red error card in Overview tab

2. **Error Details**:
   - Last error message displayed prominently
   - Timestamp of last failure
   - Full error in Hooks tab history

3. **Export Debug Info**:
   - Click "Export Debug Info" button
   - Creates `scarab-plugin-debug-{timestamp}.txt`
   - Contains full plugin state and log history

### Log Filtering

Find specific events quickly:

1. Enter search term in "Filter" box
2. Matches plugin name or log message
3. Case-insensitive search
4. Clear filter to see all logs

## Building with Plugin Inspector

The inspector is controlled by a feature flag.

### Enable Plugin Inspector

```bash
# Build client with inspector enabled
cargo build -p scarab-client --features plugin-inspector

# Run with inspector
cargo run -p scarab-client --features plugin-inspector
```

### Default Build (No Inspector)

```bash
# Build without inspector (smaller binary)
cargo build -p scarab-client
```

## Architecture

### Client-Side Components

**Module**: `crates/scarab-client/src/plugin_inspector.rs`

**Resources**:
- `PluginInspectorState`: Tracks UI state, plugins, logs, history

**Systems**:
- `toggle_inspector_input`: Handles Ctrl+Shift+P keybinding
- `render_inspector_ui`: Draws the egui interface
- `handle_plugin_messages`: Processes daemon responses

**Plugin**: `PluginInspectorPlugin`
- Automatically adds bevy_egui if needed
- Registers systems in Update schedule
- Prints keybinding hint on startup

### Protocol Extensions

**New Control Messages** (`ControlMessage` enum):
- `PluginListRequest`: Request current plugin list
- `PluginEnable { name }`: Enable a plugin
- `PluginDisable { name }`: Disable a plugin
- `PluginReload { name }`: Reload a plugin

**New Daemon Messages** (`DaemonMessage` enum):
- `PluginList { plugins }`: Send plugin list to client
- `PluginStatusChanged { name, enabled }`: Notify status change
- `PluginError { name, error }`: Notify error occurred

**New Types**:
- `PluginInspectorInfo`: Serializable plugin metadata

### Data Flow

```
Client                          Daemon
  |                               |
  |  Ctrl+Shift+P pressed         |
  |------------------------------>|
  |  PluginListRequest            |
  |                               |
  |<------------------------------|
  |  PluginList { plugins }       |
  |                               |
  |  User clicks "Disable"        |
  |------------------------------>|
  |  PluginDisable { name }       |
  |                               |
  |<------------------------------|
  |  PluginStatusChanged          |
  |                               |
```

## UI Design Principles

### Color Scheme
- **Dark Theme**: Reduces eye strain during long sessions
- **Semantic Colors**:
  - Green: Success, enabled
  - Red: Error, failure, disabled
  - Yellow/Amber: Warnings
  - Blue: Information, plugin names
  - Gray: Disabled, empty states

### Layout
- **Three-Panel Design**: Sidebar + Toolbar + Content
- **Resizable**: Adjust to your workflow
- **Scrollable**: Handle hundreds of plugins/logs
- **Responsive**: Updates without UI flicker

### Typography
- **Monospace**: For technical data (times, IDs)
- **Sans-serif**: For labels and descriptions
- **Size Hierarchy**: Headers > body > metadata

### Interaction Patterns
- **Hover Effects**: Visual feedback on all buttons
- **Selection State**: Clear indication of selected plugin
- **Loading States**: Implicit (no spinners needed with realtime)
- **Error States**: Prominent but not alarming

## Performance Considerations

### Memory Usage
- **Log Buffer**: Max 1,000 entries (configurable)
- **Hook History**: Max 500 executions (configurable)
- **Circular Buffers**: Oldest entries dropped automatically

### Rendering
- **Conditional Rendering**: Only draws when visible
- **Virtual Scrolling**: egui handles large lists efficiently
- **Lazy Updates**: Only re-render on state changes

### Network
- **On-Demand**: Only requests data when inspector opens
- **Incremental Updates**: Daemon sends change notifications
- **Efficient Protocol**: rkyv zero-copy serialization

## Development Workflow

### For Plugin Developers

1. **Develop your plugin** in `.fsx` or compile to `.fzb`
2. **Load into Scarab** via config or discovery
3. **Open inspector** with Ctrl+Shift+P
4. **Monitor execution**:
   - Watch Logs tab for debug output
   - Check Hooks tab for timing
   - View Overview for errors
5. **Iterate**: Make changes and reload

### For Scarab Contributors

Adding new inspector features:

1. **Update Protocol** (`scarab-protocol/src/lib.rs`):
   - Add new message types if needed
   - Extend `PluginInspectorInfo` for new data

2. **Update Client** (`scarab-client/src/plugin_inspector.rs`):
   - Add UI components in render functions
   - Handle new messages in `handle_plugin_messages`

3. **Update Daemon** (handler implementation):
   - Process new control messages
   - Send appropriate responses

## Troubleshooting

### Inspector Won't Open
- **Check**: Is the client built with `--features plugin-inspector`?
- **Verify**: Press Ctrl+Shift+P (not Cmd on Mac in Bevy)
- **Look**: Check console for "Plugin Inspector initialized" message

### No Plugins Showing
- **Wait**: Initial load takes ~100ms after opening
- **Check**: Is daemon running and connected?
- **Verify**: Look for "Received plugin list" in Logs tab
- **Debug**: Click Refresh button to manually request

### Plugin Not Responding to Enable/Disable
- **Check**: Look for error in Logs tab
- **Verify**: Daemon might reject if plugin is in failed state
- **Try**: Reload plugin first, then enable

### Performance Issues
- **Clear Logs**: Too many log entries can slow rendering
- **Check Filter**: Very broad filters search more data
- **Reduce History**: Modify MAX_LOG_ENTRIES constant

## Future Enhancements

Planned features for future versions:

### Source Code Viewer
- Syntax-highlighted .fsx display
- Bytecode disassembly for .fzb
- Inline error annotations
- Jump to error line

### Advanced Metrics
- Hook execution timeline (flamegraph)
- Memory usage tracking
- Network activity (if plugins use IPC)
- CPU profiling integration

### Interactive Debugging
- Breakpoints in Fusabi code
- Step-through execution
- Variable inspection
- REPL for plugin context

### Configuration Editor
- Visual plugin config editor
- Schema validation
- Live reload on save
- Config templates

### Plugin Marketplace
- Browse available plugins
- Install with one click
- Automatic updates
- Ratings and reviews

## Contributing

See main CONTRIBUTING.md for general guidelines.

For Plugin Inspector specifically:
- Follow egui UI patterns
- Keep rendering performant
- Add tests for new message handlers
- Update this documentation

## License

Same as Scarab project (see LICENSE file).
