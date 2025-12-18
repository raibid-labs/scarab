# Plugin Inspector UI Design

## Visual Mockup

```
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│  Plugin Inspector                                                                     ✖ Close│
├────────────────────────────────────────────────────────────────────────────────────────────┤
│  Total: 5    Enabled: 4    Failed: 1   │   Refresh   Clear Logs   Export Debug Info       │
├───────────────────────┬────────────────────────────────────────────────────────────────────┤
│ Plugins               │  Overview  Metadata  Hooks  Logs  Source                           │
│                       │                                                                     │
│ Filter: [______]      │  NavigationPlugin                                                   │
│                       │  Quick file navigation for terminal workflows                       │
│ ● NavigationPlugin    │                                                                     │
│   v1.0.0             │  ┌──────────────────────────────────────────────────────────────┐  │
│                       │  │ Status                                                         │  │
│ ● PalettePlugin       │  │                                                                │  │
│   v1.2.0             │  │  Enabled:              Yes                                     │  │
│                       │  │  Failure Count:        0                                       │  │
│ ● SessionPlugin       │  │  Total Executions:     1,247                                   │  │
│   v1.0.0             │  │  Total Execution Time: 342.56ms                                │  │
│                       │  │  Avg Execution Time:   0.275ms                                 │  │
│ ● AutoCompletePlugin  │  └──────────────────────────────────────────────────────────────┘  │
│   v0.9.1             │                                                                     │
│   Failures: 3         │  [ Disable ]  [ Reload ]                                           │
│                       │                                                                     │
│ ● GitIntegration      │                                                                     │
│   v2.1.0             │                                                                     │
│                       │                                                                     │
│                       │                                                                     │
│                       │                                                                     │
│                       │                                                                     │
│                       │                                                                     │
│                       │                                                                     │
└───────────────────────┴────────────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. Window Header
```
┌────────────────────────────────────────────────────────────┐
│  Plugin Inspector                                 ✖ Close  │
└────────────────────────────────────────────────────────────┘
```
- **Title**: "Plugin Inspector" in heading font
- **Close Button**: Standard X button, right-aligned
- **Draggable**: Entire header is drag handle for window positioning

### 2. Toolbar (Top Panel)
```
┌────────────────────────────────────────────────────────────────────────┐
│  Total: 5  │ Enabled: 4  │ Failed: 1  ║ Refresh  Clear Logs  Export   │
└────────────────────────────────────────────────────────────────────────┘
```

**Statistics Section** (Left):
- Total count: Gray text
- Enabled count: Green text
- Failed count: Red text (only shown if > 0)
- Vertical separators between stats

**Actions Section** (Right):
- Refresh button: Requests new plugin list
- Clear Logs button: Removes all log entries
- Export Debug Info button: Saves to file
- Close button: Far right, closes inspector

### 3. Sidebar (Left Panel - 300px default width)
```
┌─────────────────────┐
│ Plugins             │
│                     │
│ Filter: [______]    │
│                     │
│ ● PluginName       │ ← Status Dot + Name (selectable)
│   v1.0.0           │ ← Version (gray, small)
│   Failures: 3      │ ← Only if failed (red, small)
│                     │
│ ● Another Plugin   │
│   v2.0.0           │
│                     │
└─────────────────────┘
```

**Header**: "Plugins" heading

**Filter Input**:
- Text input field
- Placeholder: empty
- Live filtering as you type

**Plugin List**:
- Scrollable vertical list
- Each plugin shows:
  - Status indicator (colored dot):
    - 🟢 Green: Enabled, no failures
    - 🔴 Red: Enabled with failures
    - ⚫ Gray: Disabled
  - Plugin name (clickable/selectable)
  - Version number (small, gray)
  - Failure count (only if > 0, red)
- Selected plugin: Highlighted background
- Separator line between plugins

### 4. Tab Bar
```
┌────────────────────────────────────────────────────────┐
│  [Overview]  Metadata  Hooks  Logs  Source            │
└────────────────────────────────────────────────────────┘
```
- Five tabs across the top
- Selected tab: Underlined or highlighted
- Clicking switches content below

### 5. Content Area - Overview Tab
```
┌──────────────────────────────────────────────────────────────┐
│  NavigationPlugin                                            │
│  Quick file navigation for terminal workflows                │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Status                                                   │ │
│  │                                                          │ │
│  │  Enabled:              Yes                              │ │
│  │  Failure Count:        0                                │ │
│  │  Total Executions:     1,247                            │ │
│  │  Total Execution Time: 342.56ms                         │ │
│  │  Avg Execution Time:   0.275ms                          │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Last Error                                              │ │
│  │ Failed to parse configuration: invalid TOML             │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  [ Disable ]  [ Reload ]                                    │
└──────────────────────────────────────────────────────────────┘
```

**Plugin Header**:
- Name in large heading font
- Description below in normal font

**Status Card** (Dark background):
- Heading: "Status"
- Grid layout, two columns:
  - Label (left): Property name
  - Value (right): Actual value
- Color coding:
  - Enabled Yes: Green
  - Enabled No: Red
  - Failure Count 0: Green
  - Failure Count > 0: Red

**Error Card** (Red-tinted background, only if error exists):
- Heading: "Last Error"
- Error message in red text
- Monospace font for error details

**Action Buttons**:
- Disable/Enable: Toggles based on current state
- Reload: Always available

### 6. Content Area - Metadata Tab
```
┌──────────────────────────────────────────────────────────────┐
│  Name:                NavigationPlugin                       │
│  Version:             1.0.0                                  │
│  Description:         Quick file navigation for terminal...  │
│  Author:              Jane Developer                         │
│  Homepage:            https://github.com/user/plugin         │
│  API Version:         0.1.0                                  │
│  Min Scarab Version:  0.1.0                                  │
└──────────────────────────────────────────────────────────────┘
```

**Grid Layout**:
- Two columns, striped rows
- Labels (left): Bold text
- Values (right): Normal text
- Homepage: Clickable hyperlink (blue, underlined)

### 7. Content Area - Hooks Tab
```
┌──────────────────────────────────────────────────────────────┐
│  Hook Execution History                                      │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Total: 1247  │ Success: 1244  │ Failed: 3  │ Avg: 0.3ms│ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ✓ 2.3s ago   on_output    0.245ms                          │
│  ✓ 2.7s ago   on_input     0.189ms                          │
│  ✓ 3.1s ago   on_output    0.312ms                          │
│  ✖ 5.4s ago   on_output    1.203ms  Parse error             │
│  ✓ 5.8s ago   on_input     0.156ms                          │
│  ...                                                         │
└──────────────────────────────────────────────────────────────┘
```

**Statistics Card** (Dark background):
- Summary stats in horizontal layout
- Total, Success (green), Failed (red), Average time

**Hook List** (Scrollable):
- Each hook execution shows:
  - Success indicator: ✓ (green) or ✖ (red)
  - Relative timestamp: "X.Xs ago"
  - Hook type: on_output, on_input, etc.
  - Duration: in milliseconds
  - Error message (if failed): Red text
- Most recent at top (reverse chronological)
- Limited to last 100 for performance

### 8. Content Area - Logs Tab
```
┌──────────────────────────────────────────────────────────────┐
│  Filter: [______]  ☑ Auto-scroll  [ Clear ]                 │
│  ──────────────────────────────────────────────────────────  │
│  ℹ 0.5s  NavigationPlugin  Scanning directory /home/user    │
│  ▸ 1.2s  NavigationPlugin  Found 42 files                   │
│  ⚠ 2.1s  PalettePlugin     Color scheme not found           │
│  ✖ 3.4s  AutoComplete      Failed to load dictionary        │
│  ℹ 4.2s                    Plugin Inspector opened           │
│  ...                                                         │
└──────────────────────────────────────────────────────────────┘
```

**Controls Row**:
- Filter input: Search logs
- Auto-scroll checkbox: Stick to bottom
- Clear button: Remove all logs

**Log Entries** (Scrollable):
- Each log shows:
  - Level icon: ◦ ▸ ℹ ⚠ ✖ (color-coded)
  - Relative timestamp: "Xs ago"
  - Plugin name: Blue text (if from plugin)
  - Message: White text
- Color coding by level:
  - Trace: Dark gray
  - Debug: Light blue
  - Info: White
  - Warn: Yellow
  - Error: Red
- Auto-scroll when enabled
- Monospace font for technical messages

### 9. Content Area - Source Tab
```
┌──────────────────────────────────────────────────────────────┐
│  Plugin Source                                               │
│                                                              │
│  Source code viewing not yet implemented                     │
│                                                              │
│  In a future version, this tab will display:                │
│  - Plugin source code (.fsx files)                           │
│  - Compiled bytecode information (.fzb files)                │
│  - Plugin configuration                                      │
│  - Dependency tree                                           │
│                                                              │
│  Plugin: NavigationPlugin                                    │
└──────────────────────────────────────────────────────────────┘
```

**Placeholder Content**:
- Title: "Plugin Source"
- Gray text explaining future features
- List of planned capabilities
- Current plugin name for context

## Color Palette

### Background Colors
- **Window Background**: `#1a1a1a` (Very dark gray)
- **Panel Background**: `#2a2a2a` (Dark gray)
- **Card Background**: `#1e1e28` (Dark blue-tinted gray)
- **Error Card**: `#3c1414` (Dark red)
- **Selected Item**: `#3a3a4a` (Medium gray with purple tint)

