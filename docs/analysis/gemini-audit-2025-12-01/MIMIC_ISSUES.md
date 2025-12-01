# Mimic Feature Requests

Please submit the following issues to the `raibid-labs/mimic` repository to enable full integration with Scarab.

---

## Issue 1: Feature Request - Headless Parsing API

**Title:** Add public API for headless/stream-based parsing

**Description:**
Currently, `mimic` appears to be designed primarily around wrapping a PTY or handling interactive sessions. To use `mimic` as a verification oracle for other terminal emulators (like Scarab), we need a way to feed it raw byte streams directly.

**Use Case:**
Integration testing where a deterministic byte sequence (e.g., `\x1b[31mHello\x1b[0m`) is fed to both the system-under-test and `mimic`.

**Proposed API:**
```rust
let mut screen = ScreenState::new(80, 24);
let mut parser = VteParser::new();
let input = b"\x1b[31mHello\x1b[0m";

// Feed bytes directly without PTY overhead
parser.process(&mut screen, input);
```

---

## Issue 2: Feature Request - Public State Inspection API

**Title:** Expose Screen/Grid state for verification

**Description:**
To verify that another terminal emulator matches `mimic`'s behavior, we need to be able to inspect `mimic`'s final state after processing input. Currently, the internal grid/cell structure seems inaccessible or difficult to iterate over from external crates.

**Use Case:**
Comparing the final grid state of Scarab against Mimic to ensure correct rendering of complex ANSI sequences.

**Proposed API:**
```rust
// Allow iteration over rows/cells
for row in 0..screen.rows() {
    for col in 0..screen.cols() {
        let cell: &Cell = screen.get_cell(col, row);
        // Need access to these fields:
        println!("Char: {}, FG: {:?}, BG: {:?}", cell.char, cell.fg, cell.bg);
    }
}

// Or a structured export
let snapshot: GridSnapshot = screen.snapshot();
```
