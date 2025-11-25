# Atuin Plugin Testing Guide

This document provides comprehensive testing procedures for the Scarab Atuin plugin.

## Prerequisites

Before testing, ensure:

1. **Atuin is installed**:
   ```bash
   cargo install atuin
   atuin --version
   ```

2. **Atuin is initialized**:
   ```bash
   atuin init bash  # or your shell
   source ~/.bashrc  # reload config
   ```

3. **History is populated**:
   ```bash
   # Import existing history
   atuin import auto

   # Or run some commands
   ls -la
   git status
   cargo build
   docker ps
   ```

4. **Verify Atuin works**:
   ```bash
   atuin search git
   atuin stats
   ```

## Manual Testing Checklist

### 1. Plugin Loading

- [ ] Start Scarab daemon with plugin loaded
- [ ] Check logs for "Atuin plugin loaded successfully"
- [ ] Verify no error messages in startup
- [ ] Check notification shows "Atuin Plugin Loaded"

```bash
# Expected log output:
# [INFO] [scarab-atuin] Atuin plugin loaded successfully
# [INFO] [scarab-atuin] Press Ctrl+R to search command history
```

### 2. Basic Search (Ctrl+R)

- [ ] Press Ctrl+R in terminal
- [ ] Verify modal overlay appears
- [ ] Check title shows "Atuin History Search"
- [ ] Verify results are displayed (if history exists)

### 3. Search Filtering

- [ ] Open search with Ctrl+R
- [ ] Type a search term (e.g., "git")
- [ ] Verify results filter in real-time
- [ ] Try different search terms
- [ ] Verify empty search shows all recent commands

### 4. Navigation

- [ ] Open search and populate results
- [ ] Press Up arrow - verify selection moves up
- [ ] Press Down arrow - verify selection moves down
- [ ] Verify selected item is highlighted with ">"
- [ ] Try navigating to first/last items

### 5. Command Selection

- [ ] Open search and select a command
- [ ] Press Enter
- [ ] Verify modal closes
- [ ] Verify command is inserted at cursor
- [ ] Verify command can be executed

### 6. Cancel Search

- [ ] Open search with Ctrl+R
- [ ] Press Escape
- [ ] Verify modal closes
- [ ] Verify no command is inserted
- [ ] Verify terminal returns to normal state

### 7. Command Palette Integration

- [ ] Open Scarab command palette
- [ ] Verify "Search Atuin History" appears
- [ ] Select "Search Atuin History"
- [ ] Verify search modal opens

- [ ] Select "Sync Atuin History"
- [ ] Verify sync notification appears
- [ ] Check logs for sync completion

- [ ] Select "Show Command Statistics"
- [ ] Verify stats notification appears

### 8. Auto-Sync (if enabled)

- [ ] Enable auto_sync in config
- [ ] Run a command
- [ ] Check logs for "Atuin history synced"
- [ ] Verify no performance degradation

### 9. Configuration

- [ ] Edit `~/.config/scarab/plugins/atuin.toml`
- [ ] Set `enabled = false`
- [ ] Restart Scarab
- [ ] Verify plugin is disabled
- [ ] Re-enable and restart
- [ ] Verify plugin loads again

- [ ] Change `max_results = 5`
- [ ] Restart and search
- [ ] Verify only 5 results show

### 10. Error Handling

**Atuin not installed:**
- [ ] Uninstall Atuin temporarily
- [ ] Start Scarab
- [ ] Verify warning notification
- [ ] Press Ctrl+R
- [ ] Verify warning message
- [ ] Reinstall Atuin

**Empty history:**
- [ ] Clear Atuin history: `rm ~/.local/share/atuin/history.db`
- [ ] Press Ctrl+R
- [ ] Verify "No results found" message

**Network issues (sync):**
- [ ] Disconnect network
- [ ] Try "Sync Atuin History" command
- [ ] Verify error notification
- [ ] Reconnect and retry
- [ ] Verify success notification

## Automated Test Scripts

### Test 1: Atuin Detection

```bash
#!/bin/bash
# test_atuin_detection.sh

echo "Testing Atuin detection..."

# Check if atuin is installed
if command -v atuin &> /dev/null; then
    echo "✓ Atuin is installed"
    atuin --version
else
    echo "✗ Atuin not found"
    exit 1
fi

# Check if atuin is initialized
if [ -f "$HOME/.local/share/atuin/history.db" ]; then
    echo "✓ Atuin is initialized"
else
    echo "✗ Atuin not initialized"
    exit 1
fi

# Check history count
count=$(atuin search "" --limit 1000 | wc -l)
echo "✓ History entries: $count"

exit 0
```

### Test 2: JSON Output Parsing

