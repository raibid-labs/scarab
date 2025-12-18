# WezTerm Lua Configuration System - Comprehensive Analysis

Research conducted: 2025-12-01
Target: Understanding WezTerm's Lua configuration capabilities for comparison with Scarab's Fusabi plugin system

## Executive Summary

WezTerm provides one of the most comprehensive Lua-based configuration systems in the terminal emulator space. It uses Lua 5.4 as both a configuration language and a runtime scripting environment, exposing 200+ configuration options and extensive APIs for event handling, UI customization, and programmatic control.

**Key Strengths:**
- Deep integration between configuration and runtime behavior
- Rich event system with 12+ hookable events
- Powerful programmatic control of panes, tabs, and windows
- Plugin ecosystem with git-based distribution
- Cross-platform multiplexing with SSH, WSL, and Unix domains

**Key Differentiator vs Traditional Config:** WezTerm blurs the line between "configuration" and "scripting" - you can execute arbitrary Lua code, call external programs, make network requests, and dynamically modify behavior at runtime.

---

## 1. Core Lua API Modules

WezTerm exposes several namespaced modules:

### 1.1 Primary Module: `wezterm`
```lua
local wezterm = require 'wezterm'
```

**Key Functions:**
- `wezterm.on(event_name, callback)` - Register event handlers
- `wezterm.action_callback(fn)` - Create custom actions for keybindings
- `wezterm.action.*` - Predefined actions (200+ available)
- `wezterm.font()` / `wezterm.font_with_fallback()` - Font configuration
- `wezterm.format()` - Styled text formatting for UI elements
- `wezterm.log_info()` / `wezterm.log_error()` - Logging utilities
- `wezterm.config_builder()` - Create config tables with validation

### 1.2 Sub-modules

**wezterm.color** - Color manipulation
- `parse()` - Parse color strings into color objects
- `get_builtin_schemes()` - Retrieve all builtin color schemes
- `load_scheme()` - Load color scheme by name
- `get_default_colors()` - Get default color palette
- HSLA analysis for dynamic theming

**wezterm.gui** - GUI window and GPU operations
- `get_appearance()` - Detect system dark/light mode
- `screens()` - Query available displays
- GPU-accelerated rendering control

**wezterm.mux** - Multiplexer operations
- `get_pane()` / `get_tab()` / `get_window()` - Get objects by ID
- `all_panes()` / `all_tabs()` / `all_windows()` - Enumerate all objects
- `spawn_tab()` / `spawn_window()` - Create new tabs/windows
- Domain management (local, SSH, WSL, Unix socket)

**wezterm.plugin** - Plugin management
- `require(url)` - Load plugin from git URL or local path
- `update_all()` - Update all installed plugins
- `list()` - List installed plugins

**wezterm.procinfo** - Process information
- `get_info_for_pid(pid)` - Query process details
- `current_working_dir_for_pid(pid)` - Get process CWD
- Note: Only works for local processes, not over SSH/multiplexer

**wezterm.serde** - Serialization
- `json_encode()` / `json_decode()` - JSON serialization
- `toml_encode()` / `toml_decode()` - TOML serialization
- `yaml_encode()` / `yaml_decode()` - YAML serialization

**wezterm.time** - Time operations
- `now()` - Get current timestamp
- `parse()` - Parse time strings
- Formatting utilities

**wezterm.url** - URL parsing
- `parse()` - Parse URLs into components

---

## 2. Event System (wezterm.on)

WezTerm provides 12+ events that can be hooked with `wezterm.on()`:

### 2.1 Lifecycle Events

**gui-startup** - Fired when GUI starts
```lua
wezterm.on('gui-startup', function(cmd)
  -- cmd: SpawnCommand for the initial program
  -- Setup initial windows/tabs/workspaces
end)
```

**gui-attached** - GUI connects to mux server
```lua
wezterm.on('gui-attached', function(domain)
  -- Triggered when GUI attaches to multiplexer
end)
```

**mux-startup** - Multiplexer server starts
```lua
wezterm.on('mux-startup', function()
  -- Server-side initialization
end)
```

### 2.2 Window Events

**window-config-reloaded** - Configuration reloaded
```lua
wezterm.on('window-config-reloaded', function(window, pane)
  -- React to config changes
  window:toast_notification('wezterm', 'Config reloaded!', nil, 4000)
end)
```

**window-focus-changed** - Window gains/loses focus
```lua
wezterm.on('window-focus-changed', function(window, pane)
  local is_focused = window:is_focused()
  -- Adjust opacity, blur, or other visual effects
end)
```

**window-resized** - Window size changed
```lua
wezterm.on('window-resized', function(window, pane)
  -- React to dimension changes
end)
```

### 2.3 UI Customization Events

**format-tab-title** - Customize tab title rendering
```lua
wezterm.on('format-tab-title', function(tab, tabs, panes, config, hover, max_width)
  local title = tab.active_pane.title
  if tab.is_active then
    return { { Text = ' [' .. title .. '] ' } }
  end
  return title
end)
```

**format-window-title** - Customize window title
```lua
wezterm.on('format-window-title', function(tab, pane, tabs, panes, config)
  return 'WezTerm - ' .. pane.title
end)
```

**update-status** - Update status line (left side)
```lua
wezterm.on('update-status', function(window, pane)
  window:set_left_status(wezterm.format({
    { Text = pane:get_current_working_dir() }
  }))
end)
```

