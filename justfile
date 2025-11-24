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

# Create a new plugin from template
plugin-new name:
    #!/usr/bin/env bash
    PLUGIN_NAME="{{name}}"
    PLUGIN_FILE="examples/fusabi/${PLUGIN_NAME}.fsx"

    if [ -f "$PLUGIN_FILE" ]; then
        echo "Error: Plugin already exists: $PLUGIN_FILE"
        exit 1
    fi

    echo "Creating new plugin: $PLUGIN_NAME"
    mkdir -p examples/fusabi

    cat > "$PLUGIN_FILE" << 'PLUGINEOF'
    // Plugin initialization
    let on_load (ctx: PluginContext) =
        printfn "Plugin loaded!"
        Ok ()

    // Handle terminal output
    let on_output (line: string) (ctx: PluginContext) =
        // Process output line
        Continue

    // Handle user input
    let on_input (input: byte[]) (ctx: PluginContext) =
        // Process input
        Continue

    // Export plugin metadata
    let metadata = {
        name = "{{name}}"
        version = "0.1.0"
        description = "A new Fusabi plugin for Scarab"
        author = "Your Name"
    }
    PLUGINEOF

    # Add metadata header
    sed -i '1i// @name {{name}}\n// @version 0.1.0\n// @description A new Fusabi plugin for Scarab\n// @author Your Name\n// @api-version 0.1.0\n// @min-scarab-version 0.1.0\n' "$PLUGIN_FILE"

    echo "Created: $PLUGIN_FILE"
    echo "Edit the file and run: just plugin-build $PLUGIN_FILE"

# Show plugin development status
plugin-status:
    @echo "Fusabi Plugin Development Status"
    @echo "================================="
    @echo ""
    @echo "Example Plugins:"
    @find examples/fusabi -name "*.fsx" -type f | wc -l | xargs echo "  .fsx files:"
    @find examples/fusabi -name "*.fzb" -type f 2>/dev/null | wc -l | xargs echo "  .fzb files:"
    @echo ""
    @echo "Plugin API Version: 0.1.0"
    @echo ""
    @echo "Available Commands:"
    @echo "  just plugin-build FILE      - Build single plugin"
    @echo "  just plugin-build-all       - Build all plugins"
    @echo "  just plugin-validate FILE   - Validate single plugin"
    @echo "  just plugin-validate-all    - Validate all plugins"
    @echo "  just plugin-watch           - Watch and rebuild on changes"
    @echo "  just plugin-test            - Test plugin loading"
    @echo "  just plugin-new NAME        - Create new plugin from template"

# Clean plugin build artifacts
plugin-clean:
    #!/usr/bin/env bash
    echo "Cleaning plugin build artifacts..."
    find examples/fusabi -name "*.fzb" -type f -delete
    find examples/fusabi-config -name "*.fzb" -type f -delete
    echo "Done"
