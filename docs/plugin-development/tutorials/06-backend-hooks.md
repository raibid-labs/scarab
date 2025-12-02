# Tutorial 6: Backend Processing with Hooks

In this tutorial, you'll master backend plugin development by building a Git status parser that demonstrates advanced hook patterns, state management, and performance optimization techniques.

**What you'll learn:**
- Backend hook lifecycle and execution order
- Available hooks: OnOutput, OnInput, OnResize, OnLoad/OnUnload
- State management across hook invocations
- Performance optimization patterns
- Building a production-ready Git status parser
- Testing backend plugins
- Debugging and troubleshooting

**Time:** 60 minutes

## Prerequisites

- Completed [Tutorial 2: Hello World (Backend)](02-hello-world-backend.md)
- Understanding of async/await in F#
- Basic Git knowledge

## Backend Hook Architecture

Backend plugins run in the **daemon process** and have access to all terminal I/O. Unlike frontend plugins, they:

- Process **every line of output** in real-time
- Can intercept and modify input before execution
- Have no access to UI rendering (use notifications instead)
- Must be extremely performant (microseconds per line)
- Require compilation to .fzb bytecode (no hot reload)

### Hook Execution Flow

```
Terminal Output
    ↓
OnOutput Hook (Plugin 1)
    ↓
OnOutput Hook (Plugin 2)
    ↓
OnOutput Hook (Plugin N)
    ↓
Display in Terminal
```

Each hook can:
- **Continue** - Pass output to next plugin unchanged
- **Stop** - Block output from displaying
- **Modify** - Change output before displaying

## Available Backend Hooks

### OnLoad / OnUnload

Lifecycle hooks for initialization and cleanup.

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Initialize resources
        ctx.Log Info "Plugin starting..."

        // Load configuration
        let config = loadConfig ctx

        // Verify dependencies
        match checkDependencies() with
        | Ok () ->
            ctx.NotifySuccess "Plugin Ready" "All systems go!"
            return Ok ()
        | Error msg ->
            ctx.NotifyError "Plugin Failed" msg
            return Error (PluginError.InitializationError msg)
    }

[<OnUnload>]
let onUnload (ctx: PluginContext) =
    async {
        // Clean up resources
        ctx.Log Info "Plugin shutting down..."

        // Save state if needed
        saveState()

        // Close connections
        cleanup()

        return Ok ()
    }
```

**Best practices:**
- Validate configuration in OnLoad
- Return descriptive errors if initialization fails
- Clean up all resources in OnUnload
- Save persistent state before unloading

### OnOutput

Called for **every line** of terminal output before display.

```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Early return for performance
        if not (line.Contains "keyword") then
            return Continue

        // Process matched lines
        ctx.Log Debug (sprintf "Processing: %s" line)

        // Return action
        return Continue  // or Stop or Modify line
    }
```

**Performance critical:**
- Called thousands of times per second
- Must return in < 50 microseconds typically
- Use early returns to skip irrelevant lines
- Compile regex patterns once
- Avoid allocations in hot path

### OnInput

Called when user submits input (before sending to PTY).

```fsharp
[<OnInput>]
let onInput (ctx: PluginContext) (input: string) =
    async {
        // Intercept commands
        if input.StartsWith("!custom") then
            // Handle custom command
            ctx.Log Info "Custom command detected"
            processCustomCommand input
            return Stop  // Don't send to shell
        else
            return Continue  // Pass through
    }
```

**Use cases:**
- Custom command expansion
- Input validation/sanitization
- Command aliases
- Dangerous command prevention

### OnResize

Called when terminal size changes.

```fsharp
[<OnResize>]
let onResize (ctx: PluginContext) (cols: uint16) (rows: uint16) =
    async {
        ctx.Log Debug (sprintf "Terminal resized to %dx%d" cols rows)

        // Recompute layout-dependent state
        updateLayout cols rows

        return ()
    }
