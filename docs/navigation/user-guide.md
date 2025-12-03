# Navigation User Guide

Quick keyboard-driven navigation for Scarab terminal emulator.

## Overview

Scarab's navigation system lets you quickly jump to links, file paths, and other clickable elements without using the mouse. Similar to Vimium for browsers.

## Getting Started

### Entering Hint Mode
Press `f` (default) to enter hint mode. Yellow labels will appear on all clickable elements.

### Selecting a Target
Type the characters shown on the hint label (e.g., "a", "s", "df"). The target will be highlighted.

### Executing Actions
- Press **Enter** to activate (open link, navigate to file)
- Press **y** to yank (copy) the target
- Press **Escape** to cancel

## Navigation Modes

### Normal Mode
Default mode. Navigation keys trigger hint mode or other actions.

### Hint Mode
Labels visible on focusable elements. Type to filter and select.

### Insert Mode
Pass-through mode - all keys go to the terminal. Press Escape to exit.

## Default Keybindings

| Key | Action |
|-----|--------|
| `f` | Enter hint mode |
| `F` | Enter hint mode (new tab) |
| `Escape` | Exit hint mode |
| `y` | Yank (copy) selected target |
| `Enter` | Activate selected target |
| `Tab` | Cycle through hints |
| `j/k` | Navigate up/down in lists |

## Keymap Styles

### Vimium Style (Default)
- `f` for hints
- Single character sequences
- Familiar to Vimium users

### Cosmos Style
- `Space` followed by `n` for navigation prefix
- More key combinations available

### Spacemacs Style
- `SPC n` prefix for navigation commands
- Modal, Emacs-inspired

## Focusable Elements

The navigation system automatically detects:
- **URLs**: http://, https://, file://
- **File Paths**: /path/to/file, ./relative/path
- **Email Addresses**: user@example.com
- **Prompt Markers**: Shell prompts (OSC 133)
- **Plugin Content**: Custom elements from plugins

## Per-Pane Navigation

Each terminal pane has independent navigation state:
- Hints don't leak between panes
- Focus position remembered when switching panes
- Navigation mode reset when switching panes

## Tips & Tricks

### Quick Selection
For common targets, the first few characters are usually unique. Type quickly for faster navigation.

### Escaping
Press Escape multiple times to ensure you're back in Normal mode.

### Copying Multiple Items
Enter hint mode, yank first item, re-enter hint mode, yank next item. Each yank adds to clipboard.

### Large Screens
With many focusables, hints use 2-3 character sequences. Type the full sequence shown.

## Troubleshooting

### Hints Not Appearing
- Check that hint mode is enabled in config
- Ensure the pane has focusable content
- Try scrolling - focusables must be visible

### Wrong Element Selected
- Type the complete hint sequence
- Wait for the correct element to highlight before pressing Enter

### Keys Not Working
- Ensure you're not in Insert mode
- Check your keymap style configuration
- Some terminals capture certain keys

## Configuration

See the [Configuration Guide](../config.md) for:
- Changing keymap style
- Customizing hint characters
- Adjusting hint appearance
- Setting plugin capabilities
