# Plugin Lifecycle

Understanding the plugin lifecycle is essential for building reliable Scarab plugins.

## Overview

```
System Start
  ↓
Load Plugins → OnLoad
  ↓
Runtime Hooks (OnOutput, OnInput, etc.)
  ↓
System Shutdown → OnUnload
```

## Lifecycle Stages

### 1. Discovery

Scarab discovers plugins from:
- `~/.config/scarab/plugins/`
- Project-local `.scarab/plugins/`
- System-wide `/usr/share/scarab/plugins/`

### 2. Loading

For each enabled plugin:
1. Read `plugin.toml` manifest
2. Validate API version compatibility
3. Load runtime (VM for .fzb, interpreter for .fsx)
4. Call `OnLoad` hook

### 3. Runtime

Hooks are called based on events:
- **OnOutput** - Every line of terminal output
- **OnInput** - Every input from user
- **OnPreCommand** - Before command execution
- **OnPostCommand** - After command completion
- **OnResize** - Terminal resize events
- **OnAttach/OnDetach** - Client connections

### 4. Unloading

On shutdown or plugin disable:
1. Call `OnUnload` hook
2. Cleanup resources
3. Unload runtime

## Error Handling

If `OnLoad` returns `Error`:
- Plugin is marked as failed
- Plugin is disabled
- Error is logged
- Other plugins continue loading

## Best Practices

1. **Initialize fast** - Keep OnLoad < 100ms
2. **Clean up** - Always implement OnUnload
3. **Handle errors** - Don't crash the system
4. **Log appropriately** - Use correct log levels

## Next Steps

→ **[Frontend vs Backend](frontend-vs-backend.md)**

→ **[Performance Guide](performance.md)**