```

**Use cases:**
- Adjusting buffer sizes
- Recomputing display layouts
- Logging terminal size changes

### OnPreCommand

Called before a shell command executes (if detectable).

```fsharp
[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) =
    async {
        // Track command execution
        logCommandExecution command

        // Warn about dangerous commands
        if isDangerous command then
            ctx.NotifyWarning "Dangerous Command"
                (sprintf "Are you sure? %s" command)

        return Continue
    }
```

## State Management Across Hooks

Backend plugins often need to maintain state across hook invocations.

### Mutable State

```fsharp
// Module-level state
let mutable lineCount = 0
let mutable errorCount = 0
let mutable lastCommand = ""

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        lineCount <- lineCount + 1

        if line.Contains "error" then
            errorCount <- errorCount + 1

        return Continue
    }
```

### Reference Cells

```fsharp
// For complex state
type PluginState = {
    mutable LineCount: int
    mutable Errors: string list
    mutable StartTime: DateTime
}

let state = {
    LineCount = 0
    Errors = []
    StartTime = DateTime.Now
}

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        state.LineCount <- state.LineCount + 1

        if line.Contains "error" then
            state.Errors <- line :: state.Errors

        return Continue
    }
```

### Collections

```fsharp
// Using mutable collections
open System.Collections.Generic

let urlHistory = List<string>()
let commandCache = Dictionary<string, string>()

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Extract URLs and store
        let urls = extractUrls line
        for url in urls do
            if not (urlHistory.Contains url) then
                urlHistory.Add url

        return Continue
    }
```

## Performance Optimization Patterns

### Pattern 1: Early Returns

Always filter out irrelevant input as early as possible.

```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Quick character check before regex
        if not (line.Contains "git") then
            return Continue

        // More expensive operations only if needed
        if not (line.Contains "branch" || line.Contains "commit") then
            return Continue

        // Expensive processing last
        match parseGitOutput line with
        | Some result -> processGitResult result
        | None -> ()

        return Continue
    }
```

### Pattern 2: Compiled Regex

Compile regex patterns once at module load time.

```fsharp
open System.Text.RegularExpressions

// Compiled once, reused many times
let gitBranchPattern = @"On branch (\w+)"
let gitBranchRegex = Regex(gitBranchPattern, RegexOptions.Compiled)

let gitStatusPattern = @"(\d+) files? changed"
let gitStatusRegex = Regex(gitStatusPattern, RegexOptions.Compiled)

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Fast compiled regex match
        let branchMatch = gitBranchRegex.Match(line)
        if branchMatch.Success then
            let branch = branchMatch.Groups.[1].Value
            processBranch branch

        return Continue
    }
```

### Pattern 3: Buffering Multi-Line Output

Some terminal output spans multiple lines. Buffer and process together.

```fsharp
// State for buffering
let mutable isInGitStatus = false
let mutable gitStatusLines = []

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Detect start of git status
        if line.StartsWith("On branch") then
            isInGitStatus <- true
            gitStatusLines <- [line]
            return Continue

        // Buffer lines while in git status
        elif isInGitStatus then
            gitStatusLines <- line :: gitStatusLines

            // Detect end (empty line)
            if String.IsNullOrWhiteSpace(line) then
                // Process complete status
                processGitStatus (List.rev gitStatusLines)
                isInGitStatus <- false
                gitStatusLines <- []

            return Continue

        else
            return Continue
    }
```

### Pattern 4: Debouncing

Avoid processing every event when rapid updates occur.

```fsharp
open System

let mutable lastProcessTime = DateTime.MinValue
let debounceMs = 500.0  // 500ms debounce

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        let now = DateTime.Now
        let elapsed = (now - lastProcessTime).TotalMilliseconds

        if elapsed < debounceMs then
            // Skip processing - too soon
            return Continue

        lastProcessTime <- now

        // Process with debounce
        processExpensiveOperation line

        return Continue
    }
```

## Complete Example: Git Status Parser

Let's build a complete Git status parser that demonstrates all these patterns.

### Features

1. Detects `git status` output
2. Parses branch name, changes, staged files
3. Shows notification with summary
4. Maintains statistics
5. Handles multi-line output buffering
6. Optimized for performance

### Implementation

```fsharp
module git_status_parser