**update-right-status** - Update right status bar
```lua
wezterm.on('update-right-status', function(window, pane)
  window:set_right_status(wezterm.format({
    { Text = os.date('%H:%M:%S') }
  }))
end)
```

**new-tab-button-click** - Handle new tab button clicks
```lua
wezterm.on('new-tab-button-click', function(window, pane, button, default_action)
  if button == 'Left' then
    window:perform_action(wezterm.action.SpawnTab 'CurrentPaneDomain', pane)
    return false -- prevent default action
  end
  return true -- allow default
end)
```

**augment-command-palette** - Add custom commands
```lua
wezterm.on('augment-command-palette', function(window, pane)
  return {
    {
      brief = 'Rename tab',
      icon = 'md_rename_box',
      action = wezterm.action.PromptInputLine {
        description = 'Enter new name for tab',
        action = wezterm.action_callback(function(window, pane, line)
          if line then
            window:active_tab():set_title(line)
          end
        end),
      },
    },
  }
end)
```

### 2.4 Interaction Events

**bell** - Terminal bell event
```lua
wezterm.on('bell', function(window, pane)
  wezterm.log_info('Bell triggered in pane: ' .. pane:pane_id())
  window:toast_notification('WezTerm', 'Bell!', nil, 2000)
end)
```

**open-uri** - URI clicked/opened
```lua
wezterm.on('open-uri', function(window, pane, uri)
  wezterm.log_info('Opening URI: ' .. uri)
  -- Return false to prevent default handler
  -- Return true to allow default handler
  return true
end)
```

**user-var-changed** - User variable changed via OSC escape
```lua
wezterm.on('user-var-changed', function(window, pane, name, value)
  wezterm.log_info('User var changed: ' .. name .. ' = ' .. value)
  -- Can trigger UI updates, status changes, etc.
end)
```

### 2.5 Multiplexer Events

**mux-is-process-stateful** - Determine if process should trigger save prompt
```lua
wezterm.on('mux-is-process-stateful', function(proc)
  -- Return true if process is stateful (e.g., editors, databases)
  return proc.name:find('nvim') or proc.name:find('vim')
end)
```

---

## 3. Custom Key Bindings & Leader Keys

### 3.1 Basic Key Bindings
```lua
config.keys = {
  {
    key = 't',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.SpawnTab 'CurrentPaneDomain',
  },
  {
    key = 'w',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.CloseCurrentTab { confirm = true },
  },
}
```

### 3.2 Leader Key (Modal Modifier)
```lua
config.leader = { key = 'a', mods = 'CTRL', timeout_milliseconds = 1000 }

config.keys = {
  {
    key = '|',
    mods = 'LEADER|SHIFT',
    action = wezterm.action.SplitHorizontal { domain = 'CurrentPaneDomain' },
  },
  {
    key = '-',
    mods = 'LEADER',
    action = wezterm.action.SplitVertical { domain = 'CurrentPaneDomain' },
  },
}
```

**Leader behavior:**
- Leader key activates a modal state
- Only keybindings with `LEADER` in mods are recognized while active
- Automatically deactivates after timeout or when a key is pressed

### 3.3 Custom Actions with Lua Callbacks
```lua
config.keys = {
  {
    key = 'E',
    mods = 'CTRL|SHIFT',
    action = wezterm.action_callback(function(window, pane)
      local cwd = pane:get_current_working_dir()
      wezterm.log_info('Current directory: ' .. tostring(cwd))
      -- Can call any Lua code, spawn processes, etc.
    end),
  },
}
```

### 3.4 Key Tables (Modal Key Maps)
```lua
config.key_tables = {
  resize_pane = {
    { key = 'h', action = wezterm.action.AdjustPaneSize { 'Left', 1 } },
    { key = 'l', action = wezterm.action.AdjustPaneSize { 'Right', 1 } },
    { key = 'Escape', action = 'PopKeyTable' },
  },
}

config.keys = {
  {
    key = 'r',
    mods = 'LEADER',
    action = wezterm.action.ActivateKeyTable {
      name = 'resize_pane',
      one_shot = false,
    },
  },
}
```

### 3.5 Copy Mode (VI Mode)
Pre-configured VI-style keybindings for text selection:
- `v` - Toggle selection mode
- `h/j/k/l` - Navigation
- `w/b/e` - Word navigation
- `0/$` - Line start/end
- `gg/G` - Buffer top/bottom
- `y` - Yank selection
- `q` - Exit copy mode

Can be fully customized via `copy_mode` key table.

---

## 4. Mouse Bindings

```lua
config.mouse_bindings = {
  -- Right click sends text
  {
    event = { Down = { streak = 1, button = 'Right' } },
    mods = 'NONE',
    action = wezterm.action.SendString 'woot',
  },

  -- Triple-click selects line
  {
    event = { Down = { streak = 3, button = 'Left' } },
    mods = 'NONE',
    action = wezterm.action.SelectTextAtMouseCursor 'Line',
  },

  -- CTRL+click opens hyperlink
  {
    event = { Up = { streak = 1, button = 'Left' } },
    mods = 'CTRL',
    action = wezterm.action.OpenLinkAtMouseCursor,
  },

  -- Custom callback on click
  {
    event = { Down = { streak = 1, button = 'Middle' } },
    mods = 'NONE',
    action = wezterm.action_callback(function(window, pane)
      -- Custom logic here
    end),
  },
}
```

