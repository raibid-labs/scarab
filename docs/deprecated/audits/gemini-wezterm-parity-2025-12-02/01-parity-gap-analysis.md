# WezTerm Parity Gap Analysis
**Date:** December 2, 2025
**Auditor:** Gemini Agent

## Executive Summary
Scarab has made significant progress towards a usable MVP, with robust performance and basic terminal features. However, strictly comparing it to **WezTerm**, there is a significant architectural gap in **Multiplexing (Tabs & Panes)**. While WezTerm functions as a full window manager for terminal sessions, Scarab currently operates on a 1-Session-to-1-PTY model.

## Detailed Feature Comparison

| Feature Category | WezTerm State | Scarab State | Gap Severity |
|-----------------|---------------|--------------|--------------|
| **Configuration** | **Lua**: Full scripting capability, event hooks, dynamic config generation. | **Fusabi (.fsx)**: Implemented but extraction is partial. API surface is small compared to WezTerm's Lua API. | 🟡 Medium |
| **Multiplexing** | **Native**: Tabs, Splits (Panes), specialized layouts, persistent sessions across windows. | **None**: 1 Window = 1 Session = 1 PTY. No concept of splits or tabs within the daemon logic. | 🔴 Critical |
| **Scrollback** | **Robust**: Search, copy mode, semantic zones. | **Implemented**: 10k lines, search (regex), mouse scroll. Feature parity is close for MVP. | 🟢 Low |
| **Images** | **iTerm2, Sixel, Kitty**: Fully supported. | **Partial**: iTerm2 parser exists in Daemon, but *rendering pipeline* to Client appears missing. | 🔴 High |
| **Ligatures** | **Supported**: Harfbuzz shaping. | **Missing**: `cosmic-text` supports it, but needs verification/enabling in Scarab. | 🟡 Medium |
| **SSH/Domains** | **Built-in**: SSH client, TLS multiplexing over domains. | **None**: Relies on external `ssh` command. | ⚪ Future |
| **Shell Integration**| **OSC 133** shell integration, semantic prompts. | **Basic**: Some VTE support, but deep semantic integration missing. | 🟡 Medium |

## 1. The Multiplexing Gap (Critical)
WezTerm's core value proposition is its built-in multiplexer (like `tmux` integrated into the terminal).
- **Current Scarab**: `Session` struct holds a single `pty_master`.
- **Required Scarab**: `Session` needs to hold a **Tree of Panes**.
    - `Tab` -> `Tree<Pane>`
    - `Pane` -> `PtyMaster` + `GridState`
- **Impact**: Without this, Scarab cannot offer splits or tabs, which are standard in modern terminals (WezTerm, Windows Terminal, iTerm2).

## 2. Image Protocol Pipeline (High)
The daemon has `crates/scarab-daemon/src/images/iterm2.rs` which parses the image data. However, the **Shared Memory Protocol** (`scarab-protocol`) seems designed for character grids (`Cell` structs).
- **Problem**: How do we transport a 2MB decoded PNG from Daemon to Client over the zero-copy shared memory ring buffer?
- **WezTerm Approach**: Likely stores images in a separate cache and sends placement commands to the GUI.
- **Scarab Requirement**:
    1.  Daemon parses image.
    2.  Daemon writes image blob to a shared memory region (or separate SHM segment).
    3.  Daemon sends "Image Placement" command in the grid (or as an overlay event).
    4.  Client reads blob, creates texture, renders overlay.

## 3. Configuration API
WezTerm's `wezterm.on('event', ...)` allows users to script complex behaviors. Scarab's Fusabi implementation is currently loading static values.
- **Gap**: Event hooks in Fusabi.
- **Gap**: Exposing the `SessionManager` and `Window` objects to Fusabi scripts.

## Recommendations
1.  **Prioritize Multiplexing Data Model**: Refactor `Session` to support multiple PTYs. This is a breaking change for the internal architecture and should be done before v0.1.0 stable.
2.  **Design Image Transport**: Define how binary assets move across the IPC boundary.