### Text Colors
- **Primary Text**: `#ffffff` (White)
- **Secondary Text**: `#a0a0a0` (Light gray)
- **Heading Text**: `#ffffff` (White, bold)
- **Link Text**: `#6496ff` (Light blue)

### Status Colors
- **Success/Green**: `#64ff64` (Bright green)
- **Error/Red**: `#ff5050` (Bright red)
- **Warning/Yellow**: `#ffc800` (Bright yellow/amber)
- **Info/Blue**: `#9696ff` (Light blue)
- **Disabled/Gray**: `#808080` (Medium gray)

### UI Element Colors
- **Button Background**: `#3a3a4a`
- **Button Hover**: `#4a4a5a`
- **Button Active**: `#2a2a3a`
- **Input Background**: `#2a2a2a`
- **Input Border**: `#4a4a4a`
- **Separator**: `#3a3a3a`

## Typography

### Font Families
- **UI Text**: System sans-serif (e.g., Inter, Roboto, Segoe UI)
- **Code/Data**: Monospace (e.g., Fira Code, JetBrains Mono, Consolas)

### Font Sizes
- **Window Title**: 18px, bold
- **Tab Labels**: 14px, medium
- **Heading**: 20px, bold
- **Subheading**: 16px, semibold
- **Body Text**: 14px, regular
- **Small Text**: 12px, regular
- **Tiny Text**: 10px, regular

