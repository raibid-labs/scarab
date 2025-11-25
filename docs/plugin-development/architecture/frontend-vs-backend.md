# Frontend vs Backend Plugins: The Complete Guide

This is the **most important** decision when creating a Scarab plugin: should it run in the frontend (.fsx) or backend (.fzb)?

This guide provides a comprehensive decision framework with performance data, use case analysis, and real-world examples.

## TL;DR Decision Tree

```
Need to process terminal output/input?
├─ YES → Backend (.fzb)
└─ NO → Frontend (.fsx)

Need to show custom UI?
├─ YES → Frontend (.fsx)
└─ NO → Could be either

Performance critical? (>1000 ops/sec)
├─ YES → Backend (.fzb)
└─ NO → Frontend (.fsx)

Need hot reload during development?
├─ YES → Frontend (.fsx)
└─ NO → Either
```

## The Two Runtimes

### Frontend (.fsx) - Client Side

**Runtime:** Fusabi Frontend (interpreter) running in Bevy client process

**Execution Model:**
- F# source code parsed and interpreted at runtime
- Runs on Bevy main thread (single-threaded)
- Has direct access to Bevy ECS and UI systems
- No compilation step required

**Performance Profile:**
- Startup: 5-20ms (parse + type check)
- Function call overhead: ~100ns
- Frame budget: 16ms @ 60fps
- Memory: ~2-5MB per plugin

**Access:**
- ✅ RemoteUI components (overlays, modals, notifications)
- ✅ Keyboard/mouse events
- ✅ Client-side state
- ✅ Bevy ECS systems
- ❌ Terminal output stream
- ❌ PTY manipulation
- ❌ Direct daemon state

### Backend (.fzb) - Daemon Side

**Runtime:** Fusabi VM (bytecode VM) running in daemon process

**Execution Model:**
- F# source compiled to bytecode ahead of time
- Runs in isolated VM instance per plugin
- Async execution with tokio runtime
- JIT optimization on hot paths

**Performance Profile:**
- Startup: 50-100ms (load bytecode + JIT warmup)
- Function call overhead: ~50ns
- Throughput: 100k+ lines/sec
- Memory: ~10-20MB per plugin (VM overhead)

**Access:**
- ✅ Terminal output stream (every line)
- ✅ Terminal input stream
- ✅ PTY manipulation
- ✅ Daemon state access
- ❌ Direct UI rendering
- ✅ RemoteUI via IPC (commands sent to client)

## Detailed Comparison

### When to Use Frontend (.fsx)

#### Strengths

1. **Hot Reload** - Instant feedback during development
   ```bash
   just dev-mode my-plugin  # Edit, save, see changes immediately
   ```

2. **UI-First** - Direct access to RemoteUI components
   ```fsharp
   ctx.ShowModal "Command Palette" commands  // Instant UI
   ```

3. **Lower Latency for UI** - No IPC roundtrip
   ```
   KeyPress → Frontend Plugin → UI Update
   (< 1ms latency)
   ```

4. **Simpler Debugging** - Source maps, stack traces
   ```
   Error at hello-notification.fsx:23
   ```

5. **Easier Distribution** - Just F# source, no build step

#### Weaknesses

1. **No Terminal Output Access** - Cannot process PTY stream
   ```fsharp
   // ❌ Cannot do this in frontend
   [<OnOutput>]
   let onOutput line = detectGitCommands line
   ```

2. **Single-Threaded** - Competes with rendering
   ```
   Long-running plugin → Dropped frames → Jank
   ```

3. **Limited Performance** - Interpreter overhead
   ```
   Regex on 1000 lines: ~100ms (too slow for real-time)
   ```

4. **Client-Only** - Plugin unloaded when client disconnects

#### Best Use Cases

✅ **Keyboard shortcuts**
```fsharp
[<OnKeyPress>]
let onKeyPress ctx key =
    if key.Code = KeyCode.P && key.Ctrl then
        ctx.ShowCommandPalette()
```

✅ **UI overlays and modals**
```fsharp
[<OnLoad>]
let onLoad ctx =
    ctx.ShowOverlay "Git Status" gitInfo (x=10, y=1)
```

✅ **Custom commands**
```fsharp
[<Command("search-history")>]
let searchHistory ctx query =
    let results = searchTerminalScrollback query
    ctx.ShowModal "Search Results" results
```

✅ **Theme extensions**
```fsharp
[<Command("theme-picker")>]
let themePicker ctx =
    let themes = loadAvailableThemes()
    ctx.ShowThemePicker themes
```

✅ **Notifications and alerts**
```fsharp
ctx.Notify "Build Complete" "Deployment successful!" Success
```

### When to Use Backend (.fzb)

#### Strengths

