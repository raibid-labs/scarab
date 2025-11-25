# Scarab Justfile
# Commands for building, running, and testing the Scarab terminal emulator

# Default recipe - show available commands
default:
    @just --list

# Build all workspace crates
build:
    cargo build

# Build with release optimizations
build-release:
    cargo build --release

# Install binaries to ~/.local/bin (or custom prefix)
install PREFIX="~/.local":
    #!/usr/bin/env bash
    set -e

    # Expand tilde
    PREFIX_EXPANDED="${PREFIX/#\~/$HOME}"
    BIN_DIR="$PREFIX_EXPANDED/bin"

    echo "🪲 Installing Scarab Terminal to $BIN_DIR"

    # Build release binaries
    echo "Building release binaries..."
    cargo build --release

    # Create bin directory if it doesn't exist
    mkdir -p "$BIN_DIR"

    # Install binaries
    echo "Installing binaries..."
    cp target/release/scarab-daemon "$BIN_DIR/"
    cp target/release/scarab-client "$BIN_DIR/"

    # Create scarab symlink to client
    ln -sf "$BIN_DIR/scarab-client" "$BIN_DIR/scarab"

    # Make executable
    chmod +x "$BIN_DIR/scarab-daemon"
    chmod +x "$BIN_DIR/scarab-client"
    chmod +x "$BIN_DIR/scarab"

    # Install plugin compiler if it exists
    if [ -f "target/release/scarab-plugin-compiler" ]; then
        cp target/release/scarab-plugin-compiler "$BIN_DIR/"
        chmod +x "$BIN_DIR/scarab-plugin-compiler"
        echo "✓ scarab-plugin-compiler → $BIN_DIR/scarab-plugin-compiler"
    fi

    echo "✓ scarab-daemon → $BIN_DIR/scarab-daemon"
    echo "✓ scarab-client → $BIN_DIR/scarab-client"
    echo "✓ scarab → $BIN_DIR/scarab (symlink to scarab-client)"
    echo ""
    echo "Installation complete! 🎉"
    echo ""
    echo "Make sure $BIN_DIR is in your PATH:"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
    echo ""
    echo "To start using Scarab:"
    echo "  1. Start the daemon: scarab-daemon &"
    echo "  2. Launch the client: scarab"
    echo ""
    echo "Or use the quick start: just run"

# Uninstall binaries from ~/.local/bin (or custom prefix)
uninstall PREFIX="~/.local":
    #!/usr/bin/env bash

    # Expand tilde
    PREFIX_EXPANDED="${PREFIX/#\~/$HOME}"
    BIN_DIR="$PREFIX_EXPANDED/bin"

    echo "🪲 Uninstalling Scarab Terminal from $BIN_DIR"

    # Remove binaries
    rm -f "$BIN_DIR/scarab-daemon"
    rm -f "$BIN_DIR/scarab-client"
    rm -f "$BIN_DIR/scarab"
    rm -f "$BIN_DIR/scarab-plugin-compiler"

    echo "✓ Uninstalled from $BIN_DIR"
    echo ""
    echo "Note: Config files remain in ~/.config/scarab"
    echo "To remove config: rm -rf ~/.config/scarab"

# Run both daemon and client in parallel (main development command)
run: daemon client

# Run the daemon (headless server)
daemon:
    #!/usr/bin/env bash
    echo "🪲 Starting Scarab daemon..."
    cargo run -p scarab-daemon

# Run the client (Bevy GUI)
client:
    #!/usr/bin/env bash
    # Wait a moment for daemon to start
    sleep 1
    echo "🪲 Starting Scarab client..."
    cargo run -p scarab-client

