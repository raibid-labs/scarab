# Command Timer Plugin

Automatically times command execution and alerts you when commands take too long. Useful for identifying performance bottlenecks in scripts and workflows.

## Features

- Times every command automatically
- Configurable thresholds for warnings
- Nicely formatted duration display
- Tracks exit codes
- No performance impact

## Installation

```bash
just plugin-build command-timer
```

## Configuration

Add to your `~/.config/scarab/config.toml`:

```toml
[[plugins]]
name = "command-timer"
enabled = true

[plugins.config]
warning_threshold = 5000   # 5 seconds
slow_threshold = 10000     # 10 seconds
notify_all = false         # Only notify on slow commands
```

## Usage

The plugin runs automatically. Try these commands:

```bash
# Fast command (no notification)
echo "hello"

# Slow command (get notification)
sleep 6

# Very slow command (warning notification)
sleep 11
```

## Output Examples

Fast command (< 5s):
```
Command 'echo hello' took 12ms (exit: 0)
```

Slow command (5-10s):
```
Command Duration
'cargo build' took 7.2s
```

Very slow command (> 10s):
```
Slow Command
'npm install' took 2m 15s
```

## How It Works

1. **OnPreCommand** - Records start time before command runs
2. **OnPostCommand** - Calculates duration after command completes
3. Compares duration against thresholds
4. Shows appropriate notification based on duration

## Use Cases

- **CI/CD Optimization** - Identify slow build steps
- **Script Profiling** - Find bottlenecks in shell scripts
- **Workflow Analysis** - Track which commands slow you down
- **Awareness** - Know when to grab coffee

## Future Enhancements

- Command history with durations
- Statistics and averages
- Per-command threshold overrides
- Visual timeline of command execution

## License

MIT