1. **Terminal Output Processing** - Every line, minimal overhead
   ```fsharp
   [<OnOutput>]
   let onOutput ctx line =
       if line.Contains "error" then
           ctx.Notify "Error Detected" line Warning
   ```

2. **High Performance** - Compiled, JIT-optimized
   ```
   Process 100k lines/sec with regex matching
   ```

3. **PTY Access** - Direct terminal manipulation
   ```fsharp
   ctx.WriteToPty "\x1b[1;32m✓\x1b[0m"  // Inject ANSI codes
   ```

4. **Persistence** - Survives client disconnects
   ```
   Daemon keeps running → Plugin keeps working
   ```

5. **Background Processing** - No UI thread contention
   ```fsharp
   let! data = fetchFromApi()  // Won't block rendering
   ```

#### Weaknesses

1. **No Hot Reload** - Must rebuild and restart
   ```bash
   just plugin-build my-plugin  # Recompile
   just kill && just run-bg     # Restart daemon
   ```

2. **Compilation Required** - Build step adds friction
   ```
   Edit → Compile → Test cycle (vs Edit → Test)
   ```

3. **UI via IPC** - Extra latency for notifications
   ```
   Backend Plugin → IPC → Client → UI
   (~5-10ms overhead)
   ```

4. **Harder Debugging** - Bytecode, no source maps
   ```
   Stack traces reference bytecode offsets, not source lines
   ```

5. **Larger Memory Footprint** - VM overhead

#### Best Use Cases

✅ **Output scanning and pattern detection**
```fsharp
[<OnOutput>]
let onOutput ctx line =
    if containsUrl line then
        let urls = extractUrls line
        ctx.HighlightUrls urls
```

✅ **Git command detection**
```fsharp
[<OnOutput>]
let onOutput ctx line =
    match line with
    | GitCommit hash -> trackCommit hash
    | GitBranch name -> updateBranchIndicator name
    | _ -> ()
```

✅ **Command timing and profiling**
```fsharp
[<OnPreCommand>]
let onPreCommand ctx cmd =
    ctx.SetData "start_time" (DateTime.Now.ToString())

[<OnPostCommand>]
let onPostCommand ctx cmd exitCode =
    let duration = computeDuration ctx
    if duration > 5000 then
        ctx.Notify "Slow Command" (sprintf "%s took %dms" cmd duration) Warning
```

✅ **Log file parsing and colorization**
```fsharp
[<OnOutput>]
let onOutput ctx line =
    match parseLogLevel line with
    | Some ERROR -> ctx.ColorLine line Color.Red
    | Some WARN -> ctx.ColorLine line Color.Yellow
    | _ -> ()
```

✅ **External API integration**
```fsharp
[<OnOutput>]
let onOutput ctx line =
    if containsPullRequestNumber line then
        let! prInfo = fetchGitHubPR line
        ctx.Notify "PR Info" prInfo.title Info
```

## Hybrid Approach: Best of Both Worlds

Many plugins benefit from **both** runtimes working together:

### Example: Atuin Integration

**Backend (.fzb):**
```fsharp
// Detect Ctrl+R in daemon
[<OnInput>]
let onInput ctx input =
    if input = "\x12" then  // Ctrl+R
        let! history = queryAtuinHistory()
        ctx.SendToFrontend "show-history" history
        Stop  // Block Ctrl+R from reaching shell
    else
        Continue
```

**Frontend (.fsx):**
```fsharp
// Render search UI in client
[<OnRemoteCommand("show-history")>]
let showHistory ctx history =
    ctx.ShowSearchableModal "Command History" history
        (onSelect = fun cmd -> ctx.SendToBackend "execute" cmd)
```

### Example: Git Status Bar

**Backend (.fzb):**
```fsharp
// Parse git commands and repo state
[<OnOutput>]
let onOutput ctx line =
    match parseGitCommand line with
    | Some (GitCheckout branch) ->
        ctx.SendToFrontend "update-branch" branch
    | Some (GitStatus status) ->
        ctx.SendToFrontend "update-status" status
    | _ -> ()
    Continue
```

**Frontend (.fsx):**
```fsharp
// Render status bar widget
[<OnRemoteCommand("update-branch")>]
let updateBranch ctx branch =
    ctx.ShowStatusBarItem "git-branch" (sprintf "⎇ %s" branch)

[<OnRemoteCommand("update-status")>]
let updateStatus ctx status =
    let color = if status.clean then Green else Yellow
    ctx.ShowStatusBarItem "git-status" status.summary color
```

## Communication Between Runtimes

### Backend → Frontend

```fsharp
// In backend (.fzb)
ctx.QueueCommand (RemoteCommand.PluginNotify {
    title = "URL Detected"
    body = url
    level = Info
})

// Or use convenience method
ctx.Notify "URL Detected" url Info
```

### Frontend → Backend