```bash
#!/bin/bash
# test_atuin_json.sh

echo "Testing Atuin JSON output..."

# Query with JSON format
output=$(atuin search --limit 5 --format json "")

# Check if output is valid JSON
if echo "$output" | jq . &> /dev/null; then
    echo "✓ Valid JSON output"
    echo "$output" | jq '.[0].command' 2>/dev/null
else
    echo "✗ Invalid JSON output"
    exit 1
fi

exit 0
```

### Test 3: Search Performance

```bash
#!/bin/bash
# test_search_performance.sh

echo "Testing search performance..."

# Measure search time
start=$(date +%s%N)
atuin search --limit 20 "git" > /dev/null
end=$(date +%s%N)

# Calculate duration in milliseconds
duration=$(( (end - start) / 1000000 ))

echo "Search time: ${duration}ms"

if [ $duration -lt 500 ]; then
    echo "✓ Search performance acceptable"
    exit 0
else
    echo "✗ Search too slow"
    exit 1
fi
```

### Test 4: Sync Functionality

```bash
#!/bin/bash
# test_atuin_sync.sh

echo "Testing Atuin sync..."

# Check if logged in
if atuin status | grep -q "logged in"; then
    echo "✓ Logged in to Atuin"

    # Try sync
    if atuin sync; then
        echo "✓ Sync successful"
        exit 0
    else
        echo "✗ Sync failed"
        exit 1
    fi
else
    echo "⚠ Not logged in (sync skipped)"
    exit 0
fi
```

## Integration Tests

### Test 5: Full Workflow

```bash
#!/bin/bash
# test_full_workflow.sh

set -e

echo "=== Atuin Plugin Integration Test ==="

# 1. Check prerequisites
echo "1. Checking prerequisites..."
command -v atuin >/dev/null || { echo "Atuin not installed"; exit 1; }
[ -f "$HOME/.local/share/atuin/history.db" ] || { echo "Atuin not initialized"; exit 1; }

# 2. Add test commands to history
echo "2. Adding test commands..."
test_cmd="echo 'atuin-plugin-test-$(date +%s)'"
eval "$test_cmd"
sleep 1

# 3. Search for test command
echo "3. Searching for test command..."
if atuin search "atuin-plugin-test" | grep -q "atuin-plugin-test"; then
    echo "✓ Test command found in history"
else
    echo "✗ Test command not found"
    exit 1
fi

# 4. Test JSON output
echo "4. Testing JSON output..."
json=$(atuin search --limit 1 --format json "atuin-plugin-test")
if echo "$json" | jq -e '.[0].command' >/dev/null; then
    echo "✓ JSON parsing works"
else
    echo "✗ JSON parsing failed"
    exit 1
fi

# 5. Test statistics
echo "5. Testing statistics..."
if atuin stats >/dev/null; then
    echo "✓ Stats command works"
else
    echo "✗ Stats command failed"
    exit 1
fi

echo ""
echo "=== All tests passed! ==="
exit 0
```

## Performance Benchmarks

Expected performance metrics:

| Operation | Target | Acceptable |
|-----------|--------|------------|
| Search (empty query) | < 100ms | < 200ms |
| Search (filtered) | < 150ms | < 300ms |
| Modal open | < 50ms | < 100ms |
| Navigation | < 10ms | < 50ms |
| Selection | < 50ms | < 100ms |
| Sync | < 2s | < 5s |

## Edge Cases

1. **Very long commands** (> 1000 chars)
   - Verify truncation or scrolling works

2. **Special characters** in commands
   - Test with quotes, backticks, pipes

3. **Rapid keypresses**
   - Test typing quickly in search

4. **Large history** (> 100k entries)
   - Verify search remains fast

5. **Concurrent sessions**
   - Test with multiple Scarab instances

## Regression Testing

When making changes, verify:

- [ ] Existing shortcuts still work
- [ ] Configuration still loads
- [ ] No memory leaks (check with `htop` during extended use)
- [ ] No CPU spikes during search
- [ ] Logs don't show new errors
- [ ] Command palette integration intact

## Reporting Issues

When filing bug reports, include:

1. Scarab version: `scarab --version`
2. Atuin version: `atuin --version`
3. Plugin config: `cat ~/.config/scarab/plugins/atuin.toml`
4. Relevant logs: `scarab --log-level debug`
5. Steps to reproduce
6. Expected vs actual behavior
7. Screenshots if applicable

## Clean Up

After testing:

```bash
# Remove test commands from history (optional)
# Note: Be careful with this!
# atuin search "atuin-plugin-test" --delete

# Reset configuration
rm ~/.config/scarab/plugins/atuin.toml
cp plugins/scarab-atuin/atuin.toml ~/.config/scarab/plugins/
```
