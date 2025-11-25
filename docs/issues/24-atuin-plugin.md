# Issue #24: Atuin Plugin for Scarab

## 🎯 Goal
Create a Fusabi plugin that integrates [Atuin](https://github.com/atuinsh/atuin) shell history with Scarab Terminal.

## 🐛 Problem
Users want enhanced shell history features:
- **Cross-session history**: Sync history across all terminal sessions
- **Advanced search**: Fuzzy search, context-aware history
- **Statistics**: Command usage analytics
- **Cloud sync**: Share history across machines

Atuin provides all this, but needs integration with Scarab.

## 💡 Proposed Solution

Create a Fusabi plugin that:
1. Detects Atuin installation
2. Hooks into shell command execution
3. Provides UI overlay for Atuin search
4. Integrates with Scarab's command palette

### Architecture

```
┌─────────────────────────────────────┐
│  Scarab Client (Bevy UI)            │
│  ┌───────────────────────────────┐  │
│  │ Atuin Search Overlay          │  │ ← Rendered by plugin
│  │ > git commit ___              │  │
│  │   git commit -m "..."         │  │
│  │   git commit --amend          │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
              ↕ IPC
┌─────────────────────────────────────┐
│  Scarab Daemon (Plugin Manager)     │
│  ┌───────────────────────────────┐  │
│  │ scarab-atuin.fzb              │  │ ← Fusabi bytecode plugin
│  │ - on_key_press (Ctrl+R)       │  │
│  │ - query_atuin()               │  │
│  │ - send_ui_command()           │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
              ↕ Shell
┌─────────────────────────────────────┐
│  Atuin CLI                           │
│  $ atuin search --filter git        │
└─────────────────────────────────────┘
```

## 📋 Implementation Tasks

### Phase 1: Plugin Scaffold (half day)
- [ ] Create `plugins/scarab-atuin/` directory
- [ ] Write `scarab-atuin.fsx` (Fusabi source)
- [ ] Define plugin manifest (name, version, hooks)
- [ ] Set up build for `.fzb` bytecode

### Phase 2: Atuin Integration (1 day)
- [ ] Check for Atuin installation (`which atuin`)
- [ ] Hook `Ctrl+R` key combination
- [ ] Execute `atuin search` via shell
- [ ] Parse Atuin output (JSON format)
- [ ] Send results to UI overlay

### Phase 3: UI Overlay (1 day)
- [ ] Create search overlay component
- [ ] Display Atuin results in list
- [ ] Add fuzzy search filtering
- [ ] Arrow keys for navigation
- [ ] Enter to insert command

### Phase 4: Configuration (half day)
- [ ] Add plugin settings to `~/.config/scarab/plugins/atuin.toml`
- [ ] Configurable keybinding (default: Ctrl+R)
- [ ] Optional auto-sync on command execution
- [ ] Result limit setting (default: 20)

## 📝 Fusabi Plugin Structure

### `plugins/scarab-atuin/scarab-atuin.fsx`

```fsharp
module ScarabAtuin

open Scarab.PluginApi

// Plugin metadata
[<Plugin>]
let metadata = {
    Name = "scarab-atuin"
    Version = "0.1.0"
    Description = "Atuin shell history integration"
    Author = "Scarab Team"
}

// Check if Atuin is installed
let checkAtuinInstalled () =
    match executeCommand "which atuin" with
    | Ok _ -> true
    | Error _ ->
        notify "Atuin not found" "Please install: cargo install atuin" Warning
        false

// Query Atuin for history
let queryAtuin (filter: string) : Result<string list, string> =
    let cmd = sprintf "atuin search --limit 20 --format json '%s'" filter
    match executeCommand cmd with
    | Ok output ->
        // Parse JSON output
        let entries = parseJson output |> extractCommands
        Ok entries
    | Error err -> Error err

// Handle Ctrl+R key press
[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    if key.Modifiers.Contains(Control) && key.Key = "r" then
        if not (checkAtuinInstalled ()) then
            ()
        else
            // Open search overlay
            ctx.RemoteUI.OpenOverlay "atuin-search" {
                Title = "Atuin History Search"
                InputPlaceholder = "Search commands..."
                OnInput = fun query ->
                    match queryAtuin query with
                    | Ok results -> ctx.RemoteUI.UpdateResults results
                    | Error err -> ctx.Log Error err
            }

// On overlay result selected
[<OnOverlayResult>]
let onOverlayResult (ctx: PluginContext) (overlayId: string) (result: string) =
    if overlayId = "atuin-search" then
        // Insert command at cursor
        ctx.SendInput result
```

## 🧪 Testing

### Manual Tests
1. Install Atuin: `cargo install atuin`
2. Run some commands to populate history
3. Launch Scarab, press `Ctrl+R`
4. Verify search overlay appears
5. Type search term, verify filtering works
6. Select result, verify command inserted

### Automated Tests
```rust
#[test]
fn test_atuin_detection() {
    let plugin = AtuinPlugin::new();
    assert!(plugin.check_atuin_installed());
}

#[test]
fn test_query_parsing() {
    let json_output = r#"[{"command": "git status", "timestamp": 1234}]"#;
    let results = parse_atuin_output(json_output);
    assert_eq!(results[0], "git status");
}
```

## 🎨 UI Mockup

```
┌────────────────────────────────────────────────────┐
│ $ ls -la                                           │
│ total 48                                           │
│ drwxr-xr-x  6 user  staff   192 Nov 24 10:00 .   │
├────────────────────────────────────────────────────┤
│ Atuin History Search (Ctrl+R)                      │
│ ┌────────────────────────────────────────────────┐ │
│ │ Search: git___                                 │ │
│ └────────────────────────────────────────────────┘ │
│                                                    │
│ > git commit -m "feat: Add atuin plugin"          │ ← Selected
│   git commit --amend                              │
│   git status                                      │
│   git log --oneline                               │
│   git push origin main                            │
│                                                    │
│ 5 results • ↑↓ navigate • Enter select • Esc close│
└────────────────────────────────────────────────────┘
```

## 📚 Atuin API Reference

### Command-Line Interface
```bash
# Search history
atuin search [query] --limit N --format json

# Get statistics
atuin stats --period day|week|month

# Sync history
atuin sync
```

### JSON Output Format
```json
[
  {
    "command": "git status",
    "timestamp": 1700000000,
    "duration": 123,
    "exit": 0,
    "hostname": "laptop",
    "cwd": "/home/user/project"
  }
]
```

## 🔗 Configuration

### `~/.config/scarab/plugins/atuin.toml`
```toml
[atuin]
enabled = true

# Keybinding for history search
keybinding = "Ctrl+R"

# Maximum results to show
max_results = 20

# Auto-sync after each command
auto_sync = false

# Show command statistics in overlay
show_stats = true
```

## 📊 Success Criteria

- [ ] Plugin detects Atuin installation
- [ ] `Ctrl+R` opens search overlay
- [ ] Search filters history in real-time
- [ ] Selected command inserts into terminal
- [ ] Works with Atuin cloud sync
- [ ] No performance impact on regular terminal usage

## 🔗 Related Issues

- Issue #23: Scrollback UI (complementary history feature)
- Issue #27: Plugin Development Documentation (examples)

---

**Priority**: 🟡 HIGH
**Effort**: 2 days
**Assignee**: AI Engineer + Frontend Developer
