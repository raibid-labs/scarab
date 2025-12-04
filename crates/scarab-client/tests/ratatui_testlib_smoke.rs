//! Smoke tests using ratatui-testlib for PTY-level fidelity testing.
//!
//! These tests validate that Scarab's terminal emulation and rendering behaves correctly
//! when tested through a real PTY interface. This provides higher fidelity than unit tests
//! by testing the full pipeline: PTY → VTE parsing → Shared memory → Client rendering.
//!
//! ## Test Strategy
//!
//! We use ratatui-testlib's `TuiTestHarness` to:
//! 1. Spawn scarab-daemon in a PTY
//! 2. Send commands and keyboard input
//! 3. Verify output appears correctly in the terminal grid
//! 4. Test navigation hotkeys produce expected escape sequences
//!
//! ## Environment Variable Gate
//!
//! Many tests require the daemon binary and PTY support. Set `SCARAB_TEST_RTL=1` to run
//! the full test suite including graphics protocol and navigation tests.
//!
//! ## Current Limitations (ratatui-testlib v0.1.0)
//!
//! - **Bevy ECS integration incomplete**: BevyTuiTestHarness is a placeholder
//! - **No direct grid access**: We test via PTY output, not SharedMemory directly
//! - **Process-based only**: Can't test in-process Bevy systems
//! - **AsyncTuiTestHarness**: Not yet available in v0.1.0 (async-tokio feature is MVP)
//!
//! ## Gaps Identified (for upstream issues)
//!
//! 1. **Bevy ECS Component Querying**: Need ability to query Bevy entities/components
//! 2. **SharedMemory Integration**: Direct access to scarab-protocol::SharedState
//! 3. **Hybrid Process Testing**: Daemon in PTY + Client in-process
//! 4. **Navigation Event Verification**: Detect when NavMode changes or hints spawn
//!
//! ## Future Work
//!
//! Once ratatui-testlib Phase 4 (Bevy integration) completes, we can:
//! - Query FocusableRegion components directly
//! - Verify NavHint entities spawn in hint mode
//! - Test coordinate conversion without rendering
//! - Validate LinkHintsState and NavState resources
//!
//! See integration plan: docs/testing/RATATUI_TESTLIB_INTEGRATION_PLAN.md
//! Upstream tracking: https://github.com/raibid-labs/ratatui-testlib/blob/main/docs/ROADMAP.md#phase-4-bevy-ecs-integration

use anyhow::{Context, Result};
use ratatui_testlib::{CommandBuilder, KeyCode, TuiTestHarness};
use std::path::PathBuf;
use std::time::Duration;

// =============================================================================
// TEST GATE HELPERS
// =============================================================================

/// Returns true if SCARAB_TEST_RTL=1 is set, enabling the full RTL test suite.
///
/// Tests that require the daemon binary, PTY support, or graphics protocols should
/// check this before running and return Ok(()) early if not set.
fn should_run_rtl_tests() -> bool {
    std::env::var("SCARAB_TEST_RTL")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Returns true if daemon binary is available (built and accessible).
#[allow(dead_code)]
fn daemon_available() -> bool {
    get_daemon_binary().is_ok()
}

/// Returns true if PTY operations are likely to work in this environment.
/// Checks for /dev/ptmx on Unix systems.
fn pty_available() -> bool {
    #[cfg(unix)]
    {
        std::path::Path::new("/dev/ptmx").exists()
    }
    #[cfg(not(unix))]
    {
        // Windows ConPTY check would go here
        true
    }
}

/// Maximum time to wait for daemon startup
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Time to wait for command output to appear
const OUTPUT_TIMEOUT: Duration = Duration::from_millis(500);

/// Helper to find the workspace root
fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let contents =
                std::fs::read_to_string(&cargo_toml).context("Failed to read Cargo.toml")?;
            if contents.contains("[workspace]") {
                return Ok(current);
            }
        }

        if !current.pop() {
            anyhow::bail!("Could not find workspace root");
        }
    }
}

