# Integration Test Plan for Scarab using Mimic

We will use `mimic` to verify the correctness of Scarab's VTE (Virtual Terminal Emulator) parser. By feeding identical inputs to both Scarab's `TerminalState` and Mimic's reference `ScreenState`, we can assert that Scarab renders terminal sequences correctly.

## 1. Add Dependency
Update `raibid-labs/scarab/crates/scarab-daemon/Cargo.toml` to include `mimic`.

## 2. Create Test
Create `raibid-labs/scarab/crates/scarab-daemon/tests/vte_conformance.rs`.

## 3. Implementation
The test will:
1.  Initialize Scarab's VTE parser.
2.  Initialize Mimic's ScreenState.
3.  Feed complex ANSI sequences (colors, cursor movement).
4.  Assert that the resulting grid state matches.