**Mouse event types:**
- `Down` / `Up` / `Drag` - Button states
- `streak` - Click count (1=single, 2=double, 3=triple, 4+=custom)
- `button` - 'Left', 'Right', 'Middle', 'WheelUp', 'WheelDown'

---

## 5. UI Customization

### 5.1 Tab Bar Customization

**Basic configuration:**
```lua
config.use_fancy_tab_bar = false  -- Use retro tab bar
config.hide_tab_bar_if_only_one_tab = true
config.show_tab_index_in_tab_bar = true
config.tab_max_width = 32
```

**Custom tab styling with format-tab-title event:**
```lua
wezterm.on('format-tab-title', function(tab, tabs, panes, config, hover, max_width)
  local background = '#1e1e2e'
  local foreground = '#cdd6f4'

  if tab.is_active then
    background = '#89b4fa'
    foreground = '#1e1e2e'
  elseif hover then
    background = '#313244'
  end

  local title = tab.active_pane.title
  return {
    { Background = { Color = background } },
    { Foreground = { Color = foreground } },
    { Text = ' ' .. title .. ' ' },
  }
end)
```

### 5.2 Status Bar (Powerline-style)

```lua
wezterm.on('update-right-status', function(window, pane)
  local cells = {}

  -- Current working directory
  local cwd_uri = pane:get_current_working_dir()
  if cwd_uri then
    local cwd = cwd_uri.file_path
    table.insert(cells, cwd)
  end

  -- Time
  table.insert(cells, os.date('%H:%M'))

  -- Render with powerline separators
  local text = table.concat(cells, ' | ')
  window:set_right_status(wezterm.format({
    { Attribute = { Intensity = 'Bold' } },
    { Text = text },
  }))
end)
```

### 5.3 Background Images & Transparency

```lua
config.window_background_opacity = 0.85

config.background = {
  {
    source = { File = '/path/to/image.png' },
    width = '100%',
    height = '100%',
    opacity = 0.3,
    hsb = { brightness = 0.05 },  -- Darken image
  },
  {
    source = { Color = '#1e1e2e' },
    width = '100%',
    height = '100%',
    opacity = 0.9,
  },
}

-- Platform-specific blur effects
config.macos_window_background_blur = 20  -- macOS
config.kde_window_background_blur = 10    -- Linux/KDE Wayland
config.win32_system_backdrop = 'Acrylic'  -- Windows 11
```

### 5.4 Toast Notifications

```lua
window:toast_notification(
  'Title',
  'Message text',
  'https://optional-url.com',  -- nil if no URL
  4000  -- timeout in milliseconds
)
```

---

## 6. Font Configuration

### 6.1 Basic Font Setup
```lua
config.font = wezterm.font('JetBrains Mono', { weight = 'Medium' })
config.font_size = 13.0

-- Font with fallbacks
config.font = wezterm.font_with_fallback({
  'JetBrains Mono',
  'Noto Sans Mono',
  'Symbols Nerd Font',
})
```

### 6.2 Font Rules (Style Overrides)
```lua
config.font_rules = {
  {
    intensity = 'Bold',
    italic = true,
    font = wezterm.font({
      family = 'JetBrains Mono',
      weight = 'Bold',
      style = 'Italic',
    }),
  },
}
```

### 6.3 Advanced Font Features
```lua
config.font = wezterm.font({
  family = 'JetBrains Mono',
  harfbuzz_features = {
    'calt=0',  -- Disable contextual alternates
    'clig=0',  -- Disable contextual ligatures
    'liga=0',  -- Disable standard ligatures
  },
  freetype_load_target = 'Light',
  freetype_render_target = 'HorizontalLcd',
})
```

### 6.4 Font System Details

**Rendering Stack:**
- FreeType - Font parsing and rasterization
- HarfBuzz - Text shaping (ligatures, complex scripts)
- Texture atlas caching - GPU-accelerated rendering

**Performance characteristics:**
- First render: Few milliseconds (BIDI + shaping)
- Subsequent renders: Cached, very fast
- Cache invalidation: On line update
- Scrollback scrolling: May blow cache (worst case)

---

## 7. Color Scheme Manipulation

### 7.1 Static Color Schemes
```lua
config.color_scheme = 'Catppuccin Mocha'

-- Or define custom scheme
config.color_schemes = {
  ['My Scheme'] = {
    foreground = '#cdd6f4',
    background = '#1e1e2e',
    cursor_bg = '#f5e0dc',
    cursor_border = '#f5e0dc',
    selection_bg = '#585b70',
    ansi = {...},
    brights = {...},
  },
}
```

### 7.2 Dynamic Color Scheme Selection
```lua
function scheme_for_appearance(appearance)
  if appearance:find('Dark') then
    return 'Catppuccin Mocha'
  else
    return 'Catppuccin Latte'
  end
end

config.color_scheme = scheme_for_appearance(wezterm.gui.get_appearance())
```

### 7.3 Runtime Color Manipulation
```lua
-- Get and modify builtin schemes
local schemes = wezterm.color.get_builtin_schemes()
local custom = schemes['Gruvbox Light']
custom.background = '#ffffff'

-- Analyze colors
local color = wezterm.color.parse('#89b4fa')
local h, s, l, a = color:hsla()
wezterm.log_info('Lightness: ' .. l)
```

### 7.4 Per-Window Color Overrides
```lua
wezterm.on('window-focus-changed', function(window, pane)
  if window:is_focused() then
    window:set_config_overrides({
      color_scheme = 'Catppuccin Mocha',
    })
  else
    window:set_config_overrides({
      window_background_opacity = 0.5,
    })
  end
end)
```

