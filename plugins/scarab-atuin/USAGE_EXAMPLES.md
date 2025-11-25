# Atuin Plugin Usage Examples

This document provides practical examples of using the Scarab Atuin plugin in real-world scenarios.

## Installation & Setup

### Step 1: Install Atuin

```bash
# Install via cargo
cargo install atuin

# Or using package manager (Arch Linux)
yay -S atuin

# Or using Homebrew (macOS)
brew install atuin
```

### Step 2: Initialize Atuin

```bash
# Initialize for your shell
atuin init bash   # or zsh, fish, etc.

# Reload shell config
source ~/.bashrc  # or ~/.zshrc
```

### Step 3: Import Existing History (Optional)

```bash
# Import from shell history
atuin import auto

# Or import from specific shell
atuin import bash
atuin import zsh
```

### Step 4: Configure Plugin (Optional)

```bash
# Copy default config
mkdir -p ~/.config/scarab/plugins
cp plugins/scarab-atuin/atuin.toml ~/.config/scarab/plugins/

# Edit configuration
nano ~/.config/scarab/plugins/atuin.toml
```

## Basic Usage Scenarios

### Scenario 1: Finding Recent Git Commands

```bash
# In Scarab Terminal
$ # Press Ctrl+R
# Modal opens with search interface

# Type: git
┌────────────────────────────────────────────┐
│ Atuin History Search: git (15 results)    │
│ ┌────────────────────────────────────────┐ │
│ │ > git commit -m "feat: Add feature"   │ │
│ │   git push origin main                │ │
│ │   git pull --rebase                   │ │
│ │   git status                          │ │
│ │   git log --oneline --graph           │ │
│ └────────────────────────────────────────┘ │
│ ↑↓ navigate • Enter select • Esc close    │
└────────────────────────────────────────────┘

# Press Down arrow to select next item
# Press Enter to insert command
$ git push origin main█
```

### Scenario 2: Finding Docker Commands

```bash
# Press Ctrl+R
# Type: docker
┌────────────────────────────────────────────┐
│ Atuin History Search: docker (8 results)  │
│ ┌────────────────────────────────────────┐ │
│ │ > docker-compose up -d                │ │
│ │   docker ps -a                        │ │
│ │   docker logs -f container_name       │ │
│ │   docker exec -it container bash      │ │
│ └────────────────────────────────────────┘ │
└────────────────────────────────────────────┘

# Select and execute
$ docker-compose up -d
Starting service_1 ... done
Starting service_2 ... done
```

### Scenario 3: Complex Multi-Line Commands

```bash
# Press Ctrl+R
# Type: kubectl
┌────────────────────────────────────────────────────────────┐
│ Atuin History Search: kubectl (3 results)                 │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ > kubectl get pods -n production --watch              │ │
│ │   kubectl logs -f deployment/api --tail=100           │ │
│ │   kubectl exec -it pod/api-xxx -- /bin/bash          │ │
│ └────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘

# Select complex command
$ kubectl exec -it pod/api-xxx -- /bin/bash█
```

## Advanced Usage

### Using Command Palette

Open Scarab command palette and search for Atuin commands:

```bash
# Open command palette (Ctrl+P or configured key)
> Search Atuin History
# Opens search modal

> Sync Atuin History
# Syncs with cloud
✓ Notification: "History synchronized successfully"

> Show Command Statistics
# Displays usage stats
┌────────────────────────────────────────┐
│ Atuin Statistics                       │
│                                        │
│ Total commands: 1,234                  │
│ Unique commands: 567                   │
│ Most used: git status (89 times)      │
│ Average per day: 42                    │
└────────────────────────────────────────┘
```

### Configuration Examples

#### High-Volume User (Show more results)

```toml
[atuin]
enabled = true
max_results = 50  # Show up to 50 results
auto_sync = true  # Auto-sync for cloud backup
show_stats = true
```

#### Privacy-Focused User (Local only)

```toml
[atuin]
enabled = true
max_results = 20
auto_sync = false  # Don't sync to cloud
show_stats = false
```

#### Performance-Focused (Minimal features)

```toml
[atuin]
enabled = true
max_results = 10   # Fewer results for speed
auto_sync = false
show_stats = false
```

## Workflow Integration

### Development Workflow

```bash
# Morning routine: Check git status
$ # Ctrl+R → "git status" → Enter
$ git status
On branch main
Your branch is up to date with 'origin/main'.

# Find that complex build command
$ # Ctrl+R → "cargo build --release" → Enter
$ cargo build --release --features="full"

# Find database migration command
$ # Ctrl+R → "diesel migration" → Enter
$ diesel migration run --database-url postgres://...

# Find test command with specific args
$ # Ctrl+R → "cargo test integration" → Enter
$ cargo test --test integration_tests -- --nocapture
```

### DevOps Workflow

```bash
# Find kubectl context switch command
$ # Ctrl+R → "kubectl config use-context" → Enter
$ kubectl config use-context production

# Find terraform apply with variables
$ # Ctrl+R → "terraform apply" → Enter
$ terraform apply -var-file=prod.tfvars

# Find AWS CLI command with complex query
$ # Ctrl+R → "aws ec2 describe-instances" → Enter
$ aws ec2 describe-instances --query 'Reservations[*].Instances[*].[InstanceId,State.Name]'
```

### Data Science Workflow

