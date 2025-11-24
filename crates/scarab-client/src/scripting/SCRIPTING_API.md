# Scarab Client-Side Scripting API

## Overview

Scarab's client-side scripting system uses **Fusabi** (F# dialect) to provide hot-reloadable UI customization. Scripts are written in `.fsx` files and interpreted by `fusabi-frontend` at runtime.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  Client-Side Scripts (.fsx)                     │
│  - Interpreted by fusabi-frontend               │
│  - Hot-reloadable (no restart needed)           │
│  - Access to Bevy resources via ScriptAPI       │
└─────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│  ScriptManager (Bevy Resource)                  │
│  - Loads scripts from ~/.config/scarab/scripts  │
│  - Watches files for changes (500ms interval)   │
│  - Executes scripts with RuntimeContext         │
└─────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│  ScriptRuntime                                  │
│  - Parses .fsx with fusabi-frontend             │
│  - Executes in sandboxed environment            │
│  - Emits ScriptEvents to Bevy                   │
└─────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│  Bevy Systems                                   │
│  - Apply color changes                          │
│  - Render overlays                              │
│  - Update window properties                     │
└─────────────────────────────────────────────────┘
```

## Script Lifecycle

1. **Discovery**: On startup, ScriptLoader scans `~/.config/scarab/scripts/*.fsx`
2. **Loading**: Scripts are loaded into memory with metadata
3. **Watching**: ScriptWatcher monitors files for modifications
4. **Execution**: Scripts run with access to RuntimeContext
5. **Hot-Reload**: On file save, script is reloaded and re-executed
6. **Event Emission**: Scripts emit events via ScriptAPI
7. **Bevy Processing**: Events are handled by Bevy systems

## ScriptAPI Reference

### Colors

#### `Scarab.setColor(name: string, color: string)`
Set a color by name using hex notation.

**Parameters:**
- `name`: Color identifier (see available names below)
- `color`: Hex color string (e.g., `"#ff5555"` or `"#ff555580"` with alpha)

**Available Color Names:**
- `foreground`, `background`, `cursor`
- `selection_bg`, `selection_fg`
- ANSI colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- Bright colors: `bright_black`, `bright_red`, `bright_green`, etc.

**Example:**
```fsharp
Scarab.setColor "foreground" "#f8f8f2"
Scarab.setColor "background" "#282a36"
Scarab.setColor "cursor" "#ff79c6"
```

### Fonts

#### `Scarab.setFont(family: string, size: float)`
Change font properties.

**Parameters:**
- `family`: Font family name (e.g., `"JetBrains Mono"`)
- `size`: Font size in points

**Example:**
```fsharp
Scarab.setFont "JetBrains Mono" 16.0
Scarab.setFont "Fira Code" 14.0
```

### Window

#### `Scarab.setWindowTitle(title: string)`
Set the window title.

**Parameters:**
- `title`: New window title text

**Example:**
```fsharp
Scarab.setWindowTitle "Scarab - Production Terminal"
Scarab.setWindowTitle "Scarab [Building...]"
```

### Overlays

#### `Scarab.addOverlay(name: string, position: Position, content: Content)`
Add a custom overlay to the terminal UI.

**Parameters:**
- `name`: Unique identifier for the overlay
- `position`: Where to place the overlay (see positions below)
- `content`: What to display (see content types below)

**Positions:**
- `TopLeft`, `TopCenter`, `TopRight`
- `CenterLeft`, `Center`, `CenterRight`
- `BottomLeft`, `BottomCenter`, `BottomRight`

**Content Types:**

**Text:**
```fsharp
Scarab.addOverlay "status" "TopRight" {
    type = "Text"
    text = "Ready"
    size = 12.0
    color = "#50fa7b"
}
```

**Box:**
```fsharp
Scarab.addOverlay "indicator" "TopLeft" {
    type = "Box"
    width = 10.0
    height = 10.0
    color = "#ff5555"
    border_color = "#f8f8f2"
    border_width = 2.0
}
```

**Custom Widget:**
```fsharp
Scarab.addOverlay "git-status" "TopRight" {
    type = "Custom"
    widget_type = "GitBranch"
    properties = {
        branch = "main"
        status = "clean"
    }
}
```

#### `Scarab.removeOverlay(name: string)`
Remove a previously added overlay.

**Parameters:**
- `name`: Identifier of the overlay to remove

**Example:**
```fsharp
Scarab.removeOverlay "status"
```

### Commands

#### `Scarab.registerCommand(name: string, description: string, keybinding: Option<string>)`
Register a custom command in the command palette.

**Parameters:**
- `name`: Command identifier
- `description`: User-visible description
- `keybinding`: Optional keyboard shortcut (e.g., `Some "Ctrl+Shift+T"`)

**Example:**
```fsharp
Scarab.registerCommand "toggle-theme" "Toggle between light and dark themes" (Some "Ctrl+Shift+T")
Scarab.registerCommand "reload-config" "Reload configuration" None
```

## RuntimeContext

Scripts have read-only access to the current application context:

### Colors Context
```fsharp
context.colors.foreground    // Current foreground color
context.colors.background    // Current background color
context.colors.cursor        // Current cursor color
context.colors.selection_bg  // Selection background
context.colors.selection_fg  // Selection foreground
context.colors.palette       // Vec<Color> (16 ANSI colors)
```

### Fonts Context
```fsharp
context.fonts.family         // Current font family (String)
context.fonts.size           // Current font size (f32)
context.fonts.line_height    // Line height multiplier (f32)
```

### Window Context
```fsharp
context.window.width         // Window width in pixels (f32)
context.window.height        // Window height in pixels (f32)
context.window.scale_factor  // DPI scale factor (f32)
context.window.title         // Current window title (String)
```

### Terminal Context
```fsharp
context.terminal.rows        // Terminal rows (u16)
context.terminal.cols        // Terminal columns (u16)
context.terminal.scrollback_lines  // Scrollback buffer size (u32)
```

## Error Handling

Scripts that encounter errors will:
1. Have the error logged to the console
2. Display an error overlay in the UI (if enabled)
3. Not crash the client - errors are isolated

**Common Errors:**
- **Parse Error**: Invalid F# syntax
- **Compile Error**: Type mismatch or undefined function
- **Runtime Error**: Exception during execution
- **API Error**: Invalid API call or parameters

**Example Error Output:**
```
Script Error in 'custom-theme.fsx' at line 5:
  Parse error: Expected string, found int
```

## Best Practices

### 1. Use Comments
```fsharp
// Configure Dracula theme colors
Scarab.setColor "background" "#282a36"
Scarab.setColor "foreground" "#f8f8f2"
```

### 2. Group Related Operations
```fsharp
// Normal colors
Scarab.setColor "black" "#21222c"
Scarab.setColor "red" "#ff5555"
Scarab.setColor "green" "#50fa7b"

// Bright colors
Scarab.setColor "bright_black" "#6272a4"
Scarab.setColor "bright_red" "#ff6e6e"
```

### 3. Handle Optional Values
```fsharp
// With keybinding
Scarab.registerCommand "foo" "Description" (Some "Ctrl+Shift+F")

// Without keybinding
Scarab.registerCommand "bar" "Description" None
```

### 4. Keep Scripts Focused
One script per concern:
- `theme.fsx` - Colors only
- `overlays.fsx` - UI overlays
- `window.fsx` - Window customization

### 5. Test Incrementally
Save often to see changes immediately. Fix errors as they appear.

## Configuration

### Scripts Directory
Default: `~/.config/scarab/scripts/`

Override in `config.toml`:
```toml
[plugins]
config.scripts_directory = "/custom/path/to/scripts"
```

### Hot-Reload Interval
The file watcher checks for changes every 500ms by default.

## Advanced Patterns

### Dynamic Themes
```fsharp
// Time-based theme switching
let hour = DateTime.Now.Hour
let isDarkMode = hour < 6 || hour >= 18

if isDarkMode then
    Scarab.setColor "background" "#282a36"
    Scarab.setColor "foreground" "#f8f8f2"
else
    Scarab.setColor "background" "#f8f8f2"
    Scarab.setColor "foreground" "#282a36"
```

### Conditional Overlays
```fsharp
// Show build status only when building
let isBuildRunning = checkBuildProcess()

if isBuildRunning then
    Scarab.addOverlay "build" "TopRight" {
        type = "Text"
        text = "Building..."
        size = 11.0
        color = "#f1fa8c"
    }
else
    Scarab.removeOverlay "build"
```

### Context-Aware Titles
```fsharp
// Dynamic title based on terminal size
let size = sprintf "%dx%d" context.terminal.cols context.terminal.rows
Scarab.setWindowTitle (sprintf "Scarab [%s]" size)
```

## Integration with Daemon

While client scripts run in the client process, they can:
- React to daemon events (via event handlers)
- Display information from shared memory
- Customize how daemon data is presented

**Note**: For daemon-side logic (PTY hooks, output scanning), use daemon plugins with `.fzb` bytecode instead.

## Performance Considerations

- Scripts execute synchronously on the main thread
- Keep scripts lightweight (< 100 lines recommended)
- Avoid heavy computation in hot-reload loops
- Use conditional logic to minimize redundant operations

## Troubleshooting

### Script Not Loading
```bash
# Check if file exists
ls -la ~/.config/scarab/scripts/*.fsx

# Check permissions
chmod +r ~/.config/scarab/scripts/*.fsx

# Check logs
grep "script" ~/.config/scarab/client.log
```

### Script Not Reloading
- Ensure file is saved (check modification time)
- Verify no syntax errors (check logs)
- Try manual reload: restart client

### API Not Working
- Check function names (case-sensitive)
- Verify parameters are correct types
- Look for error messages in UI or logs

## Future Enhancements

Planned features:
- Event subscriptions (daemon events, key presses)
- State persistence between reloads
- Inter-script communication
- Full F# language support via fusabi-frontend
- Async/await for non-blocking operations
- Package manager for community scripts

## Examples Repository

See `~/.config/scarab/scripts/` for working examples:
- `custom-theme.fsx` - Color customization
- `custom-overlay.fsx` - UI overlays
- `window-title.fsx` - Dynamic window title

## Contributing

To contribute new API functions:
1. Add to `ScriptApi` struct in `scripting/api.rs`
2. Emit corresponding `ScriptEvent` variant
3. Handle event in Bevy systems
4. Update this documentation
5. Add example script

## License

Scarab scripting system is part of Scarab Terminal Emulator.
See main project license for details.