---

## 8. Multiplexing Features

### 8.1 Domain Types

**Local Domain** - Default, runs programs on local machine
```lua
-- Implicit, always available
```

**SSH Domain** - Connect to remote hosts
```lua
config.ssh_domains = {
  {
    name = 'my.server',
    remote_address = '192.168.1.1',
    username = 'wez',
    multiplexing = 'WezTerm',  -- or 'None' for direct SSH
  },
}
```

**Unix Domain** - Connect to local/WSL multiplexer
```lua
config.unix_domains = {
  { name = 'unix' },
}
```

**WSL Domain** - Windows Subsystem for Linux
```lua
config.wsl_domains = {
  {
    name = 'WSL:Ubuntu',
    distribution = 'Ubuntu',
  },
}
```

**Serial Domain** - Serial port connections
```lua
config.serial_ports = {
  {
    name = 'Arduino',
    port = '/dev/ttyUSB0',
    baud = 9600,
  },
}
```

### 8.2 Workspaces (Session Management)

```lua
-- Switch workspace
wezterm.action.SwitchToWorkspace {
  name = 'coding',
}

-- Create workspace with layout
wezterm.action_callback(function(window, pane)
  local mux = wezterm.mux
  local tab, pane, window = mux.spawn_window {
    workspace = 'coding',
    cwd = '/home/user/project',
  }
  pane:split { direction = 'Right' }
end)
```

**Workspace features:**
- Each MuxWindow belongs to a workspace
- GUI windows swap content when switching workspaces
- Persist across client disconnects (daemon continues)
- Can be scripted for complex layouts

### 8.3 Programmatic Pane/Tab Control

**Via wezterm.mux:**
```lua
local mux = wezterm.mux

-- Spawn new tab
local tab, pane, window = mux.spawn_tab { cwd = '/tmp' }

-- Get all panes
for _, pane in ipairs(mux.all_panes()) do
  wezterm.log_info('Pane ID: ' .. pane:pane_id())
end
```

**Via window/pane objects:**
```lua
-- In action callback
wezterm.action_callback(function(window, pane)
  local tab = window:active_tab()
  local new_pane = pane:split { direction = 'Bottom', size = 0.3 }
  new_pane:send_text('ls\n')
end)
```

---

## 9. URL Detection & Hyperlinks

### 9.1 Hyperlink Rules (Regex-based)
```lua
config.hyperlink_rules = {
  -- URLs in parentheses
  {
    regex = '\\((\\w+://\\S+)\\)',
    format = '$1',
    highlight = 1,
  },

  -- Explicit http/https URLs
  {
    regex = '\\bhttps?://\\S+\\b',
    format = '$0',
  },

  -- Implicit mailto
  {
    regex = '\\b\\w+@[\\w-]+(\\.[\\w-]+)+\\b',
    format = 'mailto:$0',
  },

  -- GitHub issues
  {
    regex = '\\b[a-z-]+/[a-z-]+#\\d+\\b',
    format = 'https://github.com/$0',
  },
}
```

**Important notes:**
- Uses Rust regex engine (not JavaScript)
- Backslashes must be escaped: `\\` in Lua strings
- `$0`, `$1`, `$2` = capture groups
- `highlight` specifies which capture to underline

### 9.2 Custom URI Handlers
```lua
wezterm.on('open-uri', function(window, pane, uri)
  if uri:match('^file://') then
    -- Custom file handler
    local path = uri:sub(8)
    wezterm.log_info('Opening file: ' .. path)
    return false  -- Prevent default
  end
  return true  -- Allow default
end)
```

---

## 10. Quick Select & Search

### 10.1 Quick Select Mode
```lua
config.quick_select_patterns = {
  '\\b[a-f0-9]{7,40}\\b',  -- Git hashes
  '\\b\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\b',  -- IP addresses
}

config.quick_select_alphabet = 'asdfghjkl'

-- Trigger with custom action
config.keys = {
  {
    key = 'Space',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.QuickSelectArgs {
      patterns = { '\\bhttps?://\\S+\\b' },
      action = wezterm.action_callback(function(window, pane, match)
        wezterm.open_with(match)
      end),
    },
  },
}
```

**Features:**
- Scans viewport + configurable lines above/below
- Displays one/two-character labels from alphabet
- Press label to select match
- Default patterns: URLs, paths, git hashes, IPs, numbers

### 10.2 Scrollback Search
```lua
config.keys = {
  {
    key = 'f',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.Search 'CurrentSelectionOrEmptyString',
  },
}
```

**Features:**
- Regex or plain text search
- Searches entire scrollback (not just viewport)
- Highlights all matches
- Navigates between matches

### 10.3 Semantic Zone Selection
```lua
config.mouse_bindings = {
  {
    event = { Down = { streak = 1, button = 'Left' } },
    mods = 'CTRL',
    action = wezterm.action.SelectTextAtMouseCursor 'SemanticZone',
  },
}
```

**Semantic zones** (requires shell integration):
- Command prompt
- Command output
- Entire command + output
- Allows selecting command output with single click

---

## 11. Command Palette & Launcher

### 11.1 Command Palette
Default key: `CTRL+SHIFT+P`

