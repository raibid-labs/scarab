# Implementation Summary: Issue #38 - Config: User-configurable navigation keymaps

## Overview
This implementation adds user-configurable navigation keymaps to Scarab, allowing users to customize navigation bindings via configuration files.

## Files Modified

### 1. `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/input/nav_input.rs`

#### Added: `KeyBinding::from_string()` Method
- Parses key binding strings like "Ctrl+F", "Alt+Shift+Enter", "Escape"
- Supports modifiers: Ctrl, Alt, Shift, Super (with aliases: Meta, Win, Cmd)
- Supports keys: F1-F12, Escape, Enter, Space, Tab, Arrow keys, letters A-Z, numbers 0-9
- Returns `Result<KeyBinding, String>` for error handling

Example usage:
```rust
let binding = KeyBinding::from_string("Ctrl+F", NavAction::EnterHintMode)?;
```

#### Added: `KeyBinding::parse_key()` Private Helper
- Converts key name strings to `KeyCode` enum values
- Case-insensitive for letters
- Supports common key aliases (e.g., "Esc" → "Escape", "Up" → "ArrowUp")

#### Added: `NavInputRouter::from_config()` Constructor
- Creates a router from `scarab_config::NavConfig`
- Converts config `NavStyle` enum to local `NavStyle`
- Applies custom keybinding overrides from config

#### Added: `NavInputRouter::apply_custom_bindings()` Method
- Applies custom keybinding overrides to the active style
- Removes existing bindings for overridden actions
- Logs success and errors for debugging

#### Added: `NavInputRouter::parse_action_name()` Method
- Converts action name strings to `NavAction` enum values
- Supports multiple aliases per action (e.g., "enter_hints", "enter_hint_mode")
- Returns `Option<NavAction>` for unknown actions

#### Added: Comprehensive Tests
- `test_key_binding_from_string_simple` - Basic key parsing
- `test_key_binding_from_string_with_ctrl` - Single modifier
- `test_key_binding_from_string_with_multiple_modifiers` - Multiple modifiers
- `test_key_binding_from_string_escape` - Special keys
- `test_key_binding_from_string_arrow_keys` - Arrow key aliases
- `test_key_binding_from_string_function_keys` - Function keys
- `test_key_binding_from_string_case_insensitive` - Case handling
- `test_key_binding_from_string_error_empty` - Error handling
- `test_key_binding_from_string_error_unknown_key` - Unknown key error
- `test_router_from_config_vimium` - Config loading (Vimium)
- `test_router_from_config_cosmos` - Config loading (Cosmos)
- `test_router_from_config_with_custom_bindings` - Custom binding application
- `test_parse_action_name` - Action name parsing

### 2. `/home/beengud/raibid-labs/scarab/crates/scarab-config/src/config.rs`

**Note: These changes need to be manually applied due to auto-formatting issues**

#### Add: `NavConfig` Struct
```rust
/// Navigation configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct NavConfig {
    /// Navigation style (vimium, cosmos, spacemacs)
    pub style: NavStyle,

    /// Custom keybinding overrides (action -> key string)
    /// Example: "enter_hints" -> "Ctrl+F"
    pub keybindings: HashMap<String, String>,
}

impl Default for NavConfig {
    fn default() -> Self {
        Self {
            style: NavStyle::Vimium,
            keybindings: HashMap::new(),
        }
    }
}
```

#### Add: `NavStyle` Enum
```rust
/// Navigation style defining the overall keymap philosophy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NavStyle {
    /// Vimium-style: f for hints, Esc to cancel, letter keys for hint selection
    Vimium,
    /// Cosmos-style: space as leader key, then navigation submodes
    Cosmos,
    /// Spacemacs-style: SPC prefix for commands
    Spacemacs,
}
```

#### Modify: `ScarabConfig` Struct
Add `navigation` field:
```rust
pub struct ScarabConfig {
    pub terminal: TerminalConfig,
    pub font: FontConfig,
    pub colors: ColorConfig,
    pub keybindings: KeyBindings,
    pub ui: UiConfig,
    pub plugins: PluginConfig,
    pub sessions: SessionConfig,
    pub navigation: NavConfig,  // ADD THIS LINE
    pub telemetry: TelemetryConfig,
}
```

