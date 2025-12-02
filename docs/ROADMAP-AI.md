# Scarab Development Roadmap
**Last Updated:** December 2, 2025
**Purpose:** AI-readable development context and next steps

## Project Overview

Scarab is a high-performance, split-process terminal emulator built in Rust:
- **Daemon** (`scarab-daemon`): Headless server owning PTY processes, VTE parsing, session management
- **Client** (`scarab-client`): Bevy-based GUI reading from shared memory
- **Protocol** (`scarab-protocol`): Zero-copy IPC via shared memory (`#[repr(C)]` structs)
- **Plugins**: Fusabi scripting language (F# dialect) for extensibility

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                         scarab-client                            │
│  • Bevy game engine (GPU rendering)                             │
│  • cosmic-text for text shaping                                 │
│  • Reads SharedState via mmap                                   │
│  • Sends ControlMessage via Unix socket                         │
└─────────────────────────┬───────────────────────────────────────┘
                          │ Unix Socket (commands)
                          │ Shared Memory (display data)
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                         scarab-daemon                            │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ SessionManager                                               ││
│  │  └─ Session                                                  ││
│  │      ├─ Tab 1                                                ││
│  │      │   ├─ Pane 1 (PTY + TerminalState + Grid)             ││
│  │      │   └─ Pane 2 (PTY + TerminalState + Grid)             ││
│  │      └─ Tab 2                                                ││
│  │          └─ Pane 3 (PTY + TerminalState + Grid)             ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ PaneOrchestrator                                             ││
│  │  • Spawns reader task per pane                               ││
│  │  • Reads PTY output → feeds to pane's TerminalState          ││
│  │  • Handles PaneCreated/PaneDestroyed lifecycle               ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ Compositor (main loop @ 60fps)                               ││
│  │  • Gets active pane from SessionManager                      ││
│  │  • Blits active pane's Grid → SharedState                    ││
│  │  • Updates sequence_number for client sync                   ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ IPC Server                                                   ││
│  │  • Accepts client connections                                ││
│  │  • Routes input to active pane's PTY                         ││
│  │  • Handles tab/pane/session commands via SessionManager      ││
│  │  • Notifies orchestrator on pane lifecycle events            ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Completed Work

### Phase 1: Multiplexing Architecture (COMPLETE)

**Commits:**
- `662d519` feat(daemon): implement multiplexing architecture with tabs and panes
- `f5b4c5c` feat(daemon): decouple VTE from SharedState for multiplexing
- `80f583c` feat(daemon): add PaneOrchestrator for parallel PTY reading
- `fdc589c` feat(daemon): wire IPC tab/pane commands to SessionManager
- `105c1c6` feat(daemon): notify orchestrator on tab close for pane cleanup

**What was done:**
1. **Data Model** - Session/Tab/Pane hierarchy implemented
   - `Session` contains multiple `Tab`s
   - `Tab` contains multiple `Pane`s
   - `Pane` owns PTY master, TerminalState, and Grid

2. **VTE Decoupling** - TerminalState writes to local Grid buffer
   - `blit_to_shm()` copies Grid to SharedState
   - Each pane has isolated terminal state

3. **PaneOrchestrator** - Parallel PTY reading
   - Spawns reader task for each pane
   - Handles `PaneCreated`/`PaneDestroyed` messages
   - All panes read in parallel (not just active one)

4. **Compositor Pattern** - Main loop blits active pane
   - ~60fps refresh rate
   - Only active pane copied to SharedState
   - Sequence number updated for client sync

5. **IPC Wiring** - Commands route to SessionManager
   - Tab commands: `TabCreate`, `TabClose`, `TabSwitch`, `TabRename`, `TabList`
   - Pane commands: `PaneSplit`, `PaneClose`, `PaneFocus`, `PaneResize`
   - Orchestrator notified on pane lifecycle changes

**Key Files:**
- `crates/scarab-daemon/src/session/` - Session/Tab/Pane data model
- `crates/scarab-daemon/src/orchestrator.rs` - PaneOrchestrator
- `crates/scarab-daemon/src/vte.rs` - TerminalState with Grid and blit_to_shm()
- `crates/scarab-daemon/src/ipc.rs` - IPC server with command routing
- `crates/scarab-daemon/src/main.rs` - Compositor loop

## Next Steps (Priority Order)

### Priority 1: Image Rendering Pipeline (HIGH)

**Problem:** iTerm2 image parser exists but rendering pipeline to client is missing.

**Current State:**
- `crates/scarab-daemon/src/images/iterm2.rs` - Parses iTerm2 image sequences
- `crates/scarab-daemon/src/images/placement.rs` - ImagePlacement tracking
- SharedState only supports character cells, not image blobs

**Required Work:**

1. **Protocol Extension** (`scarab-protocol/src/lib.rs`):
   ```rust
   #[repr(C)]
   pub struct SharedImageBuffer {
       pub count: u32,
       pub placements: [ImagePlacement; MAX_IMAGES],
       pub blob_offsets: [u32; MAX_IMAGES],
       pub blob_data: [u8; IMAGE_BUFFER_SIZE], // e.g., 16MB
   }

   #[repr(C)]
   pub struct ImagePlacement {
       pub image_id: u64,
       pub x: u16,
       pub y: u16,
       pub width: u16,
       pub height: u16,
       pub blob_offset: u32,
       pub blob_size: u32,
   }
   ```

2. **Daemon Changes:**
   - When `parse_iterm2_image()` succeeds, write decoded image to SharedImageBuffer
   - Track image placements per-pane
   - Include placements in compositor blit

3. **Client Changes:**
   - Poll SharedImageBuffer for new images
   - Load image data into Bevy textures
   - Render as sprites overlaid on terminal grid

**Files to Modify:**
- `crates/scarab-protocol/src/lib.rs`
- `crates/scarab-daemon/src/images/mod.rs`
- `crates/scarab-daemon/src/vte.rs` (handle image escape sequences)
- `crates/scarab-client/src/` (new image rendering system)

### Priority 2: Ligatures Support (MEDIUM)

**Problem:** `cosmic-text` supports Harfbuzz shaping but needs verification/enabling.

**Required Work:**
1. Verify `cosmic-text` is configured with Harfbuzz feature
2. Test with fonts that have ligatures (Fira Code, JetBrains Mono)
3. May need to adjust text layout pipeline in client

**Files to Check:**
- `crates/scarab-client/Cargo.toml` - cosmic-text features
- `crates/scarab-client/src/` - text rendering code

### Priority 3: Configuration API Enhancement (MEDIUM)

**Problem:** Fusabi scripts can load static config but lack event hooks.

**Required Work:**
1. Add event system to Fusabi integration
2. Expose `SessionManager` to scripts
3. Implement hooks: `on_tab_created`, `on_pane_split`, `on_input`, etc.

**Files to Modify:**
- `crates/scarab-daemon/src/plugin_manager/`
- `crates/scarab-plugin-api/`

### Priority 4: Shell Integration (MEDIUM)

**Problem:** Basic VTE support exists but deep semantic integration missing.

**Required Work:**
1. Implement OSC 133 (shell integration)
2. Track command start/end markers
3. Enable semantic prompt navigation

### Priority 5: Manual Testing (IMMEDIATE)

**What to Test:**
- Tab creation via keybind or IPC
- Tab switching updates screen content
- Input routes only to active pane
- Pane splitting works
- Tab close cleans up panes properly

## Known Issues

### Test Failures (Pre-existing, Unrelated to Multiplexing)
- `test_cache_basic` - VTE optimized cache test
- `test_cache_stats` - VTE optimized cache test
- `test_bytecode_plugin_lifecycle` - Fusabi adapter test

### E2E Test Failures
- 38 E2E tests fail due to socket binding issues (`/tmp/scarab-nav.sock` already in use)
- Likely test isolation issue, not code bug

## File Reference

### Core Daemon Files
| File | Purpose |
|------|---------|
| `src/main.rs` | Compositor loop, daemon initialization |
| `src/ipc.rs` | IPC server, command routing |
| `src/orchestrator.rs` | PaneOrchestrator for parallel PTY reading |
| `src/vte.rs` | TerminalState, Grid, VTE parsing |
| `src/session/mod.rs` | Session module exports |
| `src/session/manager.rs` | Session struct, tab/pane management |
| `src/session/tab.rs` | Tab struct, pane container |
| `src/session/pane.rs` | Pane struct, PTY + TerminalState |
| `src/session/commands.rs` | Command handlers for session/tab/pane |
| `src/images/iterm2.rs` | iTerm2 image parser |
| `src/images/placement.rs` | Image placement tracking |

### Protocol Files
| File | Purpose |
|------|---------|
| `src/lib.rs` | SharedState, ControlMessage, DaemonMessage |

### Client Files
| File | Purpose |
|------|---------|
| `src/main.rs` | Bevy app setup |
| `src/terminal/` | Terminal rendering systems |

## Development Commands

```bash
# Build daemon
cargo build -p scarab-daemon

# Run daemon tests
cargo test -p scarab-daemon --lib

# Run daemon
cargo run -p scarab-daemon

# Run client (separate terminal)
cargo run -p scarab-client

# Build entire workspace
cargo build --workspace

# Run all tests
cargo test --workspace
```

## WezTerm Parity Status

| Feature | WezTerm | Scarab | Status |
|---------|---------|--------|--------|
| Tabs | Yes | Yes | ✅ Complete |
| Panes/Splits | Yes | Yes | ✅ Complete |
| Images (iTerm2) | Yes | Parser only | 🔴 Needs pipeline |
| Images (Sixel) | Yes | No | 🔴 Not started |
| Images (Kitty) | Yes | No | 🔴 Not started |
| Ligatures | Yes | Unknown | 🟡 Needs verification |
| SSH/Domains | Yes | No | ⚪ Future |
| Shell Integration | Yes | Basic | 🟡 Partial |
| Config Scripting | Lua | Fusabi | 🟡 Partial |

## Audit Documents

- `docs/audits/gemini-roadmap-2025-12-02/01-multiplexing-gap.md` - Phase 1.5 completion
- `docs/audits/gemini-wezterm-parity-2025-12-02/01-parity-gap-analysis.md` - Feature comparison
- `docs/audits/gemini-wezterm-parity-2025-12-02/03-implementation-plan.md` - Implementation details