**Adding custom commands:**
```lua
wezterm.on('augment-command-palette', function(window, pane)
  return {
    {
      brief = 'Toggle ligatures',
      icon = 'md_format_text',
      action = wezterm.action_callback(function(window, pane)
        local overrides = window:get_config_overrides() or {}
        if overrides.harfbuzz_features then
          overrides.harfbuzz_features = nil
        else
          overrides.harfbuzz_features = { 'calt=0', 'clig=0', 'liga=0' }
        end
        window:set_config_overrides(overrides)
      end),
    },
  }
end)
```

**Custom palette fields:**
- `brief` - Short description (required)
- `doc` - Long description (optional)
- `icon` - Nerd Font icon name (optional)
- `action` - Action to perform

### 11.2 Launcher Menu
```lua
config.launch_menu = {
  {
    label = 'PowerShell',
    args = { 'pwsh.exe', '-NoLogo' },
  },
  {
    label = 'Bash',
    args = { 'bash', '-l' },
  },
}

-- Trigger with specific flags
config.keys = {
  {
    key = 'l',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.ShowLauncherArgs {
      flags = 'FUZZY|LAUNCH_MENU_ITEMS|TABS|DOMAINS',
      title = 'Select',
    },
  },
}
```

**Launcher flags:**
- `TABS` - Show tabs from current window
- `LAUNCH_MENU_ITEMS` - Show launch menu entries
- `DOMAINS` - Show available domains
- `WORKSPACES` - Show workspaces
- `COMMANDS` - Show command palette
- `FUZZY` - Start in fuzzy search mode

---

## 12. Programmatic Pane Control

### 12.1 Send Text to Pane
```lua
-- As keyboard input (respects bracketed paste)
pane:send_text('echo "hello"\n')

-- To output side (processes escape sequences)
pane:inject_output('\r\n\x1b[3mhello\r\n')
```

### 12.2 Read Pane Content
```lua
local lines = pane:get_lines_as_text()
local visible_text = pane:get_lines_as_text(pane:get_dimensions().viewport_rows)

-- With ANSI escape codes
local lines_with_escapes = pane:get_lines_as_escapes()
```

### 12.3 Query Pane State
```lua
local cwd = pane:get_current_working_dir()  -- Returns URI
local fg_proc = pane:get_foreground_process_name()
local user_vars = pane:get_user_vars()  -- OSC 1337 user vars
local dims = pane:get_dimensions()  -- Rows, cols, scrollback
```

### 12.4 CLI Control
```bash
# Send text to pane
wezterm cli send-text --pane-id 0 "ls -la"

# List panes
wezterm cli list

# Get pane ID
wezterm cli get-pane-direction next

# Spawn new pane
wezterm cli split-pane --horizontal
```

---

## 13. Shell Integration & User Variables

### 13.1 User Variables (OSC 1337)
**Set from shell:**
```bash
printf "\033]1337;SetUserVar=%s=%s\007" foo $(echo -n bar | base64)
```

**Read in Lua:**
```lua
wezterm.on('user-var-changed', function(window, pane, name, value)
  -- value is base64 encoded
  local decoded = wezterm.base64_decode(value)
  wezterm.log_info('User var ' .. name .. ' = ' .. decoded)

  -- Example: Update status when building
  if name == 'BUILD_STATUS' and decoded == 'running' then
    window:set_right_status('Building...')
  end
end)
```

### 13.2 Shell Integration Features
When enabled (via shell scripts), provides:
- Semantic zones (prompt, command, output)
- Current working directory tracking
- Command success/failure detection
- User variable setting helpers

**Example use case:**
```bash
# In shell prompt
printf "\033]1337;SetUserVar=%s=%s\007" \
  GIT_BRANCH $(git branch --show-current | base64)
```

```lua
-- In Lua config
wezterm.on('update-right-status', function(window, pane)
  local vars = pane:get_user_vars()
  local branch = vars.GIT_BRANCH and wezterm.base64_decode(vars.GIT_BRANCH) or ''
  if branch ~= '' then
    window:set_right_status('  ' .. branch)
  end
end)
```

---

## 14. Plugin System

### 14.1 Plugin Structure
A plugin must have an `init.lua` that exports `apply_to_config`:

```lua
-- init.lua
local M = {}

function M.apply_to_config(config, opts)
  config.color_scheme = opts.scheme or 'Catppuccin Mocha'
  -- ... more configuration
end

return M
```

### 14.2 Loading Plugins
```lua
-- From GitHub
local bar = wezterm.plugin.require 'https://github.com/owner/bar.wezterm'
bar.apply_to_config(config, { scheme = 'Gruvbox Dark' })

-- From local path
local my_plugin = wezterm.plugin.require 'file:///home/user/my-plugin'
my_plugin.apply_to_config(config)
```

### 14.3 Plugin Management
```bash
# Update all plugins
wezterm.plugin.update_all()

# List installed plugins
wezterm.plugin.list()

# Remove plugin (delete directory)
rm -rf ~/.local/share/wezterm/plugins/<name>
```

### 14.4 Notable Community Plugins

**tabline.wez** - Advanced tab bar styling
```lua
local tabline = wezterm.plugin.require 'https://github.com/michaelbrusegard/tabline.wez'
tabline.setup({
  options = {
    theme = 'Catppuccin Mocha',
    section_separators = { left = '', right = '' },
  },
  sections = {
    tabline_a = { 'mode' },
    tabline_b = { 'workspace' },
    tabline_c = { 'tabs' },
  },
})
tabline.apply_to_config(config)
```

**modal.wezterm** - Vim-like modal keybindings
```lua
local modal = wezterm.plugin.require 'https://github.com/MLFlexer/modal.wezterm'
modal.apply_to_config(config)
```

