# Scarab UI Implementation Plan

This document outlines the implementation plan for the next generation of scarab UI features, inspired by wezterm's architecture and designed to showcase the power of our Bevy-based terminal emulator.

## Table of Contents

1. [Unified Command Palette / Omnibar](#1-unified-command-palette--omnibar)
2. [Wezterm Feature Port](#2-wezterm-feature-port)
3. [File Breadcrumb Plugin](#3-file-breadcrumb-plugin)
4. [Bevy Showcase Features](#4-bevy-showcase-features)

---

## 1. Unified Command Palette / Omnibar

### Vision

A single, unified omnibar that handles **all search and navigation** in scarab - combining the functionality of VS Code's `Ctrl+P` (files) and `Ctrl+Shift+P` (commands), plus shell history, sessions, and plugin data.

### Current State

- `command_palette.rs` exists with:
  - Fuzzy search via `SkimMatcherV2`
  - Local command registry (6 terminal commands)
  - Remote modal support from daemon
  - Basic keybindings (Ctrl+P toggle, arrows, enter)

### Architecture: Provider-Based Design

```
┌─────────────────────────────────────────────────────────────────┐
│                        Omnibar UI                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ >  search query here...                            [Esc] │    │
│  ├─────────────────────────────────────────────────────────┤    │
│  │ [a] src/main.rs                          Files          │    │
│  │ [s] cargo build                          History        │    │
│  │ [d] > Clear Terminal                     Commands       │    │
│  │ [f] Session: nvim@dev                    Sessions       │    │
│  │ [g] phage: Scan Context                  Plugins        │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Providers

Each provider implements a common trait:

```rust
pub trait OmnibarProvider: Send + Sync {
    /// Provider identifier (e.g., "files", "history", "commands")
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Icon/emoji for this provider
    fn icon(&self) -> &str;

    /// Prefix that activates this provider exclusively (e.g., ">" for commands)
    fn prefix(&self) -> Option<&str>;

    /// Query this provider for results
    fn query(&self, query: &str, limit: usize) -> Vec<OmnibarResult>;

    /// Execute a selected result
    fn execute(&self, result: &OmnibarResult, ctx: &mut OmnibarContext);

    /// Priority for result ranking (higher = shown first)
    fn priority(&self) -> i32;
}
```

### Built-in Providers

| Provider | Prefix | Description | Data Source |
|----------|--------|-------------|-------------|
| **Commands** | `>` | Terminal commands, plugin actions | Local registry + daemon |
| **Files** | none | Files in cwd and project | `walkdir` + git ignore |
| **History** | `#` | Shell command history | Atuin integration or shell history file |
| **Sessions** | `@` | Panes across all clients | Daemon session list |
| **Plugins** | `:` | Plugin-provided results | Plugin IPC |
| **Recent** | `~` | Recently opened files/dirs | Local storage |

### Provider: Files

```rust
pub struct FilesProvider {
    /// Max depth to search
    max_depth: usize,
    /// Respect .gitignore
    use_gitignore: bool,
    /// File patterns to include
    include_patterns: Vec<glob::Pattern>,
}

impl FilesProvider {
    fn query(&self, query: &str, limit: usize) -> Vec<OmnibarResult> {
        // Use ignore crate (respects .gitignore)
        // Fuzzy match with SkimMatcherV2
        // Return with file icons based on extension
    }
}
```

### Provider: History (Atuin Integration)

```rust
pub struct HistoryProvider {
    /// Path to shell history or atuin socket
    source: HistorySource,
}

enum HistorySource {
    Atuin { socket: PathBuf },
    ShellHistory { path: PathBuf, shell: ShellType },
}
```

### Provider: Sessions (Cross-Client)

```rust
pub struct SessionsProvider;

impl SessionsProvider {
    fn query(&self, query: &str, limit: usize) -> Vec<OmnibarResult> {
        // Query daemon for all sessions across all clients
        // Format: "pane_title@client_name [tab_name]"
        // Execute: Focus that pane (may require client IPC)
    }
}
```

### UI Components

```rust
#[derive(Resource)]
pub struct OmnibarState {
    /// Is omnibar visible?
    pub active: bool,
    /// Current search query
    pub query: String,
    /// Active prefix (None = search all)
    pub active_prefix: Option<String>,
    /// Aggregated results from all providers
    pub results: Vec<OmnibarResult>,
    /// Selected index
    pub selected: usize,
    /// Hint mode active?
    pub hint_mode: bool,
    /// Current hint input
    pub hint_input: String,
}

#[derive(Clone)]
pub struct OmnibarResult {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: String,
    pub provider_id: String,
    pub score: i64,
    pub data: serde_json::Value,
}
```

### Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+P` | Open omnibar (all providers) |
| `Ctrl+Shift+P` | Open omnibar with `>` prefix (commands only) |
| `Ctrl+R` | Open omnibar with `#` prefix (history) |
| `Ctrl+O` | Open omnibar with no prefix (files) |
| `@` | When typed first, switch to sessions mode |
| `:` | When typed first, switch to plugins mode |
| `Esc` | Close omnibar or clear prefix |
| `Up/Down` | Navigate results |
| `Enter` | Execute selected |
| `Tab` | Preview (for files) |
| `a-l` | Hint mode (when many results) |

### Implementation Phases

**Phase 1: Core Omnibar (2 days)**
- Refactor `command_palette.rs` → `omnibar.rs`
- Implement provider trait
- Commands provider (migrate existing)
- Basic UI with fuzzy search

**Phase 2: File Provider (1 day)**
- Add `ignore` crate dependency
- Implement FilesProvider with gitignore support
- File icons based on extension

**Phase 3: History & Sessions (1 day)**
- Shell history provider (bash/zsh/fish)
- Optional Atuin integration
- Sessions provider via daemon IPC

**Phase 4: Plugin Provider (1 day)**
- Protocol for plugins to register omnibar results
- Scryforge integration example

**Phase 5: Polish (1 day)**
- Hint mode for results
- Preview pane for files
- Persistence of recent selections

---

## 2. Wezterm Feature Port

### Features to Port from Wezterm Config

Based on analysis of `~/.config/wezterm/`:

### 2.1 Modal System

Wezterm has 6 modes: copy, search, font, window, help, pick. Scarab should have similar:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScarabMode {
    Normal,
    Copy,      // vim-like text selection
    Search,    // pattern search in terminal
    Window,    // pane management (split, resize, navigate)
    Font,      // font size adjustment
    Pick,      // pickers (colorscheme, font, etc.)
    Hint,      // nav hint mode (already exists)
}
```

**Mode Indicator in Status Bar:**
```
┌─────────────────────────────────────────────────────────────┐
│ COPY │ h/j/k/l: move  y: copy  v: select  Esc: exit        │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Responsive Status Bar

Wezterm's status bar adapts to available width. Implement similar:

```rust
/// Content variants for different widths
pub struct ResponsiveContent {
    pub full: String,      // "~/raibid-labs/scarab"
    pub medium: String,    // "~/r/scarab"
    pub short: String,     // "scarab"
    pub minimal: String,   // ""
}

impl StatusBarRight {
    fn render(&self, available_width: f32) -> Vec<RenderItem> {
        // Calculate what fits
        // CWD → Hostname → Time → Battery (in priority order)
        // Show as much as fits
    }
}
```

### 2.3 Pickers

Port wezterm's picker system for:
- **Colorscheme picker**: List available themes, preview, apply
- **Font picker**: List system fonts, preview, apply
- **Font size picker**: Quick adjustment

```rust
pub struct PickerState {
    pub active: bool,
    pub title: String,
    pub items: Vec<PickerItem>,
    pub selected: usize,
    pub fuzzy_query: String,
}

pub struct PickerItem {
    pub id: String,
    pub label: String,
    pub preview: Option<Box<dyn Fn() -> Preview>>,
}
```

### 2.4 Tab Bar Enhancements

From wezterm's `format-tab-title.lua`:
- Unseen output indicator per tab
- Tab index numbers (with notification override)
- Powerline-style separators
- Hover state styling

```rust
pub struct TabDisplay {
    pub index: usize,
    pub title: String,
    pub has_unseen_output: bool,
    pub is_active: bool,
    pub is_hovered: bool,
}

fn render_tab(tab: &TabDisplay) -> Vec<RenderItem> {
    vec![
        // Left separator (powerline)
        RenderItem::Text(if tab.index == 0 { "" } else { "" }.into()),
        // Index or notification icon
        RenderItem::Text(if tab.has_unseen_output {
            "".into()  // notification bell
        } else {
            format!("{} ", tab.index + 1)
        }),
        // Title
        RenderItem::Text(truncate(&tab.title, 20)),
        // Right separator
        RenderItem::Text("".into()),
    ]
}
```

### 2.5 Leader Key System

Wezterm uses `<leader>` prefix for mode switching. Port this:

```rust
pub struct LeaderKeyState {
    pub active: bool,
    pub timeout: Timer,
    pub key: KeyCode,  // e.g., Space
}

// Leader key bindings
// <leader>c → Copy mode
// <leader>s → Search mode
// <leader>w → Window mode
// <leader>f → Font mode
// <leader>p → Pick mode
```

---

## 3. File Breadcrumb Plugin

### Vision

A breadcrumb bar at the top showing current directory path, where each segment is hintable and clicking/selecting opens a directory picker.

```
┌─────────────────────────────────────────────────────────────┐
│ [a]~ / [s]raibid-labs / [d]scarab / [f]crates / [g]client  │
├─────────────────────────────────────────────────────────────┤
│                     Terminal Content                         │
```

### Research Summary: File Browser Options

Based on research, the recommended approach is **ratatui-explorer** as an embeddable library:

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **ratatui-explorer** | Embeddable, lightweight, MIT | Need rendering bridge | **RECOMMENDED** |
| **broot subprocess** | Full-featured | Subprocess overhead | Use for "launch external" |
| **yazi subprocess** | Modern, async | Not embeddable, Lua plugins | Use for "launch external" |
| **Custom in Fusabi** | Perfect integration | Significant work | Future enhancement |

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    BreadcrumbPlugin                          │
├─────────────────────────────────────────────────────────────┤
│ BreadcrumbState                                              │
│   - current_path: PathBuf                                    │
│   - segments: Vec<PathSegment>                               │
│   - picker_active: bool                                      │
│   - picker_target: Option<PathBuf>                          │
├─────────────────────────────────────────────────────────────┤
│ Systems:                                                     │
│   - track_cwd() - watch terminal PWD changes                │
│   - render_breadcrumbs() - draw path segments               │
│   - handle_segment_select() - open picker for segment       │
│   - render_picker() - show directory contents               │
│   - execute_navigation() - cd to selected path              │
│   - execute_file_open() - open file in $EDITOR              │
└─────────────────────────────────────────────────────────────┘
```

### Breadcrumb Segment

```rust
pub struct PathSegment {
    pub name: String,
    pub full_path: PathBuf,
    pub hint_key: String,
    pub is_home: bool,
}
```

### Directory Picker

When a breadcrumb segment is selected, show a hinted picker:

```
┌─────────────────────────────────────────────────────────────┐
│ ~/raibid-labs/scarab/                                        │
├─────────────────────────────────────────────────────────────┤
│ [a]  crates/                                                │
│ [s]  docs/                                                  │
│ [d]  plugins/                                               │
│ [f]  examples/                                              │
│ [g]  Cargo.toml                                             │
│ [h]  CLAUDE.md                                              │
│ [j]  README.md                                              │
└─────────────────────────────────────────────────────────────┘
```

### File/Directory Actions

| Entry Type | Action on Select |
|------------|------------------|
| Directory | `cd` to that directory, update breadcrumb |
| File | Open in `$EDITOR` (nvim, code, etc.) |
| Executable | Option to run or edit |

### PWD Tracking

Track current directory via:
1. **OSC 7** escape sequence (terminal reports PWD)
2. **Shell integration** (PROMPT_COMMAND hook)
3. **Polling** `/proc/{pid}/cwd` (Linux fallback)

```rust
fn track_cwd(
    mut state: ResMut<BreadcrumbState>,
    terminal: Res<TerminalState>,
) {
    // Check for OSC 7 in terminal output
    if let Some(new_cwd) = terminal.last_osc7_path() {
        if new_cwd != state.current_path {
            state.current_path = new_cwd.clone();
            state.segments = path_to_segments(&new_cwd);
        }
    }
}
```

### Implementation Phases

**Phase 1: Basic Breadcrumb Display (1 day)**
- Parse current path into segments
- Render as hintable text
- Track PWD via OSC 7

**Phase 2: Directory Picker (1 day)**
- Show directory contents on segment select
- Hint keys for entries
- Navigate with keyboard

**Phase 3: Actions (0.5 days)**
- `cd` for directories
- `$EDITOR` for files

**Phase 4: ratatui-explorer Integration (1.5 days)**
- Create rendering bridge
- Advanced file browser mode (`Ctrl+O`)

---

## 4. Bevy Showcase Features

### Vision

Demonstrate the unique capabilities of having a Bevy-powered terminal emulator - things that would be impossible or very difficult in traditional terminals.

### 4.1 Visual Polish

#### Animated Tab Transitions

```rust
pub struct TabTransition {
    pub from_index: usize,
    pub to_index: usize,
    pub progress: f32,  // 0.0 to 1.0
    pub duration: f32,  // seconds
    pub easing: EasingFunction,
}

fn animate_tab_switch(
    mut transitions: Query<&mut TabTransition>,
    time: Res<Time>,
) {
    for mut transition in transitions.iter_mut() {
        transition.progress += time.delta_secs() / transition.duration;
        if transition.progress >= 1.0 {
            // Complete transition
        }
        // Apply easing, render intermediate state
    }
}
```

**Effects:**
- Slide animation between tabs
- Fade in/out for new/closing tabs
- Scale animation on hover

#### Glow Effects (Already Partially Implemented)

Enhance existing `shaders/glow.rs`:
- Active element glow (focused pane, active input)
- Notification pulse
- Mode indicator glow (different colors per mode)

#### Particle Effects

- Command execution sparkles
- Error notification shake
- Success confetti (optional, configurable)

### 4.2 Data Visualization: Dashboard Pane

A special pane type that renders dashboards instead of terminal output:

```rust
pub enum PaneContent {
    Terminal(TerminalState),
    Dashboard(DashboardState),
    Graph(GraphState),
}

pub struct DashboardState {
    pub widgets: Vec<DashboardWidget>,
    pub layout: DashboardLayout,
    pub refresh_rate: Duration,
}

pub enum DashboardWidget {
    LineChart { data: Vec<f32>, label: String },
    BarChart { data: Vec<(String, f32)> },
    Gauge { value: f32, max: f32, label: String },
    Text { content: String, style: TextStyle },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
}
```

**Use Cases:**
- System monitor (CPU, memory, disk)
- Git status dashboard
- Build progress visualization
- Log stream with filtering

### 4.3 Graph/DAG Rendering

Render directed acyclic graphs for:
- Dependency visualization (cargo tree)
- Git commit graph
- Task dependency graphs

```rust
pub struct GraphState {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub layout: GraphLayout,
}

pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub position: Vec2,
    pub color: Color,
    pub shape: NodeShape,
}

pub enum NodeShape {
    Circle,
    Rectangle,
    Diamond,
    Hexagon,
}
```

**Rendering Options:**
- 2D node-link diagram (default)
- 3D voxel representation (experimental)
- Force-directed layout with animation

### 4.4 3D Terminal Theme (Experimental)

Use Bevy's 3D capabilities for depth effects:

```rust
pub struct Terminal3DConfig {
    pub depth_enabled: bool,
    pub layer_separation: f32,
    pub perspective_amount: f32,
    pub lighting: LightingConfig,
}
```

**Effects:**
- Panes at different Z depths
- Parallax scrolling
- Ambient occlusion between panes
- Subtle shadows

### 4.5 Image/Media Rendering

Leverage Bevy's image handling:

```rust
pub enum InlineMedia {
    Image { path: PathBuf, size: Vec2 },
    Gif { path: PathBuf, playing: bool },
    Video { path: PathBuf, state: VideoState },  // Future
}
```

**Protocols:**
- Kitty graphics protocol
- Sixel (via conversion)
- iTerm2 inline images

### Implementation Priority

| Feature | Effort | Impact | Priority |
|---------|--------|--------|----------|
| Tab transitions | Low | Medium | P1 |
| Glow enhancements | Low | High | P1 |
| Dashboard pane | Medium | High | P2 |
| Graph rendering | Medium | Medium | P2 |
| 3D effects | High | Low (gimmick) | P3 |
| Particle effects | Low | Low | P3 |
| Inline images | Medium | High | P2 |

### Research Findings

**Bevy UI Animation (Bevy 0.15+):**
- New `Curve` trait with cyclic splines and common easing functions
- `AnimationClip` can animate component fields with arbitrary curves
- Animation graphs support blending multiple animations
- Box shadows now configurable on UI nodes
- Layout powered by `taffy` library
- See: [Bevy 0.15 Release Notes](https://bevy.org/news/bevy-0-15/), [Animation Events Discussion](https://thisweekinbevy.com/issue/2024-10-14-animation-events-curves-and-nostd)

**Bevy Charts/Visualization:**
- No dedicated charting crate for Bevy currently
- Best approach: Use `bevy_egui` + `egui_plot` for charts
- `bevy_metrics_dashboard` exists for metrics visualization
- For custom graphs: Build with Bevy 2D rendering primitives
- See: [Bevy Metrics Discussion](https://github.com/bevyengine/bevy/discussions/11738)

**Implementation Approach for Animations:**
```rust
// Use Bevy 0.15's Curve API for smooth transitions
use bevy::animation::{AnimationClip, AnimationPlayer};
use bevy::math::curve::{Curve, EasingCurve, EaseFunction};

// Tab transition example
let ease_in_out = EasingCurve::new(0.0, 1.0, EaseFunction::CubicInOut);
```

**Graph Layout:**
- `petgraph` crate for graph data structures
- `layout-rs` or custom force-directed implementation
- Sugiyama algorithm for DAG layering

---

## Summary: Implementation Roadmap

### Sprint 1: Unified Omnibar (6 days)
- Core omnibar with provider system
- Commands, Files, History providers
- Sessions provider (cross-client)
- Plugin provider protocol

### Sprint 2: Wezterm Features (6 days)
- Modal system (copy, search, window, font, pick)
- Responsive status bar
- Pickers (colorscheme, font)
- Tab bar enhancements
- Leader key system

### Sprint 3: File Breadcrumb (4 days)
- Breadcrumb bar with hints
- Directory picker overlay
- PWD tracking
- File open in $EDITOR

### Sprint 4: Bevy Showcase (6 days)
- Tab animations
- Glow enhancements
- Dashboard pane prototype
- Inline image support (Kitty protocol)

### Sprint 5: Polish & Integration (4 days)
- Cross-feature integration testing
- Performance optimization
- Documentation
- Example plugins

---

## Dependencies to Add

```toml
# Cargo.toml additions
ignore = "0.4"           # .gitignore-aware file walking
skim = "0.10"            # Fuzzy matching (already have via fuzzy-matcher)
ratatui-explorer = "0.2" # File browser widget (for advanced mode)
walkdir = "2.5"          # Directory traversal
notify = "6.1"           # File system watching
egui_plot = "0.29"       # Charting (via bevy_egui if needed)
```

---

## Questions for Discussion

1. **Atuin Integration**: Should we require Atuin for history, or support both Atuin and raw shell history files?

2. **Cross-Client Sessions**: How should we handle focusing a pane in a different client? (Raise window, IPC, just show info?)

3. **Dashboard Data Sources**: Should dashboards pull data from shell commands, plugins, or both?

4. **3D Effects**: Worth the complexity for "wow factor", or keep things 2D and fast?

5. **File Preview**: In omnibar file mode, should Tab show a preview pane? What about syntax highlighting?