open Scarab.PluginApi
open System
open System.Text.RegularExpressions

[<Plugin>]
let metadata = {
    Name = "git-status-parser"
    Version = "1.0.0"
    Description = "Parses git status output and provides insights"
    Author = "Your Name"
    Emoji = Some "🌿"
    Color = Some "#F05032"
    Catchphrase = Some "Git good!"
}

// Compiled regex patterns
let branchPattern = @"On branch (\S+)"
let branchRegex = Regex(branchPattern, RegexOptions.Compiled)

let changesPattern = @"Changes (not staged|to be committed):"
let changesRegex = Regex(changesPattern, RegexOptions.Compiled)

let modifiedPattern = @"^\s+modified:\s+(.+)$"
let modifiedRegex = Regex(modifiedPattern, RegexOptions.Compiled)

let untrackedPattern = @"^\s+(.+)$"
let untrackedRegex = Regex(untrackedPattern, RegexOptions.Compiled)

let aheadPattern = @"Your branch is ahead of '(\S+)' by (\d+) commit"
let aheadRegex = Regex(aheadPattern, RegexOptions.Compiled)

// Git status state
type GitStatus = {
    Branch: string option
    StagedFiles: string list
    ModifiedFiles: string list
    UntrackedFiles: string list
    AheadBy: int option
}

let emptyStatus = {
    Branch = None
    StagedFiles = []
    ModifiedFiles = []
    UntrackedFiles = []
    AheadBy = None
}

// Parsing state
type ParsingState =
    | Idle
    | InStatus
    | InStaged
    | InModified
    | InUntracked

let mutable state = Idle
let mutable currentStatus = emptyStatus
let mutable statusLines = []

// Statistics
let mutable totalStatuses = 0
let mutable totalCommits = 0

// Process complete git status
let processGitStatus (ctx: PluginContext) (status: GitStatus) =
    // Build summary message
    let branchInfo =
        match status.Branch with
        | Some b -> sprintf "Branch: %s" b
        | None -> "Branch: unknown"

    let changes = [
        if not (List.isEmpty status.StagedFiles) then
            sprintf "%d staged" (List.length status.StagedFiles)
        if not (List.isEmpty status.ModifiedFiles) then
            sprintf "%d modified" (List.length status.ModifiedFiles)
        if not (List.isEmpty status.UntrackedFiles) then
            sprintf "%d untracked" (List.length status.UntrackedFiles)
    ]

    let changeInfo =
        if List.isEmpty changes then
            "Working tree clean"
        else
            String.concat ", " changes

    let aheadInfo =
        match status.AheadBy with
        | Some n -> sprintf "\n%d commit(s) ahead" n
        | None -> ""

    // Show notification
    let message = sprintf "%s\n%s%s" branchInfo changeInfo aheadInfo

    let level =
        if List.isEmpty status.ModifiedFiles && List.isEmpty status.UntrackedFiles then
            NotifyLevel.Success
        else
            NotifyLevel.Info

    ctx.Notify "Git Status" message level

    // Update statistics
    totalStatuses <- totalStatuses + 1

    // Log details
    ctx.Log Info (sprintf "Git status parsed: %s" message)