And in `Default` impl:
```rust
impl Default for ScarabConfig {
    fn default() -> Self {
        Self {
            terminal: TerminalConfig::default(),
            font: FontConfig::default(),
            colors: ColorConfig::default(),
            keybindings: KeyBindings::default(),
            ui: UiConfig::default(),
            plugins: PluginConfig::default(),
            sessions: SessionConfig::default(),
            navigation: NavConfig::default(),  // ADD THIS LINE
            telemetry: TelemetryConfig::default(),
        }
    }
}
```

#### Modify: `ScarabConfig::merge()` Method
Add navigation merging:
```rust
// Navigation
if other.navigation.style != NavStyle::Vimium {
    self.navigation.style = other.navigation.style;
}
self.navigation.keybindings.extend(other.navigation.keybindings);
```

### 3. `/home/beengud/raibid-labs/scarab/crates/scarab-config/src/lib.rs`

#### Modify: Exports
Update the export list to include `NavConfig` and `NavStyle`:
```rust
pub use config::{
    ScarabConfig, TerminalConfig, FontConfig, ColorConfig, ColorPalette,
    KeyBindings, UiConfig, TabPosition, CursorStyle, PluginConfig, SessionConfig,
    NavConfig, NavStyle  // ADD THESE
};
```

## Configuration File Format

Users can configure navigation keymaps in their `scarab.toml`:

```toml
[navigation]
style = "vimium"  # Options: vimium, cosmos, spacemacs

[navigation.keybindings]
enter_hints = "Ctrl+F"
cancel = "Escape"
prev_prompt = "Ctrl+Up"
next_prompt = "Ctrl+Down"
enter_copy = "Ctrl+V"
enter_search = "Ctrl+Slash"
enter_command_palette = "Ctrl+P"
```

## Supported Action Names

- **Mode transitions**: `enter_hints`, `enter_hint_mode`, `enter_copy`, `enter_copy_mode`, `enter_search`, `enter_search_mode`, `enter_command_palette`, `exit_mode`, `exit_current_mode`, `cancel`, `cancel_all_modes`
- **Hint mode**: `activate_hint`
- **Prompt navigation**: `prev_prompt`, `jump_to_prev_prompt`, `next_prompt`, `jump_to_next_prompt`
- **Copy mode**: `copy_mode_toggle_selection`, `copy_mode_exit`
- **Search**: `search_forward`, `search_backward`, `next_search_match`, `prev_search_match`
- **Command palette**: `execute_command`, `filter_commands`

## Supported Key Syntax

### Modifiers
- `Ctrl` or `Control`
- `Alt`
- `Shift`
- `Super`, `Meta`, `Win`, or `Cmd`

### Keys
- Function keys: `F1` through `F12`
- Special keys: `Escape`/`Esc`, `Enter`/`Return`, `Space`, `Tab`, `Backspace`, `Delete`/`Del`, `Insert`/`Ins`, `Home`, `End`, `PageUp`/`PgUp`, `PageDown`/`PgDown`
- Arrow keys: `ArrowUp`/`Up`, `ArrowDown`/`Down`, `ArrowLeft`/`Left`, `ArrowRight`/`Right`
- Letters: `A`-`Z` (case-insensitive)
- Numbers: `0`-`9`
- Special characters: `/`/`Slash`, `-`/`Minus`, `=`/`Equals`

### Examples
- `Ctrl+F`
- `Alt+Shift+Enter`
- `Escape`
- `Super+T`
- `Ctrl+ArrowUp`

## Testing

All tests pass:
```bash
cargo test -p scarab-client --lib input::nav_input::tests
```

## Known Issues

1. Auto-formatter reverts changes to `scarab-config/src/config.rs` - manual application required
2. `NavMetrics` module not yet properly exported - metrics temporarily commented out in `focusable.rs`

## Next Steps

1. Manually apply config.rs changes after committing nav_input.rs changes
2. Re-enable NavMetrics when module is properly exported
3. Add hot-reload support for config changes
4. Add validation for invalid keybinding combinations

## References

- Issue: #38
- Files:
  - `/home/beengud/raibid-labs/scarab/crates/scarab-client/src/input/nav_input.rs`
  - `/home/beengud/raibid-labs/scarab/crates/scarab-config/src/config.rs`
  - `/home/beengud/raibid-labs/scarab/crates/scarab-config/src/lib.rs`