# Run daemon and client in separate tmux panes (recommended)
dev:
    #!/usr/bin/env bash
    if ! command -v tmux &> /dev/null; then
        echo "Error: tmux not found. Install tmux or use 'just run-split' for alternatives."
        exit 1
    fi

    # Create new tmux session or attach to existing
    SESSION="scarab-dev"

    if tmux has-session -t $SESSION 2>/dev/null; then
        echo "Attaching to existing session: $SESSION"
        tmux attach-session -t $SESSION
    else
        echo "Creating new tmux session: $SESSION"
        # Create session with daemon in first pane
        tmux new-session -d -s $SESSION -n "scarab" "cargo run -p scarab-daemon"
        # Split window and run client in second pane
        tmux split-window -h -t $SESSION "sleep 2 && cargo run -p scarab-client"
        # Select layout and attach
        tmux select-layout -t $SESSION even-horizontal
        tmux attach-session -t $SESSION
    fi

# Run daemon and client with process substitution (background daemon)
run-bg:
    #!/usr/bin/env bash
    echo "🪲 Starting daemon in background..."
    cargo run -p scarab-daemon > /tmp/scarab-daemon.log 2>&1 &
    DAEMON_PID=$!
    echo "Daemon PID: $DAEMON_PID"

    sleep 2

    echo "🪲 Starting client..."
    cargo run -p scarab-client

    # Kill daemon when client exits
    echo "Stopping daemon..."
    kill $DAEMON_PID 2>/dev/null || true

# Check all crates for errors
check:
    cargo check --workspace

# Run tests for all crates
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run benchmarks
bench:
    cargo bench --workspace

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
clippy:
    cargo clippy --workspace -- -D warnings

# Run clippy with fixes
clippy-fix:
    cargo clippy --workspace --fix --allow-dirty --allow-staged

# Build documentation
doc:
    cargo doc --workspace --no-deps --open

# Check dependency tree
tree:
    cargo tree

# Update dependencies
update:
    cargo update

# Profile with release build
profile:
    cargo build --profile profiling -p scarab-daemon
    cargo build --profile profiling -p scarab-client

# Run with performance profiling enabled
run-profile:
    #!/usr/bin/env bash
    echo "🪲 Running with profiling enabled..."
    cargo run --profile profiling -p scarab-daemon &
    DAEMON_PID=$!
    sleep 2
    cargo run --profile profiling -p scarab-client
    kill $DAEMON_PID 2>/dev/null || true

# Build specific crate
build-crate crate:
    cargo build -p {{crate}}

# Run specific crate
run-crate crate:
    cargo run -p {{crate}}

# Watch and rebuild on changes (requires cargo-watch)
watch:
    cargo watch -x check -x test

# Install cargo-watch if not present
install-watch:
    cargo install cargo-watch

# Show crate sizes
bloat:
    cargo bloat --release -p scarab-daemon
    cargo bloat --release -p scarab-client

# Audit dependencies for security issues
audit:
    cargo audit

# Install cargo-audit if not present
install-audit:
    cargo install cargo-audit

# Run all quality checks (format, clippy, test)
ci: fmt-check clippy test

# Quick iteration: check + test
quick: check test

# Full rebuild from scratch
rebuild: clean build

# Kill any running scarab processes
kill:
    #!/usr/bin/env bash
    echo "Killing scarab processes..."
    pkill -f scarab-daemon || true
    pkill -f scarab-client || true
    echo "Done"

# Clean up shared memory segments
clean-shm:
    #!/usr/bin/env bash
    echo "Cleaning up shared memory..."
    rm -f /dev/shm/scarab_shm_* || true
    rm -f /tmp/scarab_* || true
    echo "Done"

# Full cleanup (kill processes + clean shared memory + cargo clean)
nuke: kill clean-shm clean

# Show build status
status:
    @echo "🪲 Scarab Build Status"
    @echo "====================="
    @cargo --version
    @rustc --version
    @echo ""
    @echo "Workspace crates:"
    @cargo metadata --no-deps --format-version 1 | grep -o '"name":"[^"]*"' | cut -d'"' -f4 | grep scarab

# ============================================
# Plugin Development Commands
# ============================================