/// Helper to build and find the scarab-daemon binary
fn get_daemon_binary() -> Result<PathBuf> {
    let workspace_root = find_workspace_root()?;

    // Check CARGO_TARGET_DIR first
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let debug_bin = PathBuf::from(&target_dir).join("debug/scarab-daemon");
        let release_bin = PathBuf::from(&target_dir).join("release/scarab-daemon");
        if release_bin.exists() {
            return Ok(release_bin);
        }
        if debug_bin.exists() {
            return Ok(debug_bin);
        }
    }

    // Check standard target directory
    let debug_bin = workspace_root.join("target/debug/scarab-daemon");
    let release_bin = workspace_root.join("target/release/scarab-daemon");

    if release_bin.exists() {
        return Ok(release_bin);
    }

    if debug_bin.exists() {
        return Ok(debug_bin);
    }

    // Need to build
    println!("scarab-daemon not found, building...");
    let status = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("scarab-daemon")
        .current_dir(&workspace_root)
        .status()
        .context("Failed to execute cargo build")?;

    if !status.success() {
        anyhow::bail!("Failed to build scarab-daemon");
    }

    // Recheck after build
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let debug_bin = PathBuf::from(target_dir).join("debug/scarab-daemon");
        if debug_bin.exists() {
            return Ok(debug_bin);
        }
    }

    if debug_bin.exists() {
        Ok(debug_bin)
    } else {
        anyhow::bail!("Failed to locate scarab-daemon after build")
    }
}

/// Test 1: Basic PTY output passthrough
///
/// Verifies that text sent through the daemon's PTY appears in the terminal grid.
/// This tests the VTE parsing → SharedMemory → Rendering pipeline.
#[test]
fn test_pty_output_passthrough() -> Result<()> {
    println!("=== Test: PTY Output Passthrough ===");

    // Create PTY-based test harness (80x24 terminal)
    let mut harness = TuiTestHarness::new(80, 24)?;
    println!("Created PTY: 80x24");

    // Spawn scarab-daemon
    let daemon_bin = get_daemon_binary()?;
    println!("Daemon binary: {}", daemon_bin.display());

    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("RUST_LOG", "info");
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    println!("Spawned daemon");

    // Wait for daemon to initialize
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    // Send a simple echo command
    harness.send_text("echo 'Hello from PTY test'\r")?;
    println!("Sent: echo 'Hello from PTY test'");

    // Wait for output to appear
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    // Verify output appears in terminal grid
    let contents = harness.screen_contents();
    println!("Screen contents:\n{}", contents);

    assert!(
        contents.contains("Hello from PTY test"),
        "Expected echo output not found in terminal grid"
    );

    println!("✓ PTY output passthrough working");
    Ok(())
}