```fsharp
// In frontend (.fsx)
ctx.SendInput "git status\n"  // Send to PTY via daemon
```

**IPC Latency:** ~2-5ms typical, ~10-20ms p99

## Performance Benchmarks

### Output Processing (10,000 lines)

| Task | Frontend | Backend | Winner |
|------|----------|---------|--------|
| Simple regex | 450ms | 45ms | Backend (10x) |
| JSON parsing | 890ms | 120ms | Backend (7x) |
| URL detection | 320ms | 38ms | Backend (8x) |
| Simple filter | 180ms | 22ms | Backend (8x) |

### UI Operations

| Task | Frontend | Backend | Winner |
|------|----------|---------|--------|
| Show notification | 0.5ms | 8ms | Frontend (16x) |
| Update overlay | 0.3ms | 12ms | Frontend (40x) |
| Show modal | 1ms | 15ms | Frontend (15x) |
| Handle keypress | 0.1ms | N/A | Frontend (only) |

### Memory Usage (per plugin)

| Runtime | Baseline | After 1hr | After 24hr |
|---------|----------|-----------|------------|
| Frontend | 2.1 MB | 2.8 MB | 3.2 MB |
| Backend | 12.5 MB | 13.1 MB | 14.8 MB |

## Decision Matrix

| Requirement | Frontend | Backend | Notes |
|-------------|----------|---------|-------|
| Process output | ❌ | ✅ | Backend only |
| Custom UI | ✅ | ⚠️ | Frontend direct, backend via IPC |
| Keyboard shortcuts | ✅ | ❌ | Frontend only |
| Hot reload | ✅ | ❌ | Development speed |
| High throughput | ❌ | ✅ | 10x faster |
| Low latency UI | ✅ | ❌ | 16x faster |
| PTY access | ❌ | ✅ | Backend only |
| Persistence | ❌ | ✅ | Survives client disconnect |
| Small memory | ✅ | ❌ | Frontend 6x smaller |
| External APIs | ⚠️ | ✅ | Both work, backend better |

## Real-World Examples

### Frontend Plugins

1. **Command Palette** - Fuzzy search commands
2. **Theme Switcher** - Live preview themes
3. **Clipboard History** - Visual clipboard manager
4. **Quick Notes** - Scratchpad overlay
5. **Window Manager** - Split/tab management

### Backend Plugins

1. **URL Detector** - Highlight and track URLs
2. **Git Integration** - Parse git commands and output
3. **Command Timer** - Measure command duration
4. **Log Colorizer** - Parse and color log levels
5. **Auto-CD** - Suggest directory navigation

### Hybrid Plugins

1. **Atuin History** - Backend search + Frontend UI
2. **Status Bar** - Backend state + Frontend display
3. **Error Monitor** - Backend detection + Frontend alerts
4. **File Watcher** - Backend inotify + Frontend notifications

## Choosing Wisely

### Start with Frontend if:
- You're prototyping
- UI is the primary feature
- Performance isn't critical
- You want fast iteration

### Start with Backend if:
- You need terminal output
- Processing is performance-critical
- You're building a production tool
- You need daemon persistence

### Use Both if:
- Complex state management needed
- High-performance processing + rich UI
- You're building a substantial feature
- Multiple interaction points required

## Migration Path

### Frontend → Backend

If your frontend plugin becomes too slow:

1. **Profile first** - Measure where time is spent
2. **Extract hot path** - Move slow processing to backend
3. **Keep UI in frontend** - Only migrate processing
4. **Use IPC** - Backend sends events to frontend

### Backend → Frontend

If you add UI to a backend plugin:

1. **Keep output processing in backend**
2. **Add frontend plugin** for UI
3. **Use RemoteCommand** for communication
4. **Register both plugins** in config

## Best Practices

### Frontend Plugins

1. **Stay under 16ms** per frame - Avoid long operations
2. **Batch UI updates** - Don't update on every keystroke
3. **Use async** for any I/O - Don't block the render thread
4. **Minimize allocations** - Reuse objects when possible
5. **Test at 144Hz** - Some users have high refresh rates

### Backend Plugins

1. **Return early** - Don't process every line if possible
2. **Compile regex** - Use `RegexOptions.Compiled`
3. **Limit notifications** - Don't spam the user
4. **Use async properly** - Leverage tokio
5. **Profile with Tracy** - Measure actual performance

## Conclusion

The frontend vs backend decision is about **matching capabilities to requirements**:

- **Frontend** = UI-first, fast iteration, lower performance
- **Backend** = Processing-first, high performance, slower iteration
- **Hybrid** = Complex plugins, best of both worlds

When in doubt:
1. Start with frontend (easier to iterate)
2. Profile performance
3. Move hot paths to backend if needed
4. Most plugins don't need optimization

Happy plugin building!