**smart_workspace_switcher.wezterm** - Fuzzy workspace finder
```lua
local switcher = wezterm.plugin.require 'https://github.com/MLFlexer/smart_workspace_switcher.wezterm'
switcher.apply_to_config(config)
```

---

## 15. Performance Optimization

### 15.1 Font Rendering Caching
```lua
-- Increase cache sizes (trades memory for performance)
-- Note: These are not direct config options, just conceptual
-- WezTerm automatically manages caches

-- What gets cached:
-- - BIDI analysis per line
-- - Font shaping per line
-- - Texture atlas for glyphs

-- Cache invalidation:
-- - Line updated by output
-- - Configuration changed
-- - Window resized
```

### 15.2 Rendering Configuration
```lua
config.front_end = 'WebGpu'  -- or 'OpenGL' or 'Software'
config.webgpu_power_preference = 'HighPerformance'  -- or 'LowPower'
config.animation_fps = 60
config.max_fps = 60

-- Reduce redraws for full-screen apps
-- (artificial throttle for output parsing)
config.visual_bell = {
  fade_in_duration_ms = 0,
  fade_out_duration_ms = 0,
}
```

### 15.3 Scrollback Limits
```lua
config.scrollback_lines = 10000  -- Balance memory vs. history

-- Large scrollback + rapid scrolling = cache misses
-- Tradeoff: history vs. scroll performance
```

### 15.4 Font Antialiasing
```lua
config.font_antialias = 'Subpixel'  -- or 'Greyscale'
config.font_hinting = 'Full'  -- or 'None', 'Vertical', 'VerticalSubpixel'
config.freetype_load_target = 'Normal'  -- or 'Light', 'Mono', 'HorizontalLcd'
```

---

## 16. Per-Window Configuration Overrides

### 16.1 Runtime Configuration Changes
```lua
window:set_config_overrides({
  window_background_opacity = 0.9,
  font_size = 14.0,
  color_scheme = 'Different Scheme',
})

-- Clear overrides
window:set_config_overrides({})
```

### 16.2 Avoiding Infinite Loops
```lua
wezterm.on('window-config-reloaded', function(window, pane)
  local overrides = window:get_config_overrides() or {}

  -- Only change if different to avoid loop
  local new_opacity = window:is_focused() and 1.0 or 0.8
  if overrides.window_background_opacity ~= new_opacity then
    overrides.window_background_opacity = new_opacity
    window:set_config_overrides(overrides)
  end
end)
```

### 16.3 Conditional Configuration
```lua
-- Different config based on hostname
if wezterm.hostname() == 'work-laptop' then
  config.color_scheme = 'Corporate Theme'
  config.font_size = 11.0
else
  config.color_scheme = 'Catppuccin Mocha'
  config.font_size = 13.0
end
```

---

## 17. Process & System Information

### 17.1 Process Detection (Local Only)
```lua
-- Get foreground process
local proc_info = pane:get_foreground_process_info()
if proc_info then
  wezterm.log_info('Process: ' .. proc_info.name)
  wezterm.log_info('PID: ' .. proc_info.pid)
  wezterm.log_info('CWD: ' .. proc_info.cwd)
end

-- Get just the name
local proc_name = pane:get_foreground_process_name()
```

**Limitations:**
- Only works for local panes (not SSH, not multiplexer)
- Platform-specific (Linux, macOS, Windows supported)
- Has performance overhead if overused

### 17.2 System Information
```lua
local hostname = wezterm.hostname()
local home = wezterm.home_dir

wezterm.on('gui-startup', function()
  local appearance = wezterm.gui.get_appearance()  -- 'Light' or 'Dark'
  local screens = wezterm.gui.screens()

  for _, screen in ipairs(screens) do
    wezterm.log_info('Screen: ' .. screen.width .. 'x' .. screen.height)
  end
end)
```

---

## 18. Comparison to Scarab's Fusabi Goals

### 18.1 What WezTerm Does Better

**1. Mature Event System:**
- 12+ predefined events with clear semantics
- Event handlers receive typed objects (window, pane, tab)
- No need to poll - reactive programming model

**2. Integrated Configuration + Scripting:**
- Single language (Lua) for both config and runtime
- No compilation step - immediate feedback
- Can call external programs, make HTTP requests, etc.

**3. Rich Standard Library:**
- Color manipulation, serialization, time/date
- Process info, URL parsing
- Plugin management built-in

**4. Mature UI Customization:**
- Tab bar, status bar, command palette, launcher
- All customizable via events
- Formatted text rendering (colors, fonts, icons)

**5. Powerful Multiplexing:**
- Multiple domain types (SSH, WSL, Unix, Serial)
- Workspace persistence across client crashes
- Programmatic control of layout

### 18.2 What Fusabi Could Do Better