/// Test 2: Grid text rendering
///
/// Spawns a command that outputs text and verifies it renders at the correct positions
/// in the terminal grid.
#[test]
fn test_grid_text_rendering() -> Result<()> {
    println!("=== Test: Grid Text Rendering ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    // Spawn daemon
    let daemon_bin = get_daemon_binary()?;
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    // Send multi-line output
    harness.send_text("printf 'Line 1\\nLine 2\\nLine 3\\n'\r")?;
    println!("Sent: multi-line printf");

    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    let contents = harness.screen_contents();
    println!("Screen contents:\n{}", contents);

    // Verify all lines appear
    assert!(contents.contains("Line 1"), "Line 1 not found");
    assert!(contents.contains("Line 2"), "Line 2 not found");
    assert!(contents.contains("Line 3"), "Line 3 not found");

    println!("✓ Multi-line text rendering correct");
    Ok(())
}

/// Test 3: Navigation hotkey sequences
///
/// Tests that navigation hotkeys (like 'f' for hint mode) produce escape sequences
/// that can be detected in the PTY output stream.
///
/// **Note**: This is a partial test. Full validation requires Bevy ECS integration
/// to verify that NavMode actually changes and NavHint entities spawn.
///
/// **Gate**: Requires `SCARAB_TEST_RTL=1` environment variable.
#[test]
fn test_nav_hotkey_sequences() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test_nav_hotkey_sequences (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }
    if !pty_available() {
        println!("Skipping test_nav_hotkey_sequences (PTY not available)");
        return Ok(());
    }

    println!("=== Test: Navigation Hotkey Sequences ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = match get_daemon_binary() {
        Ok(bin) => bin,
        Err(e) => {
            println!("Skipping: daemon binary not available: {}", e);
            return Ok(());
        }
    };
    let mut cmd = CommandBuilder::new(daemon_bin);
    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    harness.send_text("echo 'Visit https://example.com for info'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);

    harness.send_key(KeyCode::Char('f'))?;
    println!("Sent: 'f' (enter hint mode)");

    std::thread::sleep(Duration::from_millis(200));
    harness.update_state()?;

    let contents = harness.screen_contents();
    println!("Screen after 'f':\n{}", contents);

    // TODO: Once Bevy integration is complete, we should verify:
    // - NavState.current_mode == NavMode::Hints
    // - NavHint entities spawned with labels
    // - HintOverlay components visible
    //
    // See: docs/testing/RATATUI_TESTLIB_INTEGRATION_PLAN.md (Phase 2)

    assert!(
        contents.contains("https://example.com"),
        "URL should still be visible"
    );

    println!("✓ Hotkey sent (full verification requires Bevy integration)");
    Ok(())
}

/// Test 4: Cursor position tracking
///
/// Verifies that cursor position is correctly tracked as commands are executed.
#[test]
fn test_cursor_position_tracking() -> Result<()> {
    println!("=== Test: Cursor Position Tracking ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = get_daemon_binary()?;
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);
    harness.update_state()?;

    let (row_before, col_before) = harness.cursor_position();
    println!("Initial cursor: ({}, {})", row_before, col_before);

    // Type some text (without Enter)
    harness.send_text("test input")?;
    std::thread::sleep(Duration::from_millis(100));
    harness.update_state()?;

    let (row_after, col_after) = harness.cursor_position();
    println!("After typing: ({}, {})", row_after, col_after);

    // Cursor should have moved horizontally (col increased)
    assert!(
        col_after > col_before,
        "Cursor should move right after typing text"
    );

    println!("✓ Cursor position tracking works");
    Ok(())
}

/// Test 5: Wait for text condition
///
/// Tests the wait_for helper to ensure we can wait for specific text to appear.
#[test]
fn test_wait_for_text_condition() -> Result<()> {
    println!("=== Test: Wait for Text Condition ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = get_daemon_binary()?;
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    // Send command that takes a moment to execute
    harness.send_text("sleep 0.2 && echo 'Command completed'\r")?;
    println!("Sent: sleep 0.2 && echo 'Command completed'");

    // Use wait_for to poll for the expected text (returns Result<()>)
    harness.wait_for(|state| state.contents().contains("Command completed"))?;

    let contents = harness.screen_contents();
    println!("Screen after wait:\n{}", contents);

    assert!(contents.contains("Command completed"));

    println!("✓ wait_for correctly detected text appearance");
    Ok(())
}

/// Test 6: Multiple commands in sequence
///
/// Verifies that multiple commands can be executed and their output correctly captured.
#[test]
fn test_multiple_commands_sequence() -> Result<()> {
    println!("=== Test: Multiple Commands Sequence ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = get_daemon_binary()?;
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    // Command 1
    harness.send_text("echo 'First command'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;
    assert!(harness.screen_contents().contains("First command"));
    println!("✓ Command 1 executed");

    // Command 2
    harness.send_text("echo 'Second command'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;
    assert!(harness.screen_contents().contains("Second command"));
    println!("✓ Command 2 executed");

    // Command 3
    harness.send_text("echo 'Third command'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    let contents = harness.screen_contents();
    println!("Final screen:\n{}", contents);

    // All three should be visible (in scrollback or on screen)
    assert!(
        contents.contains("Third command"),
        "Third command not found"
    );

    println!("✓ Multiple commands sequence works");
    Ok(())
}

// =============================================================================
// GRAPHICS PROTOCOL TESTS (Issue #63)
// =============================================================================

/// Test 7: Sixel image sequence handling
///
/// Verifies that Sixel DCS sequences are properly parsed without crashes.
/// This tests the VTE parser's ability to handle inline graphics.
///
/// ## What This Tests
/// - VTE parser handles Sixel DCS (Device Control String) sequences
/// - Daemon doesn't crash on graphics data
/// - Terminal remains responsive after graphics input
///
/// ## Limitations
/// Full verification of rendered images requires access to SharedMemory
/// and image buffer state, which is not yet available through ratatui-testlib.
///
/// **Gate**: Requires `SCARAB_TEST_RTL=1` environment variable.
#[test]
fn test_sixel_sequence_handling() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test_sixel_sequence_handling (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }
    if !pty_available() {
        println!("Skipping test_sixel_sequence_handling (PTY not available)");
        return Ok(());
    }

    println!("=== Test: Sixel Sequence Handling ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = match get_daemon_binary() {
        Ok(bin) => bin,
        Err(e) => {
            println!("Skipping: daemon binary not available: {}", e);
            return Ok(());
        }
    };
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    // Sixel DCS sequence: ESC P q <sixel data> ESC \
    // This draws a simple 2x2 pixel red square
    let sixel_sequence = "\x1bPq#0;2;100;0;0#0~~$~~\x1b\\";

    harness.send_text(sixel_sequence)?;
    println!("Sent: Minimal Sixel sequence (2x2 red square)");

    std::thread::sleep(Duration::from_millis(200));
    harness.update_state()?;

    let contents = harness.screen_contents();
    println!("Screen after Sixel:\n{}", contents);

    // TODO: Once SharedMemory integration is complete, verify:
    // - Image was decoded and stored in image buffer
    // - Image metadata was recorded (width, height, position)
    // See: docs/testing/RATATUI_TESTLIB_INTEGRATION_PLAN.md (Phase 3)

    harness.send_text("echo 'Post-Sixel test'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    let contents_after = harness.screen_contents();
    assert!(
        contents_after.contains("Post-Sixel test"),
        "Daemon should remain responsive after Sixel sequence"
    );

    println!("✓ Sixel sequence handled without crash");
    Ok(())
}

/// Test 8: Kitty graphics protocol basic transfer
///
/// Tests Kitty APC graphics sequence parsing and acknowledgment.
/// This verifies the daemon can handle inline graphics commands.
///
/// ## What This Tests
/// - Kitty APC (Application Program Command) graphics sequences
/// - Base64 PNG payload decoding
/// - Daemon stability with graphics input
///
/// ## Limitations
/// Full verification requires access to image buffer state and
/// rendering pipeline, which is not yet available through ratatui-testlib.
///
/// **Gate**: Requires `SCARAB_TEST_RTL=1` environment variable.
#[test]
fn test_kitty_graphics_basic() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test_kitty_graphics_basic (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }
    if !pty_available() {
        println!("Skipping test_kitty_graphics_basic (PTY not available)");
        return Ok(());
    }

    println!("=== Test: Kitty Graphics Basic ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = match get_daemon_binary() {
        Ok(bin) => bin,
        Err(e) => {
            println!("Skipping: daemon binary not available: {}", e);
            return Ok(());
        }
    };
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    // Kitty graphics: ESC _ G <params>;<base64-payload> ESC \
    // 1x1 red PNG pixel (base64 encoded)
    let red_pixel_png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
    let kitty_cmd = format!("\x1b_Ga=T,f=100,s=1,v=1;{}\x1b\\", red_pixel_png_base64);

    harness.send_text(&kitty_cmd)?;
    println!("Sent: Minimal Kitty graphics command (1x1 red PNG)");

    std::thread::sleep(Duration::from_millis(200));
    harness.update_state()?;

    let contents = harness.screen_contents();
    println!("Screen after Kitty graphics:\n{}", contents);

    // TODO: Once SharedMemory integration is complete, verify:
    // - Image was decoded from base64 and PNG format
    // - Image stored in image registry with correct ID
    // See: docs/testing/RATATUI_TESTLIB_INTEGRATION_PLAN.md (Phase 3)

    harness.send_text("echo 'Post-Kitty test'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    let contents_after = harness.screen_contents();
    assert!(
        contents_after.contains("Post-Kitty test"),
        "Daemon should remain responsive after Kitty graphics sequence"
    );

    println!("✓ Kitty graphics command handled without crash");
    Ok(())
}

/// Test 9: Kitty graphics chunked transfer
///
/// Tests multi-chunk Kitty graphics transmission using the 'm' (more) parameter.
/// This verifies the daemon can handle large images split across multiple APC sequences.
///
/// ## What This Tests
/// - Chunked image transfer using m=1 (more) and m=0 (final) parameters
/// - Chunk reassembly in the graphics pipeline
/// - Daemon stability with multi-part graphics sequences
///
/// ## Protocol Reference
/// See: https://sw.kovidgoyal.net/kitty/graphics-protocol/#chunked-transmission
///
/// **Gate**: Requires `SCARAB_TEST_RTL=1` environment variable.
#[test]
fn test_kitty_graphics_chunked_transfer() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test_kitty_graphics_chunked_transfer (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }
    if !pty_available() {
        println!("Skipping test_kitty_graphics_chunked_transfer (PTY not available)");
        return Ok(());
    }

    println!("=== Test: Kitty Graphics Chunked Transfer ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = match get_daemon_binary() {
        Ok(bin) => bin,
        Err(e) => {
            println!("Skipping: daemon binary not available: {}", e);
            return Ok(());
        }
    };
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    // Chunked transfer: m=1 means "more chunks coming", m=0 means "final chunk"
    let chunk1 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB";
    let chunk2 = "CAYAAAA";
    let chunk3 = "fFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

    let cmd1 = format!("\x1b_Ga=T,f=100,m=1;{}\x1b\\", chunk1);
    harness.send_text(&cmd1)?;
    println!("Sent: Chunk 1 of 3 (m=1)");
    std::thread::sleep(Duration::from_millis(50));

    let cmd2 = format!("\x1b_Gm=1;{}\x1b\\", chunk2);
    harness.send_text(&cmd2)?;
    println!("Sent: Chunk 2 of 3 (m=1)");
    std::thread::sleep(Duration::from_millis(50));

    let cmd3 = format!("\x1b_Gm=0;{}\x1b\\", chunk3);
    harness.send_text(&cmd3)?;
    println!("Sent: Chunk 3 of 3 (m=0, final)");
    std::thread::sleep(Duration::from_millis(200));

    harness.update_state()?;

    harness.send_text("echo 'Post-chunked-transfer test'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    let contents = harness.screen_contents();
    assert!(
        contents.contains("Post-chunked-transfer test"),
        "Daemon should remain responsive after chunked Kitty graphics transfer"
    );

    println!("✓ Chunked Kitty graphics transfer handled without crash");
    Ok(())
}

// =============================================================================
// NAVIGATION SYSTEM TESTS (Issue #63)
// =============================================================================

/// Test 10: Navigation hint mode activation
///
/// Tests entering hint mode and verifies the navigation state changes.
/// This is a partial test - full verification requires Bevy ECS access.
///
/// ## What This Tests
/// - 'f' key sends hint mode activation
/// - Terminal remains stable during mode transitions
/// - Escape key exits hint mode cleanly
///
/// ## Limitations
/// Full verification requires querying NavStateRegistry and
/// checking for NavHint entities, which requires ratatui-testlib Phase 4.
///
/// **Gate**: Requires `SCARAB_TEST_RTL=1` environment variable.
#[test]
fn test_nav_hint_mode() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test_nav_hint_mode (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }
    if !pty_available() {
        println!("Skipping test_nav_hint_mode (PTY not available)");
        return Ok(());
    }

    println!("=== Test: Navigation Hint Mode Activation ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = match get_daemon_binary() {
        Ok(bin) => bin,
        Err(e) => {
            println!("Skipping: daemon binary not available: {}", e);
            return Ok(());
        }
    };
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    harness.send_text("echo 'Check https://example.com and https://rust-lang.org'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    let contents_before = harness.screen_contents();
    println!("Screen before hint mode:\n{}", contents_before);
    assert!(contents_before.contains("https://example.com"));
    assert!(contents_before.contains("https://rust-lang.org"));

    harness.send_key(KeyCode::Char('f'))?;
    println!("Sent: 'f' (enter hint mode)");
    std::thread::sleep(Duration::from_millis(300));
    harness.update_state()?;

    let contents_after = harness.screen_contents();
    println!("Screen after 'f':\n{}", contents_after);

    // TODO: Once Bevy integration is complete, verify:
    // - NavStateRegistry.get_active().current_mode == NavMode::Hints
    // - NavHint entities spawned for each URL
    // See: docs/testing/RATATUI_TESTLIB_INTEGRATION_PLAN.md (Phase 2)

    assert!(
        contents_after.contains("https://example.com")
            || contents_after.contains("https://rust-lang.org"),
        "URLs should still be visible after entering hint mode"
    );

    harness.send_key(KeyCode::Esc)?;
    println!("Sent: Escape (exit hint mode)");
    std::thread::sleep(Duration::from_millis(200));
    harness.update_state()?;

    println!("✓ Navigation hint mode activation works");
    Ok(())
}

/// Test 11: Pane navigation with keyboard shortcuts
///
/// Tests pane navigation commands (Ctrl+H/J/K/L for vim-style directional nav).
/// This verifies that navigation keybindings are processed correctly.
///
/// ## What This Would Test
/// - Ctrl+H/J/K/L directional pane navigation
/// - PaneFocusedEvent firing on focus changes
/// - Visual focus indicator movement
///
/// ## Why Ignored
/// Multi-pane support is not yet implemented. This test requires:
/// 1. Pane splitting functionality in scarab-daemon
/// 2. Bevy ECS access to verify NavStateRegistry pane switching
/// 3. ratatui-testlib Phase 4 for component querying
///
/// See: docs/testing/RATATUI_TESTLIB_INTEGRATION_PLAN.md (Phase 2)
#[test]
#[ignore = "Multi-pane support not yet implemented"]
fn test_pane_navigation() -> Result<()> {
    if !should_run_rtl_tests() {
        println!("Skipping test_pane_navigation (SCARAB_TEST_RTL != 1)");
        return Ok(());
    }

    println!("=== Test: Pane Navigation ===");

    let mut harness = TuiTestHarness::new(80, 24)?;

    let daemon_bin = match get_daemon_binary() {
        Ok(bin) => bin,
        Err(e) => {
            println!("Skipping: daemon binary not available: {}", e);
            return Ok(());
        }
    };
    let mut cmd = CommandBuilder::new(daemon_bin);
    cmd.env("SHELL", "/bin/sh");

    harness.spawn(cmd)?;
    std::thread::sleep(DAEMON_STARTUP_TIMEOUT);

    harness.send_text("echo 'Pane navigation test'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    let contents_before = harness.screen_contents();
    println!("Initial screen:\n{}", contents_before);

    // Ctrl+H (backspace 0x08) and Ctrl+L (form feed 0x0C)
    harness.send_text("\x08")?;
    println!("Sent: Ctrl+H (navigate left)");
    std::thread::sleep(Duration::from_millis(100));

    harness.send_text("\x0C")?;
    println!("Sent: Ctrl+L (navigate right)");
    std::thread::sleep(Duration::from_millis(100));

    harness.update_state()?;

    harness.send_text("echo 'Post-nav test'\r")?;
    std::thread::sleep(OUTPUT_TIMEOUT);
    harness.update_state()?;

    let contents_after = harness.screen_contents();
    assert!(
        contents_after.contains("Post-nav test"),
        "Daemon should remain responsive after navigation commands"
    );

    println!("✓ Pane navigation commands sent");
    Ok(())
}

// =============================================================================
// DOCUMENTATION: Gaps and Future Test Cases
// =============================================================================

/// **GAP 1: Bevy ECS Component Querying**
///
/// What we want to test (blocked by ratatui-testlib Phase 4):
///
/// ```rust,ignore
/// #[test]
/// fn test_focusable_regions_spawn_in_hint_mode() -> Result<()> {
///     let mut test = BevyTuiTestHarness::with_scarab_client()?;
///
///     // Display text with URLs
///     test.send_daemon_command("echo 'Visit https://example.com'\r")?;
///     test.update()?;
///
///     // Enter hint mode
///     test.send_key(KeyCode::Char('f'))?;
///     test.update()?;
///
///     // Query Bevy ECS directly
///     let hints: Vec<&NavHint> = test.query::<&NavHint>().iter().collect();
///     assert!(hints.len() > 0, "Should spawn NavHint entities");
///
///     let nav_state = test.resource::<NavState>()?;
///     assert_eq!(nav_state.current_mode, NavMode::Hints);
///
///     Ok(())
/// }
/// ```
///
/// **Integration Plan**: See docs/testing/RATATUI_TESTLIB_INTEGRATION_PLAN.md
/// **Upstream Issue**: https://github.com/raibid-labs/ratatui-testlib/issues/TBD

/// **GAP 2: SharedMemory Direct Access**
///
/// What we want to test:
///
/// ```rust,ignore
/// #[test]
/// fn test_shared_memory_grid_synchronization() -> Result<()> {
///     let mut test = ScarabIntegrationTest::new()?;
///
///     // Access SharedState directly
///     let shared_state = test.get_shared_state()?;
///
///     // Send text to daemon
///     test.daemon_send("Hello Grid\r")?;
///     test.wait_for_sequence_update()?;
///
///     // Verify grid cells updated
///     let row_0_text: String = shared_state.cells[0..80]
///         .iter()
///         .map(|cell| char::from_u32(cell.char_codepoint).unwrap_or(' '))
///         .collect();
///
///     assert!(row_0_text.contains("Hello Grid"));
///     Ok(())
/// }
/// ```
///
/// **Integration Plan**: See docs/testing/RATATUI_TESTLIB_INTEGRATION_PLAN.md
/// **Upstream Issue**: Requires hybrid PTY + SharedMemory harness

/// **GAP 3: Navigation State Verification**
///
/// What we want to test:
///
/// ```rust,ignore
/// #[test]
/// fn test_prompt_navigation_jumps() -> Result<()> {
///     let mut test = BevyTuiTestHarness::with_scarab_client()?;
///
///     // Populate terminal with multiple prompts
///     for i in 0..5 {
///         test.send_daemon_command(&format!("echo 'Command {}'\r", i))?;
///         test.wait_for_prompt()?;
///     }
///
///     // Verify PromptMarkers resource populated
///     let markers = test.resource::<PromptMarkers>()?;
///     assert_eq!(markers.markers.len(), 5);
///
///     // Press Ctrl+Up (jump to previous prompt)
///     test.send_key_with_modifiers(KeyCode::Up, Modifiers::CONTROL)?;
///     test.update()?;
///
///     // Verify scroll position changed
///     let scroll_state = test.resource::<ScrollbackState>()?;
///     assert!(scroll_state.current_offset > 0);
///
///     Ok(())
/// }
/// ```

/// **GAP 4: Coordinate Conversion Verification**
///
/// What we want to test:
///
/// ```rust,ignore
/// #[test]
/// fn test_grid_to_world_coordinate_conversion() -> Result<()> {
///     let mut test = BevyTuiTestHarness::with_scarab_client()?;
///
///     // Spawn a focusable at grid position (10, 5)
///     test.spawn_test_focusable(FocusableRegion {
///         grid_start: (10, 5),
///         grid_end: (30, 5),
///         ..Default::default()
///     })?;
///
///     test.update()?;
///
///     // Query the entity after coordinate conversion
///     let focusable = test.query::<&FocusableRegion>().first()?;
///     let metrics = test.resource::<TerminalMetrics>()?;
///
///     // Verify screen_position was calculated correctly
///     let expected_x = 10.0 * metrics.cell_width;
///     let expected_y = -(5.0 * metrics.cell_height);
///
///     assert_eq!(focusable.screen_position, Some(Vec2::new(expected_x, expected_y)));
///
///     Ok(())
/// }
/// ```

// =============================================================================
// Test Helpers (placeholder for future expansion)
// =============================================================================

/// Helper trait for Scarab-specific test extensions
///
/// This will be implemented once ratatui-testlib Bevy integration is complete.
#[allow(dead_code)]
trait ScarabTestExt {
    /// Send a command to the daemon PTY
    fn send_daemon_command(&mut self, cmd: &str) -> Result<()>;

    /// Wait for a new prompt to appear
    fn wait_for_prompt(&mut self) -> Result<()>;

    /// Enter navigation hint mode
    fn enter_hint_mode(&mut self) -> Result<()>;

    /// Exit navigation hint mode
    fn exit_hint_mode(&mut self) -> Result<()>;

    /// Get current NavState
    fn nav_state(&self) -> Result<&scarab_client::navigation::NavState>;

    /// Query all FocusableRegion components
    fn focusables(&self) -> Result<Vec<scarab_client::navigation::focusable::FocusableRegion>>;
}

// Note: Implementation blocked until ratatui-testlib Phase 4 (Bevy ECS integration)
// See: https://github.com/raibid-labs/ratatui-testlib/blob/main/docs/ROADMAP.md