# Hot-reload development mode for a plugin
dev-mode plugin_name:
    #!/usr/bin/env bash
    echo "🔄 Starting dev mode for {{plugin_name}}"
    echo "   Watching: plugins/{{plugin_name}}"
    echo "   Press Ctrl+C to stop"

    # Check if cargo-watch is installed
    if ! command -v cargo-watch &> /dev/null; then
        echo "Error: cargo-watch not found. Install with: cargo install cargo-watch"
        exit 1
    fi

    # Build plugin initially
    just plugin-build {{plugin_name}}

    # Watch for changes and rebuild
    cargo watch -w plugins/{{plugin_name}} \
        -s "just plugin-build {{plugin_name}}" \
        -s "just reload-plugin {{plugin_name}}"

# Create new plugin from template
plugin-new plugin_name type="frontend":
    #!/usr/bin/env bash
    mkdir -p plugins/{{plugin_name}}

    if [ "{{type}}" = "frontend" ]; then
        cat > plugins/{{plugin_name}}/{{plugin_name}}.fsx << 'EOF'
module {{plugin_name}}

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "{{plugin_name}}"
    Version = "0.1.0"
    Description = "TODO: Add description"
    Author = "Your Name"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    ctx.Log Info "{{plugin_name}} loaded!"
    async { return Ok () }