**1. Type Safety:**
- Lua is dynamically typed
- Fusabi (F#) offers static typing, algebraic data types
- Compile-time guarantees vs. runtime errors

**2. Performance for Compute-Heavy Tasks:**
- F# compiles to bytecode (or native with AOT)
- Lua interprets (though LuaJIT is fast)
- For heavy data processing, compiled wins

**3. Functional Programming Paradigm:**
- F# first-class: pattern matching, immutability, monads
- Lua is imperative (can do functional, but not idiomatic)
- Better for complex state transformations

**4. Separation of Concerns:**
- Scarab: Daemon (.fzb compiled) + Client (.fsx interpreted)
- Clearer separation between performance-critical (daemon) and UI (client)
- WezTerm: Everything in one process

**5. Plugin Distribution:**
- WezTerm: Git repos only
- Fusabi: Could support package managers, versioned registries

### 18.3 Feature Parity Checklist

| Feature | WezTerm Lua | Scarab Fusabi Target | Notes |
|---------|-------------|----------------------|-------|
| Custom keybindings | Yes | Yes | Fusabi needs action registry |
| Leader key / modal | Yes | Yes | Implement key table stack |
| Event hooks | 12+ events | TBD | Define Scarab event system |
| UI customization | Tab/status bars | TBD | Need rendering hooks |
| Color scheme control | Full API | TBD | Runtime palette swapping |
| Font configuration | Advanced | Basic (cosmic-text) | Fusabi may not need full control |
| Plugin system | Git-based | TBD | Could use NuGet or custom |
| Multiplexing | SSH/WSL/Unix | Local only (v1) | Future: SSH via daemon |
| Programmatic pane control | Full API | TBD | Need daemon↔client protocol |
| Process detection | Local only | Local only | Same limitations |
| Mouse bindings | Yes | TBD | Lower priority |
| Quick select | Yes | TBD | Nice-to-have |
| Search | Yes | TBD | Lower priority |
| User variables | OSC 1337 | TBD | Can implement same |
| Command palette | Yes | TBD | Medium priority |
| Per-window overrides | Yes | TBD | Useful for focus effects |

### 18.4 Recommended Fusabi Feature Prioritization

**Phase 1: Core Scripting (6-day sprint compatible)**
1. Event system: `gui-startup`, `update-status`, `window-focus-changed`
2. Custom keybindings with action callbacks
3. Basic color scheme runtime switching
4. User variables (OSC 1337 support)

**Phase 2: UI Customization**
5. Status bar formatting (left/right)
6. Tab title formatting
7. Custom command palette entries
8. Toast notifications

**Phase 3: Advanced Control**
9. Programmatic pane/tab control (spawn, split, close)
10. Leader key / modal keybindings
11. Mouse bindings
12. Per-window configuration overrides

**Phase 4: Ecosystem**
13. Plugin system (NuGet or custom registry)
14. Hot reload for .fsx scripts
15. Debugger integration
16. Performance profiling tools

---

## 19. Key Insights for Scarab Development

### 19.1 Architecture Lessons

**1. Separation of State and Rendering:**
- WezTerm keeps terminal state in Rust, exposes to Lua via FFI
- Scarab: Keep grid state in daemon, expose to Fusabi via safe API
- Don't let scripts corrupt terminal state directly

**2. Event-Driven > Polling:**
- WezTerm emits events at key moments
- Scripts react rather than poll
- Better performance, clearer causality

**3. Typed Objects > Raw Data:**
- Lua receives `Window`, `Pane`, `Tab` objects with methods
- Not just JSON blobs
- Fusabi can leverage F# records with methods

**4. Config as Code:**
- Lua config is executable code (can run at startup)
- Fusabi: .fsx scripts can do the same
- Enables conditional config based on environment

### 19.2 API Design Patterns

**1. Builder Pattern for Config:**
```lua
local config = wezterm.config_builder()
config.font_size = 13
return config
```
Fusabi equivalent:
```fsharp
let config = Config.builder()
config.FontSize <- 13
config
```

**2. Action Registry:**
```lua
wezterm.action.SpawnTab 'CurrentPaneDomain'
wezterm.action_callback(fun)
```
Fusabi equivalent:
```fsharp
Action.spawnTab CurrentPaneDomain
Action.callback (fun window pane -> ...)
```

**3. Event Registration:**
```lua
wezterm.on('event-name', function(args) ... end)
```
Fusabi equivalent:
```fsharp
Wezterm.on "event-name" (fun args -> ...)
```

### 19.3 Performance Considerations

**1. Event Handler Performance:**
- WezTerm warns against slow handlers
- Especially `format-tab-title` (called on every render)
- Fusabi: Warn if handler exceeds threshold (e.g., 5ms)

**2. Caching:**
- WezTerm caches shaping, BIDI per line
- Scarab: Cache cosmic-text layouts similarly
- Invalidate on line update

**3. Async vs. Sync:**
- WezTerm: Most events are synchronous
- Allows async for heavy operations (e.g., HTTP requests)
- Fusabi: Leverage F# async for long-running tasks

### 19.4 UX Patterns

**1. Progressive Disclosure:**
- WezTerm has sane defaults
- Power users customize incrementally
- Scarab: Ship with good defaults, document customization

**2. Discoverability:**
- Command palette shows available actions
- `wezterm show-keys` lists keybindings
- Fusabi: Provide introspection tools

**3. Documentation:**
- WezTerm docs are comprehensive (200+ config options)
- Every function, event, and type documented
- Fusabi: Generate docs from F# XML comments

---

## 20. Summary & Recommendations

### 20.1 What Makes WezTerm's Lua System Powerful

1. **Tight Integration**: Lua is first-class, not bolted-on
2. **Rich API Surface**: 200+ config options + runtime control
3. **Event System**: Reactive, not polling-based
4. **UI Extensibility**: Tab bar, status bar, palette all scriptable
5. **Plugin Ecosystem**: Git-based, easy to share
6. **Cross-Platform**: Same API on Linux, macOS, Windows

### 20.2 Gaps Where Fusabi Can Excel

1. **Type Safety**: Static typing catches errors at compile time
2. **Functional Paradigm**: Pattern matching, immutability, ADTs
3. **Performance**: Compiled bytecode vs. interpreted
4. **Separation**: Daemon (compiled) vs. Client (interpreted)
5. **Ecosystem**: Could leverage NuGet, F# tooling

### 20.3 Recommended Fusabi Minimum Viable Feature Set

For Scarab to compete with WezTerm's Lua system:

**Must-Have (MVP):**
- [ ] Event system: 5+ core events (startup, status, focus, bell, user-var)
- [ ] Custom keybindings with F# callback support
- [ ] Runtime color scheme switching
- [ ] Status bar formatting (left/right)
- [ ] Basic pane control (spawn, split, send text)

**Should-Have (V2):**
- [ ] Tab title formatting
- [ ] Command palette augmentation
- [ ] Leader key / modal keybindings
- [ ] Font configuration via Fusabi
- [ ] Plugin system (git or NuGet)

**Nice-to-Have (V3):**
- [ ] Mouse bindings
- [ ] Quick select customization
- [ ] Per-window config overrides
- [ ] Advanced multiplexing (SSH domains)

### 20.4 Final Thoughts

WezTerm's Lua system is **the gold standard** for terminal emulator extensibility. It succeeds because:
- It's comprehensive (covers every aspect of the terminal)
- It's well-documented (every API is explained)
- It's performant (doesn't block the render loop)
- It's accessible (Lua is easy to learn)