```bash
# Find Jupyter notebook launch command
$ # Ctrl+R → "jupyter notebook" → Enter
$ jupyter notebook --port=8888 --no-browser

# Find Python script with arguments
$ # Ctrl+R → "python train.py" → Enter
$ python train.py --epochs=100 --batch-size=32 --lr=0.001

# Find database query command
$ # Ctrl+R → "psql -c" → Enter
$ psql -h localhost -d analytics -c "SELECT COUNT(*) FROM users WHERE created_at > NOW() - INTERVAL '7 days';"
```

## Tips & Tricks

### Tip 1: Fuzzy Search

Atuin supports fuzzy search, so you don't need exact matches:

```bash
# Search: "gps"
# Finds: "git push origin main"
# Also finds: "git pull --rebase", "git push --force", etc.
```

### Tip 2: Search by Directory

Atuin remembers where commands were run:

```bash
# Commands run in ~/projects/api/ will show up when in that directory
# Configure in ~/.config/atuin/config.toml:
# filter_mode = "directory"  # or "session", "global"
```

### Tip 3: Exclude Sensitive Commands

Add to `~/.config/atuin/config.toml`:

```toml
## Filter out commands starting with these strings
filter_mode_shell_up_key_binding = "directory"

## Don't record commands starting with a space
history_filter = [
  "^pass ",
  "^aws .* secret",
  "^export .*PASSWORD",
]
```

### Tip 4: Quick Exit

Press Escape to close search without selecting:

```bash
# Open search
$ # Ctrl+R
# Change your mind
# Press Escape
$ # Back to normal prompt
```

### Tip 5: Chain with Other Tools

Combine Atuin search with shell features:

```bash
# Select command, then modify before executing
$ # Ctrl+R → select "git commit -m 'old message'"
$ git commit -m 'old message'█
# Edit message before executing
$ git commit -m 'new message'
```

## Troubleshooting Examples

### Example 1: No Results Found

```bash
# Symptom: Search returns "No results found"
# Solution: Import your history

$ atuin import auto
Imported 1,234 commands from bash history

# Try search again
# Ctrl+R → "git"
# Now shows results
```

### Example 2: Slow Search

```bash
# Symptom: Search takes > 500ms
# Solution: Reduce max_results or rebuild database

# Edit config
nano ~/.config/scarab/plugins/atuin.toml
# Set: max_results = 10

# Or rebuild Atuin database
atuin sync
```

### Example 3: Sync Failed

```bash
# Symptom: "Sync Atuin History" shows error
# Solution: Login to Atuin

$ atuin login
# Enter credentials

# Or register if new
$ atuin register
Email: user@example.com
Username: myusername
Password: ********

# Retry sync from Scarab command palette
> Sync Atuin History
✓ Success
```

## Real-World Use Cases

### Use Case 1: Team Onboarding

New team member learning project commands:

```bash
# Senior dev shares Atuin sync key
$ atuin key
Your key: at_abc123def456...

# New dev imports team history
$ atuin login
$ atuin sync

# Now has access to all team's frequently-used commands
# Ctrl+R → "deploy staging"
# Shows: ./scripts/deploy.sh --env=staging --dry-run
```

### Use Case 2: Context Switching

Developer working on multiple projects:

```bash
# Switch to project A
$ cd ~/projects/project-a
$ # Ctrl+R → "npm"
# Shows: npm run dev, npm test, npm build

# Switch to project B
$ cd ~/projects/project-b
$ # Ctrl+R → "cargo"
# Shows: cargo run, cargo test, cargo build --release

# Atuin filters by directory automatically
```

### Use Case 3: Disaster Recovery

Lost terminal session, need to remember commands:

```bash
# Before: Lost session with important commands
# After: Open Scarab, Ctrl+R

# Find that database backup command
# Ctrl+R → "pg_dump"
# Found: pg_dump -h prod-db -U admin -d myapp > backup.sql

# Find that SSH tunnel command
# Ctrl+R → "ssh -L"
# Found: ssh -L 5432:localhost:5432 user@bastion-host
```

## Integration with Other Tools

### Integrating with fzf

While Scarab Atuin provides its own search, you can still use fzf:

```bash
# In Atuin config, keep fzf for shell history
# In Scarab, use Ctrl+R for Atuin search
# Best of both worlds!
```

### Integrating with Starship Prompt

Atuin and Starship work great together:

```toml
# ~/.config/starship.toml
[character]
success_symbol = "[➜](bold green)"
error_symbol = "[➜](bold red)"

# Commands tracked by Atuin, prompt styled by Starship
```

### Integrating with tmux

Use Atuin search in Scarab inside tmux:

```bash
# In tmux pane running Scarab
# Ctrl+R still works!
# Searches across all tmux sessions tracked by Atuin
```

## Performance Benchmarks

Expected performance with different history sizes:

| History Size | Search Time | Results Load |
|--------------|-------------|--------------|
| 1,000 cmds   | < 50ms      | < 10ms       |
| 10,000 cmds  | < 100ms     | < 20ms       |
| 100,000 cmds | < 200ms     | < 50ms       |
| 1,000,000 cmds | < 500ms   | < 100ms      |

Note: Times may vary based on system specs and disk speed.

## Next Steps

After mastering Atuin plugin:

1. **Customize Atuin**: Edit `~/.config/atuin/config.toml`
2. **Explore stats**: Use command palette → "Show Command Statistics"
3. **Set up sync**: Register account for cloud backup
4. **Share with team**: Export/import sync keys
5. **Create aliases**: For frequently-used search patterns

## Resources

- Atuin Documentation: https://docs.atuin.sh
- Scarab Plugin API: /docs/plugin-api.md
- Configuration Reference: plugins/scarab-atuin/README.md
- Testing Guide: plugins/scarab-atuin/TESTING.md
