# Issue #1: VTE Parser & Grid State Management

**Phase**: 1A - Core Terminal Emulation
**Priority**: 🔴 Critical
**Workstream**: Terminal Emulation
**Estimated Effort**: 1-2 weeks
**Assignee**: Terminal Emulation Specialist Agent

---

## 🎯 Objective

Integrate a VTE (Virtual Terminal Emulator) parser to handle ANSI escape sequences and update the SharedState grid with proper terminal emulation.

---

## 📋 Background

Currently, the daemon reads PTY output but doesn't parse it. We need to:
1. Parse ANSI/VT100 escape sequences
2. Update the SharedState grid cells with colors, attributes, and text
3. Handle cursor positioning and scrollback

The `alacritty_terminal` crate has dependency conflicts. We need to either:
- **Option A**: Fix dependency conflicts (preferred)
- **Option B**: Use `vte` crate directly with custom state machine
- **Option C**: Write minimal parser for common sequences

---

## ✅ Acceptance Criteria

- [ ] VTE parser integrated and compiling
- [ ] Parse basic ANSI escape sequences (SGR, cursor movement, clear)
- [ ] Update SharedState.cells with parsed content
- [ ] Update SharedState.cursor_x/cursor_y positions
- [ ] Handle UTF-8 multibyte characters correctly
- [ ] Support scrollback buffer (at least 10,000 lines)
- [ ] Can run `ls --color=auto` with colored output
- [ ] Can run `vim` with syntax highlighting
- [ ] Can run `htop` with proper rendering
- [ ] Unit tests for common escape sequences

---

## 🔧 Technical Approach

### Step 1: Dependency Resolution
```bash
# Try upgrading alacritty_terminal
cargo update -p alacritty_terminal

# Or switch to vte crate
cargo add vte --version "0.13"
```

### Step 2: Parser Integration (daemon/src/main.rs)
```rust
use vte::{Parser, Perform};

struct TerminalState {
    grid: &mut SharedState,
    parser: Parser,
}

impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        // Update grid cell at cursor position
    }

    fn execute(&mut self, byte: u8) {
        // Handle control characters (CR, LF, etc.)
    }

    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, action: char) {
        // Handle CSI sequences (colors, cursor, etc.)
    }
}
```

### Step 3: Grid Update Logic
- Maintain cursor position in SharedState
- Handle line wrapping
- Implement scrollback ring buffer
- Support double-width characters

### Step 4: Testing
```bash
# Test basic commands
echo -e "\x1b[31mRed Text\x1b[0m"
ls --color=auto
cat some_file.txt
vim test.txt
```

---

## 📦 Deliverables

1. **Code**: Updated `scarab-daemon/src/main.rs` with VTE parser
2. **Tests**: Unit tests in `scarab-daemon/src/vte_tests.rs`
3. **Documentation**: Comment VTE parsing flow
4. **Examples**: Test scripts for common terminal programs

---

## 🔗 Dependencies

- **Blocks**: Issue #2 (Text Rendering) - needs correct grid data
- **Depends On**: None (can start immediately)

---

## 📚 Resources

- [VTE Crate Docs](https://docs.rs/vte/)
- [ANSI Escape Sequences](https://en.wikipedia.org/wiki/ANSI_escape_code)
- [Alacritty VTE Implementation](https://github.com/alacritty/alacritty/tree/master/alacritty_terminal)
- [VT100 User Guide](https://vt100.net/docs/vt100-ug/)

---

## 🎯 Success Metrics

- ✅ All common shell commands render correctly
- ✅ Colors and text attributes work
- ✅ Vim navigation and editing functional
- ✅ No visual artifacts or rendering bugs
- ✅ Parser overhead <5% CPU during heavy output

---

## 💡 Implementation Notes

### Performance Considerations
- Parse in chunks (don't process byte-by-byte)
- Use lock-free updates to SharedState
- Batch grid updates when possible
- Profile with `perf` or `cargo flamegraph`

### Edge Cases to Handle
- Incomplete escape sequences at buffer boundary
- Invalid UTF-8 sequences
- Cursor out of bounds
- Rapid resize during output
- Very long lines (>10k characters)

---

## 🐛 Known Issues

- alacritty_terminal 0.24.2 has rustix version conflicts
- May need to vendor or patch dependencies
- Scrollback buffer allocation strategy TBD

---

**Created**: 2025-11-21
**Labels**: `phase-1`, `critical`, `terminal-emulation`, `parser`