### Line Heights
- **Headings**: 1.2
- **Body Text**: 1.5
- **Code**: 1.4

## Spacing & Layout

### Padding
- **Window**: 0px (handled by egui)
- **Panels**: 8px
- **Cards**: 12px
- **Buttons**: 6px vertical, 12px horizontal

### Margins
- **Between Cards**: 10px
- **Between Sections**: 16px
- **Between Elements**: 8px

### Sizing
- **Sidebar Width**: 300px (resizable, min 200px, max 500px)
- **Toolbar Height**: 40px (fixed)
- **Tab Bar Height**: 32px (fixed)
- **Window Min Size**: 800x600px
- **Window Default**: 1000x700px

## Interaction States

### Buttons
- **Default**: Medium gray background, white text
- **Hover**: Lighter background, white text
- **Active**: Darker background, white text
- **Disabled**: Dark background, gray text

### Selectable Items
- **Default**: Transparent background
- **Hover**: Slight highlight
- **Selected**: Colored background, white text

### Input Fields
- **Default**: Dark background, light border
- **Focus**: Brighter border, white text
- **Error**: Red border

## Accessibility

### Keyboard Navigation
- **Tab**: Move between focusable elements
- **Enter**: Activate buttons
- **Escape**: Close window
- **Ctrl+Shift+P**: Toggle inspector
- **Arrow Keys**: Navigate lists

### Screen Readers
- All buttons have aria-labels
- Status indicators have text equivalents
- Lists are properly structured

### Color Contrast
- All text meets WCAG AA standards (4.5:1 ratio)
- Interactive elements are distinguishable
- Status colors are not sole indicators (icons too)

## Responsive Behavior

### Window Resizing
- **Minimum Size**: 800x600px
- **Sidebar**: Maintains width unless window too small
- **Content**: Wraps and scrolls as needed
- **Toolbar**: Items compress or wrap

### Scrolling
- **Plugin List**: Vertical scroll, visible scrollbar
- **Content Tabs**: Vertical scroll within tab
- **Logs**: Auto-scroll option, stick to bottom
- **Hooks**: Reverse chronological, scroll to see older

### Empty States
- **No Plugins**: "No plugins found" gray text
- **No Logs**: "No logs" gray text
- **No Hooks**: "No hook executions recorded" gray text
- **No Error**: Error card not shown

## Animation & Transitions

### Minimal Animations
- **Window Open**: Instant (no fade)
- **Tab Switch**: Instant content swap
- **Button Hover**: 100ms color transition
- **Selection**: 100ms background transition
- **Scroll**: Smooth (handled by egui)

### Performance
- No unnecessary animations
- GPU-accelerated where possible
- Target: 60fps UI rendering

## Implementation Notes

### egui Widgets Used
- `Window`: Top-level container
- `TopBottomPanel`: Toolbar
- `SidePanel`: Plugin list sidebar
- `CentralPanel`: Content area
- `ScrollArea`: All scrollable content
- `Grid`: Metadata and status displays
- `Frame`: Card backgrounds
- `Button`: All clickable actions
- `Label`: Text display
- `Hyperlink`: External links
- `TextEdit`: Filter inputs
- `Checkbox`: Auto-scroll option

### Performance Optimizations
- Only render when visible
- Limit hook history to 500 entries
- Limit logs to 1000 entries
- Efficient string filtering
- Lazy rendering of off-screen content

### Testing Checklist
- [ ] Window opens/closes correctly
- [ ] All tabs display proper content
- [ ] Plugin list filters work
- [ ] Enable/disable buttons work
- [ ] Reload button works
- [ ] Logs auto-scroll works
- [ ] Export creates file
- [ ] Keyboard shortcuts work
- [ ] Resizing works correctly
- [ ] Scrolling is smooth
- [ ] Color-coding is correct
- [ ] Error states display properly