Scarab's Fusabi system can differentiate by:
- Offering type safety for complex configs
- Leveraging F#'s functional paradigm
- Providing clear separation between daemon and client
- Integrating with .NET ecosystem tools

The key is **not to replicate WezTerm**, but to **learn from it** and build something that plays to Fusabi's strengths while covering the same core use cases.

---

## Sources

1. [Configuration - Wez's Terminal Emulator](https://wezterm.org/config/files.html)
2. [Full Config & Lua Reference - Wez's Terminal Emulator](https://wezterm.org/config/lua/general.html)
3. [Plugins - Wez's Terminal Emulator](https://wezterm.org/config/plugins.html)
4. [wezterm.action_callback - Wez's Terminal Emulator](https://wezterm.org/config/lua/wezterm/action_callback.html)
5. [wezterm.on - Wez's Terminal Emulator](https://wezterm.org/config/lua/wezterm/on.html)
6. [Key Binding - Wez's Terminal Emulator](https://wezterm.org/config/keys.html)
7. [augment-command-palette - Wez's Terminal Emulator](https://wezterm.org/config/lua/window-events/augment-command-palette.html)
8. [Colors & Appearance - Wez's Terminal Emulator](https://wezterm.org/config/appearance.html)
9. [get_builtin_schemes - Wez's Terminal Emulator](https://wezterm.org/config/lua/wezterm.color/get_builtin_schemes.html)
10. [Multiplexing - Wez's Terminal Emulator](https://wezterm.org/multiplexing.html)
11. [object: SshDomain - Wez's Terminal Emulator](https://wezterm.org/config/lua/SshDomain.html)
12. [ActivateCommandPalette - Wez's Terminal Emulator](https://wezterm.org/config/lua/keyassignment/ActivateCommandPalette.html)
13. [Leader Key - Commentary of Dotfiles](https://coralpink.github.io/commentary/wezterm/leader.html)
14. [GitHub - MLFlexer/modal.wezterm](https://github.com/MLFlexer/modal.wezterm)
15. [user-var-changed - Wez's Terminal Emulator](https://wezterm.org/config/lua/window-events/user-var-changed.html)
16. [Shell Integration - Wez's Terminal Emulator](https://wezterm.org/shell-integration.html)
17. [hyperlink_rules - Wez's Terminal Emulator](https://wezterm.org/config/lua/config/hyperlink_rules.html)
18. [Quick Select Mode - Wez's Terminal Emulator](https://wezterm.org/quickselect.html)
19. [inject_output - Wez's Terminal Emulator](https://wezterm.org/config/lua/pane/inject_output.html)
20. [toast_notification - Wez's Terminal Emulator](https://wezterm.org/config/lua/window/toast_notification.html)
21. [Workspaces / Sessions - Wez's Terminal Emulator](https://wezterm.org/recipes/workspaces.html)
22. [Font System | wezterm/wezterm | DeepWiki](https://deepwiki.com/wezterm/wezterm/3.3-font-system)
23. [Copy Mode - Wez's Terminal Emulator](https://wezterm.org/copymode.html)
24. [ShowLauncherArgs - Wez's Terminal Emulator](https://wezterm.org/config/lua/keyassignment/ShowLauncherArgs.html)
25. [Mouse Binding - Wez's Terminal Emulator](https://wezterm.org/config/mouse.html)
26. [get_foreground_process_info - Wez's Terminal Emulator](https://wezterm.org/config/lua/pane/get_foreground_process_info.html)
27. [set_config_overrides - Wez's Terminal Emulator](https://wezterm.org/config/lua/window/set_config_overrides.html)
28. [kde_window_background_blur - Wez's Terminal Emulator](https://wezterm.org/config/lua/config/kde_window_background_blur.html)
29. [macos_window_background_blur - Wez's Terminal Emulator](https://wezterm.org/config/lua/config/macos_window_background_blur.html)
30. [Change WezTerm blurred Window Background & Opacity on Focus | Roman Zipp](https://romanzipp.com/blog/how-to-toggle-wezterm-blurred-window-background-on-focus)