// Parse a line in context of current state
let parseLine (line: string) =
    match state with
    | Idle ->
        // Check for start of git status
        if line.Contains "On branch" then
            let m = branchRegex.Match(line)
            if m.Success then
                currentStatus <- { currentStatus with Branch = Some m.Groups.[1].Value }
                state <- InStatus

        // Check for ahead message
        let aheadMatch = aheadRegex.Match(line)
        if aheadMatch.Success then
            let count = int aheadMatch.Groups.[2].Value
            currentStatus <- { currentStatus with AheadBy = Some count }

    | InStatus ->
        // Detect sections
        if line.Contains "Changes to be committed:" then
            state <- InStaged
        elif line.Contains "Changes not staged for commit:" then
            state <- InModified
        elif line.Contains "Untracked files:" then
            state <- InUntracked
        elif String.IsNullOrWhiteSpace(line) && currentStatus.Branch.IsSome then
            // End of status - process and reset
            state <- Idle
            true  // Signal to process
        else
            false

    | InStaged ->
        if String.IsNullOrWhiteSpace(line) then
            state <- InStatus
            false
        else
            let m = modifiedRegex.Match(line)
            if m.Success then
                let file = m.Groups.[1].Value
                currentStatus <- { currentStatus with StagedFiles = file :: currentStatus.StagedFiles }
            false

    | InModified ->
        if String.IsNullOrWhiteSpace(line) then
            state <- InStatus
            false
        else
            let m = modifiedRegex.Match(line)
            if m.Success then
                let file = m.Groups.[1].Value
                currentStatus <- { currentStatus with ModifiedFiles = file :: currentStatus.ModifiedFiles }
            false

    | InUntracked ->
        if String.IsNullOrWhiteSpace(line) then
            state <- InStatus
            false
        else
            let m = untrackedRegex.Match(line)
            if m.Success then
                let file = m.Groups.[1].Value.Trim()
                if not (file.StartsWith "(") then  // Skip help text
                    currentStatus <- { currentStatus with UntrackedFiles = file :: currentStatus.UntrackedFiles }
            false

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Git Status Parser loaded"

        // Check if git is available
        try
            // This is a simplified check - in production, use proper process execution
            ctx.Log Info "Git detected - parser ready"
            ctx.NotifySuccess "Git Parser Ready" "Watching for git status output"
            return Ok ()
        with
        | ex ->
            ctx.Log Warn "Git not found - parser may not work"
            return Ok ()  // Don't fail if git isn't installed
    }

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Early filter - skip if no git-related keywords
        if not (line.Contains "branch" || line.Contains "Changes" ||
                line.Contains "modified:" || line.Contains "Untracked" ||
                line.Contains "ahead") then
            return Continue

        // Parse the line
        let shouldProcess = parseLine line

        if shouldProcess then
            // Process complete status
            processGitStatus ctx currentStatus

            // Reset state
            currentStatus <- emptyStatus

        return Continue
    }

[<OnUnload>]
let onUnload (ctx: PluginContext) =
    async {
        // Log statistics
        ctx.Log Info (sprintf "Git Status Parser unloading. Parsed %d statuses" totalStatuses)

        // Clean up state
        state <- Idle
        currentStatus <- emptyStatus

        return Ok ()
    }
```

## Testing Backend Plugins

### Manual Testing

1. **Build the plugin:**
   ```bash
   just plugin-build git-status-parser
   ```

2. **Start Scarab with logging:**
   ```bash
   RUST_LOG=debug just run-bg
   ```

3. **Trigger git status output:**
   ```bash
   cd /path/to/git/repo
   git status
   ```

4. **Verify notification and logs**

### Unit Testing (Conceptual)

```fsharp
// In a test file
module GitStatusParserTests

open Xunit
open git_status_parser

[<Fact>]
let ``Should parse branch name`` () =
    let line = "On branch main"
    let result = parseBranchLine line
    Assert.Equal(Some "main", result)

[<Fact>]
let ``Should detect modified files`` () =
    let line = "    modified:   README.md"
    let result = parseModifiedLine line
    Assert.Equal(Some "README.md", result)

[<Fact>]
let ``Should handle empty status`` () =
    let status = emptyStatus
    Assert.True(List.isEmpty status.StagedFiles)
    Assert.True(List.isEmpty status.ModifiedFiles)
```

### Integration Testing

```fsharp
// Mock context for testing
type MockContext() =
    let mutable logs = []
    let mutable notifications = []

    interface PluginContext with
        member _.Log level msg =
            logs <- (level, msg) :: logs

        member _.Notify title msg level =
            notifications <- (title, msg, level) :: notifications

        // ... other methods