[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    // TODO: Handle key presses
    async { return Continue }
EOF
    else
        cat > plugins/{{plugin_name}}/{{plugin_name}}.fsx << 'EOF'
module {{plugin_name}}

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "{{plugin_name}}"
    Version = "0.1.0"
    Description = "TODO: Add description"
    Author = "Your Name"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    ctx.Log Info "{{plugin_name}} loaded!"
    async { return Ok () }

[<OnOutput>]
let onOutput (ctx: PluginContext) (text: string) =
    // TODO: Process terminal output
    async { return Continue }
EOF
    fi

    # Create manifest
    cat > plugins/{{plugin_name}}/plugin.toml << 'EOF'
[plugin]
name = "{{plugin_name}}"
version = "0.1.0"
runtime = "{{type}}"

[plugin.metadata]
description = "TODO: Add description"
author = "Your Name"
license = "MIT"

[hooks]
# List enabled hooks (uncomment as needed)
# on_load = true
# on_output = true
# on_input = true
# on_resize = true
# on_key_press = true
EOF

    # Create README
    cat > plugins/{{plugin_name}}/README.md << 'EOF'
# {{plugin_name}}

TODO: Add description

## Installation

```bash
just plugin-build {{plugin_name}}
```

## Configuration

Add to your `~/.config/scarab/config.toml`:

```toml
[[plugins]]
name = "{{plugin_name}}"
enabled = true
```

## Usage

TODO: Add usage instructions

## Development

```bash
just dev-mode {{plugin_name}}
```
EOF

    echo "✅ Created new {{type}} plugin: {{plugin_name}}"
    echo "   Location: plugins/{{plugin_name}}"
    echo "   Next steps:"
    echo "     1. Edit plugins/{{plugin_name}}/{{plugin_name}}.fsx"
    echo "     2. Run: just dev-mode {{plugin_name}}"

# Build plugin to .fzb bytecode
plugin-build plugin_name:
    #!/usr/bin/env bash
    if [ ! -d "plugins/{{plugin_name}}" ]; then
        echo "Error: Plugin not found: plugins/{{plugin_name}}"
        exit 1
    fi

    echo "🔨 Building plugin: {{plugin_name}}"

    # For now, just copy the .fsx file as we're setting up the infrastructure
    # In the future, this will use fusabi-frontend to compile to .fzb
    if [ -f "plugins/{{plugin_name}}/{{plugin_name}}.fsx" ]; then
        echo "   Source: plugins/{{plugin_name}}/{{plugin_name}}.fsx"
        echo "   Note: Fusabi compilation infrastructure is being developed"
        echo "   For now, .fsx files will be interpreted by the frontend"
    else
        echo "Error: Source file not found: plugins/{{plugin_name}}/{{plugin_name}}.fsx"
        exit 1
    fi

# Test plugin
plugin-test plugin_name:
    #!/usr/bin/env bash
    echo "🧪 Testing plugin: {{plugin_name}}"
    cargo test -p scarab-plugin-api -- {{plugin_name}}

# Reload plugin in running daemon
reload-plugin plugin_name:
    #!/usr/bin/env bash
    if pgrep scarab-daemon > /dev/null; then
        echo "🔄 Reloading plugin: {{plugin_name}}"
        # Send SIGUSR1 to daemon to reload plugins
        pkill -SIGUSR1 scarab-daemon
        echo "✅ Plugin reloaded"
    else
        echo "⚠️  Daemon not running"
    fi

# Package plugin for distribution
plugin-package plugin_name:
    #!/usr/bin/env bash
    just plugin-build {{plugin_name}}
    mkdir -p dist/plugins
    tar -czf dist/plugins/{{plugin_name}}.tar.gz \
        -C plugins/{{plugin_name}} \
        {{plugin_name}}.fsx \
        plugin.toml \
        README.md
    echo "📦 Package created: dist/plugins/{{plugin_name}}.tar.gz"

# Build a single Fusabi plugin (.fsx -> .fzb)
plugin-build file:
    #!/usr/bin/env bash
    echo "Building plugin: {{file}}"
    ./scripts/build-plugin.sh "{{file}}"

# Build all example plugins
plugin-build-all:
    #!/usr/bin/env bash
    echo "Building all example plugins..."
    ./scripts/build-plugin.sh --all

# Validate a single plugin
plugin-validate file:
    #!/usr/bin/env bash
    echo "Validating plugin: {{file}}"
    ./scripts/plugin-validator.sh "{{file}}"

# Validate all example plugins
plugin-validate-all:
    #!/usr/bin/env bash
    echo "Validating all example plugins..."
    ./scripts/plugin-validator.sh --all

# Watch plugins and rebuild on changes (requires cargo-watch)
plugin-watch:
    #!/usr/bin/env bash
    echo "Watching plugins for changes..."
    if ! command -v cargo-watch &> /dev/null; then
        echo "Error: cargo-watch not found. Install with: cargo install cargo-watch"
        exit 1
    fi
    cargo watch -w examples/fusabi -s "just plugin-build-all"

# Test plugin loading in daemon
plugin-test:
    #!/usr/bin/env bash
    echo "Testing plugin loading..."
    cargo test -p scarab-daemon plugin -- --nocapture

# Run plugin CI checks (validate + test)
plugin-ci: plugin-validate-all plugin-test
    @echo "Plugin CI checks complete"

# Show plugin development status
plugin-status:
    @echo "Fusabi Plugin Development Status"
    @echo "================================="
    @echo ""
    @echo "Example Plugins:"
    @find examples/fusabi -name "*.fsx" -type f 2>/dev/null | wc -l | xargs echo "  .fsx files:"
    @find examples/fusabi -name "*.fzb" -type f 2>/dev/null | wc -l | xargs echo "  .fzb files:"
    @echo ""
    @find plugins -name "*.fsx" -type f 2>/dev/null | wc -l | xargs echo "  plugins/ .fsx files:"
    @echo ""
    @echo "Plugin API Version: 0.1.0"
    @echo ""
    @echo "Available Commands:"
    @echo "  just dev-mode NAME          - Hot reload dev server"
    @echo "  just plugin-new NAME TYPE   - Create new plugin"
    @echo "  just plugin-build NAME      - Build plugin"
    @echo "  just plugin-test NAME       - Test plugin"
    @echo "  just plugin-package NAME    - Package plugin"
    @echo "  just plugin-build-all       - Build all plugins"
    @echo "  just plugin-validate-all    - Validate all plugins"
    @echo "  just plugin-watch           - Watch and rebuild"

# Clean plugin build artifacts
plugin-clean:
    #!/usr/bin/env bash
    echo "Cleaning plugin build artifacts..."
    find examples/fusabi -name "*.fzb" -type f -delete 2>/dev/null || true
    find examples/fusabi-config -name "*.fzb" -type f -delete 2>/dev/null || true
    find plugins -name "*.fzb" -type f -delete 2>/dev/null || true
    echo "Done"
