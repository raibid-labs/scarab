# Plugin Inspector Quick Reference

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+P` | Toggle Plugin Inspector |
| `Escape` | Close Inspector |
| `Tab` | Navigate between UI elements |
| `Enter` | Activate focused button |
| `Arrow Keys` | Navigate plugin list |

## Status Indicators

| Symbol | Meaning |
|--------|---------|
| 🟢 (Green) | Plugin enabled, no failures |
| 🔴 (Red) | Plugin enabled with failures |
| ⚫ (Gray) | Plugin disabled |

## Log Levels

| Icon | Level | Color | Use Case |
|------|-------|-------|----------|
| ◦ | Trace | Dark Gray | Detailed debugging info |
| ▸ | Debug | Light Blue | Development messages |
| ℹ | Info | White | General information |
| ⚠ | Warn | Yellow | Warnings, non-critical issues |
| ✖ | Error | Red | Errors, failures |

## Toolbar Actions

| Button | Function |
|--------|----------|
| **Refresh** | Request updated plugin list from daemon |
| **Clear Logs** | Remove all log entries |
| **Export Debug Info** | Save plugin state to `scarab-plugin-debug-{timestamp}.txt` |
| **Close** | Close the inspector window |

## Tab Descriptions

| Tab | Purpose |
|-----|---------|
| **Overview** | Quick status, metrics, and actions |
| **Metadata** | Plugin details (author, version, homepage) |
| **Hooks** | Execution history with timing |
| **Logs** | Real-time log output |
| **Source** | Plugin code (future feature) |

## Quick Actions (Overview Tab)

| Button | Effect |
|--------|--------|
| **Enable/Disable** | Toggle plugin activation |
| **Reload** | Restart plugin without restarting Scarab |

## Filtering

### Plugin Filter (Sidebar)
- Type in the filter box to search by:
  - Plugin name
  - Description
  - Author name
- Case-insensitive
- Live updating

### Log Filter (Logs Tab)
- Type in the filter box to search by:
  - Log message content
  - Plugin name
- Case-insensitive
- Live updating

## Performance Metrics

| Metric | Description |
|--------|-------------|
| **Total Executions** | Number of times plugin hooks have run |
| **Total Execution Time** | Cumulative time spent in plugin code |
| **Avg Execution Time** | Mean duration per hook execution |
| **Failure Count** | Number of consecutive failures |

## Build Commands

```bash
# Build with inspector
cargo build -p scarab-client --features plugin-inspector

# Build without inspector (smaller binary)
cargo build -p scarab-client

# Run with inspector
cargo run -p scarab-client --features plugin-inspector
```

## Common Workflows

### Debugging a Failing Plugin

1. Press `Ctrl+Shift+P` to open inspector
2. Look for 🔴 red dots in plugin list
3. Click the failing plugin
4. Check **Overview** tab for last error
5. Switch to **Hooks** tab for execution history
6. Switch to **Logs** tab for detailed output
7. Click **Export Debug Info** to save for sharing

### Monitoring Performance

1. Open inspector
2. Select plugin from list
3. Check **Overview** tab metrics:
   - If avg execution time > 10ms, consider optimizing
   - High failure count indicates unstable plugin
4. Switch to **Hooks** tab to see outliers
5. Look for patterns in timing data

### Testing Plugin Changes

1. Make changes to plugin code
2. Open inspector
3. Click the plugin
4. Click **Reload** button
5. Watch **Logs** tab for initialization messages
6. Check **Overview** for new metrics
7. Verify failure count resets to 0

### Finding Slow Hooks

1. Open inspector
2. Go through plugin list
3. Compare avg execution times
4. Switch to **Hooks** tab for detailed view
5. Sort mentally by duration
6. Identify outliers (> 1ms)

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Inspector won't open | Check if built with `--features plugin-inspector` |
| No plugins showing | Wait for daemon response or click Refresh |
| Enable/Disable not working | Check Logs tab for error messages |
| UI frozen | Try closing and reopening inspector |
| Export fails | Check file permissions in current directory |

## Color Reference

| Element | Color | Hex |
|---------|-------|-----|
| Success/Enabled | Green | #64ff64 |
| Error/Failed | Red | #ff5050 |
| Warning | Yellow | #ffc800 |
| Info | Blue | #9696ff |
| Disabled | Gray | #808080 |
| Background | Dark Gray | #1a1a1a |
| Panel | Medium Gray | #2a2a2a |
| Card | Blue-Gray | #1e1e28 |

## File Locations

| File | Purpose |
|------|---------|
| `~/.config/scarab/config.toml` | Scarab configuration |
| `scarab-plugin-debug-{timestamp}.txt` | Exported debug info |
| `~/.local/share/scarab/plugins/` | Plugin directory (typical) |

## Protocol Messages

### Client → Daemon

| Message | Purpose |
|---------|---------|
| `PluginListRequest` | Request current plugin list |
| `PluginEnable { name }` | Enable a plugin |
| `PluginDisable { name }` | Disable a plugin |
| `PluginReload { name }` | Reload a plugin |

### Daemon → Client

| Message | Purpose |
|---------|---------|
| `PluginList { plugins }` | Send plugin list |
| `PluginStatusChanged { name, enabled }` | Notify status change |
| `PluginError { name, error }` | Notify error occurred |

## Limits & Constraints

| Resource | Limit | Configurable |
|----------|-------|--------------|
| Max log entries | 1,000 | Yes (MAX_LOG_ENTRIES constant) |
| Max hook history | 500 | Yes (MAX_HOOK_HISTORY constant) |
| Hook display limit | 100 most recent | No |
| Min window size | 800x600px | No |
| Default window size | 1000x700px | No |
| Sidebar width | 300px (200-500px range) | Via resize |

## Tips & Tricks

1. **Keep inspector open during development** for immediate feedback
2. **Use log filter** to focus on specific plugin output
3. **Export before closing** if you encounter a bug
4. **Monitor avg execution time** to catch performance regressions
5. **Check hooks tab** to understand plugin behavior patterns
6. **Clear logs periodically** if UI becomes sluggish
7. **Disable plugins** you're not using to reduce noise
8. **Reload instead of restarting** for faster iteration

## Related Documentation

- Full documentation: `crates/scarab-client/PLUGIN_INSPECTOR.md`
- UI design spec: `docs/plugin-inspector-ui-design.md`
- Implementation summary: `PLUGIN_INSPECTOR_SUMMARY.md`
- Plugin API docs: `crates/scarab-plugin-api/README.md`

## Support

For issues or questions:
1. Check troubleshooting section above
2. Export debug info for bug reports
3. Include plugin list and error messages
4. Mention Scarab version and platform

---

**Last Updated**: 2025-11-24
**Version**: 0.1.0
**Component**: scarab-client plugin-inspector feature