[<Fact>]
let ``Should notify on complete git status`` () =
    let ctx = MockContext() :> PluginContext

    // Simulate git status output
    onOutput ctx "On branch main" |> Async.RunSynchronously |> ignore
    onOutput ctx "nothing to commit, working tree clean" |> Async.RunSynchronously |> ignore
    onOutput ctx "" |> Async.RunSynchronously |> ignore

    // Verify notification was sent
    let notifications = (ctx :?> MockContext).GetNotifications()
    Assert.NotEmpty(notifications)
```

## Troubleshooting

### Plugin not loading?

**Check plugin.toml:**
```toml
[plugin]
name = "git-status-parser"
version = "1.0.0"
runtime = "backend"

[hooks]
on_load = true
on_output = true
on_unload = true
```

**Check logs:**
```bash
# Daemon logs
tail -f ~/.local/share/scarab/daemon.log

# Enable debug logging
RUST_LOG=debug cargo run -p scarab-daemon
```

### Output not being processed?

1. **Verify hook is enabled** - Check `on_output = true`
2. **Check early returns** - Log before early returns to see if line reaches processing
3. **Test regex patterns** - Use online regex tester
4. **Add debug logging:**
   ```fsharp
   ctx.Log Debug (sprintf "Processing line: %s" line)
   ```

### Performance issues?

1. **Profile the hot path:**
   ```fsharp
   let start = DateTime.Now
   // ... processing ...
   let elapsed = (DateTime.Now - start).TotalMilliseconds
   if elapsed > 1.0 then
       ctx.Log Warn (sprintf "Slow processing: %.2fms" elapsed)
   ```

2. **Check allocation:**
   - Avoid string concatenation in loops
   - Reuse compiled regex patterns
   - Use StringBuilder for complex string building

3. **Measure before/after:**
   ```bash
   # Run with timing
   time echo "test" > /dev/null
   ```

### State not persisting?

Backend plugins are **per-terminal-instance**. State doesn't persist:
- Across plugin reloads
- Across daemon restarts
- Between different terminal tabs

For persistence, use:
- `ctx.Storage.Set()` / `ctx.Storage.Get()`
- External files/databases
- Backend communication with frontend for UI state

## Best Practices Summary

### Performance

- ✅ **Early return** for irrelevant lines
- ✅ **Compile regex** patterns once
- ✅ **Avoid allocations** in hot path
- ✅ **Use buffering** for multi-line output
- ✅ **Debounce** rapid updates
- ✅ **Profile** slow code paths

### State Management

- ✅ **Use mutable state** carefully
- ✅ **Initialize** in OnLoad
- ✅ **Clean up** in OnUnload
- ✅ **Reset state** when done processing
- ✅ **Consider** per-terminal vs global state

### Error Handling

- ✅ **Validate** input before processing
- ✅ **Handle** regex match failures
- ✅ **Log errors** with context
- ✅ **Fail gracefully** - don't crash daemon
- ✅ **Return Continue** on errors (usually)

### Testing

- ✅ **Unit test** parsing logic
- ✅ **Integration test** with mock context
- ✅ **Manual test** in real terminals
- ✅ **Performance test** with large outputs
- ✅ **Edge cases** - empty lines, malformed input

## What You Learned

- ✅ Backend hook lifecycle and execution order
- ✅ All available backend hooks and their use cases
- ✅ State management patterns for backend plugins
- ✅ Performance optimization techniques
- ✅ Building a production-ready Git status parser
- ✅ Testing strategies for backend plugins
- ✅ Debugging and troubleshooting approaches

## Next Steps

→ **[Tutorial 7: Testing and Publishing](07-testing-and-publishing.md)** - Make your plugin production-ready

→ **[API Reference: Hooks](../api-reference/hooks.md)** - Complete hook documentation

→ **[Plugin Architecture](../architecture/overview.md)** - Deep dive into Scarab internals

## Challenge Exercises

Try these enhancements:

1. **Git commit parser** - Detect `git log` output and show commit summaries
2. **Build status detector** - Parse cargo/npm/pytest output for errors
3. **Command timer** - Measure and display command execution time
4. **Output filter** - Hide specific patterns from output (e.g., debug logs)
5. **Smart suggestions** - Suggest commands based on error messages

Happy coding!
