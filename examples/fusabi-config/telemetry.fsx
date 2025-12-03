// Scarab Configuration with Telemetry Enabled
//
// This example shows how to enable telemetry/logging for development and debugging.
// Copy to ~/.config/scarab/config.fsx and adjust settings as needed.

let terminal = {
    DefaultShell = "/bin/zsh";
    ScrollbackLines = 10000;
    AltScreen = true;
    ScrollMultiplier = 3.0;
    AutoScroll = true;
    Columns = 120;
    Rows = 40
}

let font = {
    Family = "JetBrains Mono";
    Size = 14.0;
    LineHeight = 1.2;
    Fallback = ("Fira Code", "Hack", "monospace");
    BoldIsBright = true;
    UseThinStrokes = false
}

let colors = {
    Theme = "dracula";
    Opacity = 0.95;
    DimOpacity = 0.7
}

let ui = {
    LinkHints = true;
    CommandPalette = true;
    Animations = true;
    SmoothScroll = true;
    ShowTabs = true;
    CursorBlink = true
}

// Telemetry configuration for observability
// All settings are opt-in (disabled by default) to avoid performance impact
let telemetry = {
    // Log compositor FPS every N seconds (0 = disabled)
    // Example output: [INFO] Compositor: 60.2 fps (avg over 5s), 3012 frames
    FpsLogIntervalSecs = 5;

    // Log sequence number changes in compositor
    // Helps debug shared memory synchronization issues
    // Example output: [DEBUG] Sequence: 1234 -> 1235
    LogSequenceChanges = true;

    // Log dirty region sizes when blitting to shared memory
    // Useful for understanding update patterns and performance
    // Example output: [DEBUG] Blit: 847 dirty cells (4.2% of grid)
    LogDirtyRegions = false;

    // Log pane lifecycle events (create, destroy, reader status)
    // Validates tab/pane flow in the orchestrator
    // Example output: [INFO] PaneOrchestrator: Pane 1 created, reader task spawned
    LogPaneEvents = true
}

// Return the full configuration object
{
    terminal = terminal;
    font = font;
    colors = colors;
    ui = ui;
    telemetry = telemetry
}
