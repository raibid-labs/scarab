# Git Status Plugin

Automatically tracks git branch and status by parsing git command output. Shows branch information in notifications and can update a status bar.

## Features

- Detects branch switches
- Parses git status output
- Tracks repository state
- Low performance impact
- No external git calls

## Installation

```bash
just plugin-build git-status
```

## Configuration

Add to your `~/.config/scarab/config.toml`:

```toml
[[plugins]]
name = "git-status"
enabled = true

[plugins.config]
notify_branch_change = true
show_status = true
update_frequency = 10
```

## Usage

The plugin monitors git command output automatically:

```bash
# Initialize a repo
git init
git status  # Plugin detects branch: main

# Create and switch branches
git checkout -b feature/awesome
# Plugin notifies: ⎇ feature/awesome

# Check status
git status
# Plugin updates internal state
```

## How It Works

1. Monitors all terminal output for git command responses
2. Uses regex to parse branch names and status
3. Stores current git state in plugin data
4. Sends notifications on important changes
5. Can send updates to frontend for status bar display

## Detected Patterns

- `On branch <name>` - git status output
- `Switched to branch '<name>'` - git checkout output
- `Your branch is <status>` - upstream comparison

## Future Enhancements

- Direct git repository monitoring (libgit2)
- File change indicators
- Commit count display
- Stash tracking
- Integration with status bar widget

## License

MIT
