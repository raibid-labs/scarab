# Tutorial 7: Testing and Publishing

In this tutorial, you'll learn how to make your plugins production-ready through comprehensive testing, performance profiling, proper packaging, and publishing to the Scarab plugin registry.

**What you'll learn:**
- Unit testing with MockPluginContext
- Integration testing strategies
- Performance profiling and benchmarking
- Memory usage monitoring
- Packaging with plugin.toml
- Using `just plugin-package` workflow
- Publishing to the plugin registry
- Versioning strategy
- Production deployment checklist

**Time:** 60 minutes

## Prerequisites

- Completed [Tutorial 6: Backend Hooks](06-backend-hooks.md)
- A plugin ready for testing
- Understanding of software testing principles

## Part 1: Unit Testing

### Setting Up Test Infrastructure

Create a test directory for your plugin:

```bash
cd plugins/my-plugin
mkdir tests
touch tests/test_plugin.fsx
```

### MockPluginContext

Create a mock context for testing without running Scarab:

```fsharp
// tests/mock_context.fsx
module MockPluginContext

open Scarab.PluginApi
open System.Collections.Generic

type MockContext() =
    // Storage for assertions
    let logs = List<LogLevel * string>()
    let notifications = List<string * string * NotifyLevel>()
    let commands = List<RemoteCommand>()
    let storage = Dictionary<string, string>()

    // Mock configuration
    let config = Dictionary<string, string>()

    // Mock terminal state
    let mutable terminalSize = (80us, 24us)
    let mutable cursorPos = (0us, 0us)

    // Helper to set config values
    member this.SetConfig(key: string, value: string) =
        config.[key] <- value

    // Helper to get logged messages
    member this.GetLogs() = List.ofSeq logs
    member this.GetNotifications() = List.ofSeq notifications
    member this.GetCommands() = List.ofSeq commands

    // Clear all state
    member this.Clear() =
        logs.Clear()
        notifications.Clear()
        commands.Clear()

    interface PluginContext with
        member _.Log level msg =
            logs.Add((level, msg))

        member _.Notify title msg level =
            notifications.Add((title, msg, level))

        member _.NotifySuccess title msg =
            notifications.Add((title, msg, NotifyLevel.Success))

        member _.NotifyInfo title msg =
            notifications.Add((title, msg, NotifyLevel.Info))

        member _.NotifyWarning title msg =
            notifications.Add((title, msg, NotifyLevel.Warning))

        member _.NotifyError title msg =
            notifications.Add((title, msg, NotifyLevel.Error))

        member _.QueueCommand cmd =
            commands.Add(cmd)

        member _.GetCell x y =
            { Char = ' '; Fg = 0xFFFFFFu; Bg = 0x000000u }

        member _.GetLine y =
            String.replicate (int (fst terminalSize)) " "

        member _.GetSize() =
            terminalSize

        member _.SetSize cols rows =
            terminalSize <- (cols, rows)

        member _.GetCursor() =
            cursorPos

        member _.GetEnv key =
            match key with
            | "HOME" -> Some "/home/test"
            | "USER" -> Some "testuser"
            | _ -> None

        member _.Config
            with get() =
                { new PluginConfig with
                    member _.Get key =
                        match config.TryGetValue(key) with
                        | true, value -> value
                        | false, _ -> failwith (sprintf "Config key not found: %s" key)

                    member _.GetOpt key =
                        match config.TryGetValue(key) with
                        | true, value -> Some value
                        | false, _ -> None
                }

        member _.Storage
            with get() =
                { new PluginStorage with
                    member _.Get key =
                        match storage.TryGetValue(key) with
                        | true, value -> Some value
                        | false, _ -> None

                    member _.Set key value =
                        storage.[key] <- value

                    member _.Delete key =
                        storage.Remove(key) |> ignore

                    member _.Clear() =
                        storage.Clear()
                }

        member _.SendToFrontend msg =
            // Could track these if needed
            ()

        member _.SendToBackend msg =
            ()
```

### Writing Unit Tests

Create tests for your plugin's core logic:

```fsharp
// tests/test_plugin.fsx
module PluginTests

open Xunit
open MockPluginContext
open my_plugin  // Your plugin module

[<Fact>]
let ``OnLoad should initialize successfully`` () =
    let ctx = MockContext()

    // Execute OnLoad
    let result = onLoad (ctx :> PluginContext) |> Async.RunSynchronously

    // Assert success
    match result with
    | Ok () ->
        // Check that initialization logged
        let logs = ctx.GetLogs()
        Assert.NotEmpty(logs)

        // Check for success notification
        let notifications = ctx.GetNotifications()
        Assert.Contains(notifications, fun (_, _, level) -> level = NotifyLevel.Success)
    | Error err ->
        Assert.True(false, sprintf "OnLoad failed: %A" err)

[<Fact>]
let ``OnOutput should detect URLs`` () =
    let ctx = MockContext()

    // Test input
    let line = "Check out https://github.com/raibid-labs/scarab"

    // Execute OnOutput
    let result = onOutput (ctx :> PluginContext) line |> Async.RunSynchronously

    // Assert URL was detected
    let logs = ctx.GetLogs()
    Assert.Contains(logs, fun (level, msg) ->
        level = LogLevel.Info && msg.Contains("URL")
    )

    // Assert notification was sent
    let notifications = ctx.GetNotifications()
    Assert.NotEmpty(notifications)

[<Fact>]
let ``OnOutput should ignore localhost URLs`` () =
    let ctx = MockContext()

    let line = "Server running at http://localhost:3000"

    let result = onOutput (ctx :> PluginContext) line |> Async.RunSynchronously

    // Should not notify for localhost
    let notifications = ctx.GetNotifications()
    Assert.Empty(notifications)

[<Fact>]
let ``OnOutput should return Continue for normal lines`` () =
    let ctx = MockContext()

    let line = "normal output without special content"

    let result = onOutput (ctx :> PluginContext) line |> Async.RunSynchronously

    Assert.Equal(Continue, result)

[<Theory>]
[<InlineData("https://example.com")>]
[<InlineData("http://test.org")>]
[<InlineData("https://github.com/user/repo")>]
let ``Should detect various URL formats`` (url: string) =
    let ctx = MockContext()

    let result = onOutput (ctx :> PluginContext) url |> Async.RunSynchronously

    let logs = ctx.GetLogs()
    Assert.NotEmpty(logs)

[<Fact>]
let ``Configuration should be loaded correctly`` () =
    let ctx = MockContext()
    ctx.SetConfig("min_url_length", "50")

    // Load plugin with config
    let result = onLoad (ctx :> PluginContext) |> Async.RunSynchronously

    // Verify config was read
    let logs = ctx.GetLogs()
    Assert.Contains(logs, fun (_, msg) ->
        msg.Contains("min URL length = 50")
    )

[<Fact>]
let ``OnUnload should clean up resources`` () =
    let ctx = MockContext()

    // Initialize
    onLoad (ctx :> PluginContext) |> Async.RunSynchronously |> ignore

    // Unload
    let result = onUnload (ctx :> PluginContext) |> Async.RunSynchronously

    match result with
    | Ok () ->
        let logs = ctx.GetLogs()
        Assert.Contains(logs, fun (_, msg) -> msg.Contains("unload"))
    | Error err ->
        Assert.True(false, sprintf "OnUnload failed: %A" err)
```

### Running Tests

```bash
# Using dotnet test (if set up)
dotnet test tests/

# Or using F# Interactive
fsharpc tests/test_plugin.fsx && mono tests/test_plugin.exe
```

## Part 2: Integration Testing

Integration tests verify that your plugin works with real Scarab components.

### Test Harness

Create a minimal test harness:

```fsharp
// tests/integration_test.fsx
module IntegrationTests

open Scarab.PluginApi
open my_plugin

// Simulate a complete plugin lifecycle
let testCompleteLifecycle() =
    printfn "Testing complete plugin lifecycle..."

    // Create a real context (would need actual Scarab running)
    // For now, we'll use the mock
    let ctx = MockContext.MockContext() :> PluginContext

    // 1. Load
    printfn "  1. Loading plugin..."
    match onLoad ctx |> Async.RunSynchronously with
    | Ok () -> printfn "     ✓ Loaded successfully"
    | Error err -> failwith (sprintf "     ✗ Load failed: %A" err)

    // 2. Process some output
    printfn "  2. Processing output..."
    let testLines = [
        "normal text"
        "Check out https://github.com/raibid-labs/scarab"
        "Server at http://localhost:3000"
        "Another URL: https://example.com/very/long/path/here"
    ]

    for line in testLines do
        let result = onOutput ctx line |> Async.RunSynchronously
        match result with
        | Continue -> ()
        | Stop -> printfn "     Output stopped for: %s" line
        | Modify newLine -> printfn "     Output modified: %s -> %s" line newLine

    printfn "     ✓ Processed %d lines" (List.length testLines)

    // 3. Simulate resize
    printfn "  3. Resizing terminal..."
    onResize ctx 120us 40us |> Async.RunSynchronously
    printfn "     ✓ Resized to 120x40"

    // 4. Unload
    printfn "  4. Unloading plugin..."
    match onUnload ctx |> Async.RunSynchronously with
    | Ok () -> printfn "     ✓ Unloaded successfully"
    | Error err -> failwith (sprintf "     ✗ Unload failed: %A" err)

    printfn "✓ Integration test passed"

// Run the test
testCompleteLifecycle()
```

### End-to-End Testing

Test with actual Scarab:

```bash
# 1. Build plugin
just plugin-build my-plugin

# 2. Start Scarab in test mode
SCARAB_TEST_MODE=1 just run-bg

# 3. Run test script
./tests/e2e_test.sh

# 4. Verify output
# Check logs, notifications, etc.
```

## Part 3: Performance Profiling

### Benchmarking

Measure hook performance:

```fsharp
// tests/benchmark.fsx
module Benchmarks

open System
open System.Diagnostics
open my_plugin

let benchmark name iterations fn =
    // Warm up
    for _ in 1..10 do
        fn() |> ignore

    // Measure
    let sw = Stopwatch.StartNew()
    for _ in 1..iterations do
        fn() |> ignore
    sw.Stop()

    let avgMs = float sw.ElapsedMilliseconds / float iterations
    let avgUs = avgMs * 1000.0

    printfn "%s: %.2f μs/iter (%.2f ms total for %d iterations)"
        name avgUs sw.ElapsedMilliseconds iterations

let runBenchmarks() =
    let ctx = MockContext.MockContext() :> PluginContext

    // Benchmark URL detection
    benchmark "URL Detection (match)" 10000 (fun () ->
        onOutput ctx "https://github.com/raibid-labs/scarab"
        |> Async.RunSynchronously
    )

    benchmark "URL Detection (no match)" 10000 (fun () ->
        onOutput ctx "normal text without urls"
        |> Async.RunSynchronously
    )

    // Benchmark regex compilation benefit
    let uncompiledRegex = Text.RegularExpressions.Regex(@"https?://[^\s]+")
    benchmark "Uncompiled Regex" 10000 (fun () ->
        uncompiledRegex.IsMatch("https://example.com")
    )

    benchmark "Compiled Regex" 10000 (fun () ->
        urlRegex.IsMatch("https://example.com")
    )

    printfn ""
    printfn "Performance targets:"
    printfn "  Backend OnOutput: < 50 μs"
    printfn "  Frontend hooks:   < 16 ms (60 FPS)"
    printfn "  OnLoad/OnUnload:  < 100 ms"

runBenchmarks()
```

### Running Benchmarks

```bash
# Run benchmarks
fsi tests/benchmark.fsx

# Expected output:
# URL Detection (match): 12.34 μs/iter
# URL Detection (no match): 2.15 μs/iter
# Uncompiled Regex: 45.67 μs/iter
# Compiled Regex: 8.90 μs/iter
```

### Profiling with Real Data

```fsharp
// tests/profile.fsx
module Profiling

open System
open System.IO
open my_plugin

let profileWithRealData() =
    let ctx = MockContext.MockContext() :> PluginContext

    // Load real terminal output sample
    let lines = File.ReadAllLines("tests/sample_output.txt")

    printfn "Profiling with %d lines of real output..." (Array.length lines)

    let sw = Stopwatch.StartNew()
    let mutable processedLines = 0
    let mutable matchedLines = 0

    for line in lines do
        let result = onOutput ctx line |> Async.RunSynchronously
        processedLines <- processedLines + 1

        match result with
        | Continue -> ()
        | Stop | Modify _ -> matchedLines <- matchedLines + 1

    sw.Stop()

    printfn "Results:"
    printfn "  Processed: %d lines" processedLines
    printfn "  Matched:   %d lines" matchedLines
    printfn "  Time:      %d ms" sw.ElapsedMilliseconds
    printfn "  Rate:      %.2f lines/sec" (float processedLines / sw.Elapsed.TotalSeconds)
    printfn "  Avg:       %.2f μs/line" (float sw.ElapsedMilliseconds * 1000.0 / float processedLines)
```

## Part 4: Memory Usage Monitoring

### Memory Profiling

Track memory usage during execution:

```fsharp
// tests/memory_test.fsx
module MemoryTests

open System
open System.Diagnostics

let measureMemoryUsage name iterations fn =
    // Force GC before measurement
    GC.Collect()
    GC.WaitForPendingFinalizers()
    GC.Collect()

    let beforeMem = GC.GetTotalMemory(false)

    // Run function
    for _ in 1..iterations do
        fn() |> ignore

    // Force GC after
    GC.Collect()
    GC.WaitForPendingFinalizers()
    GC.Collect()

    let afterMem = GC.GetTotalMemory(false)
    let deltaBytes = afterMem - beforeMem
    let deltaKB = float deltaBytes / 1024.0

    printfn "%s: %.2f KB total, %.2f bytes/iter"
        name deltaKB (float deltaBytes / float iterations)

let runMemoryTests() =
    let ctx = MockContext.MockContext() :> PluginContext

    measureMemoryUsage "URL Detection (string allocation)" 1000 (fun () ->
        onOutput ctx (sprintf "https://example.com/%d" (Random().Next()))
        |> Async.RunSynchronously
    )

    measureMemoryUsage "URL Detection (reused string)" 1000 (fun () ->
        onOutput ctx "https://example.com/fixed"
        |> Async.RunSynchronously
    )

    printfn ""
    printfn "Memory targets:"
    printfn "  Per-hook execution: < 1 KB"
    printfn "  Plugin total (idle): < 10 MB"

runMemoryTests()
```

### Leak Detection

Check for memory leaks over extended runs:

```fsharp
let detectLeaks() =
    let ctx = MockContext.MockContext() :> PluginContext

    printfn "Testing for memory leaks over 100,000 iterations..."

    let measurements = ResizeArray<int64>()

    for i in 1..10 do
        GC.Collect()
        let mem = GC.GetTotalMemory(false)
        measurements.Add(mem)

        // Run many iterations
        for _ in 1..10000 do
            onOutput ctx "test data" |> Async.RunSynchronously |> ignore

        let memKB = float mem / 1024.0
        printfn "  After %d iterations: %.2f KB" (i * 10000) memKB

    // Check if memory is growing unbounded
    let first = measurements.[0]
    let last = measurements.[measurements.Count - 1]
    let growthPercent = float (last - first) / float first * 100.0

    if growthPercent > 50.0 then
        printfn "⚠ WARNING: Memory grew by %.2f%% - possible leak!" growthPercent
    else
        printfn "✓ Memory stable (%.2f%% growth)" growthPercent
```

## Part 5: Packaging

### Plugin Metadata (plugin.toml)

Create a comprehensive plugin manifest:

```toml
[plugin]
name = "my-awesome-plugin"
version = "1.2.3"
runtime = "backend"  # or "frontend" or "hybrid"

[plugin.metadata]
description = "A comprehensive description of what your plugin does"
author = "Your Name <your.email@example.com>"
license = "MIT"
homepage = "https://github.com/yourname/my-awesome-plugin"
repository = "https://github.com/yourname/my-awesome-plugin"
documentation = "https://github.com/yourname/my-awesome-plugin/blob/main/README.md"
keywords = ["productivity", "git", "terminal"]
categories = ["development", "utilities"]

# Scarab compatibility
min_scarab_version = "0.1.0"
max_scarab_version = "0.9.9"

# Dependencies (other plugins)
[plugin.dependencies]
other-plugin = "^1.0.0"

# Plugin configuration schema
[plugin.config]
# Define expected config keys with defaults
enabled = true
min_url_length = 50
debounce_ms = 500
notify_on_match = true

# Hooks that this plugin uses
[hooks]
on_load = true
on_output = true
on_input = false
on_resize = true
on_unload = true
on_key_press = false
on_pre_command = false
on_remote_command = false

# Optional: Multiple components for hybrid plugins
[[component]]
name = "my-plugin-backend"
runtime = "backend"
source = "backend.fsx"

[component.hooks]
on_load = true
on_output = true

[[component]]
name = "my-plugin-frontend"
runtime = "frontend"
source = "frontend.fsx"

[component.hooks]
on_load = true
on_key_press = true
```

### README.md

Create comprehensive documentation:

```markdown
# My Awesome Plugin

A brief, compelling description of what your plugin does.

## Features

- Feature 1: Does amazing thing X
- Feature 2: Handles Y automatically
- Feature 3: Provides Z insights

## Installation

```bash
scarab plugin install my-awesome-plugin
```

## Configuration

Add to `~/.config/scarab/config.toml`:

```toml
[[plugins]]
name = "my-awesome-plugin"
enabled = true

[plugins.config]
min_url_length = 50
notify_on_match = true
```

## Usage

1. Start Scarab
2. The plugin automatically detects...
3. You can trigger X by pressing...

### Keyboard Shortcuts

- `Ctrl+P` - Open command palette
- `Ctrl+Shift+U` - Show URL history

## Examples

### Example 1: Basic Usage

```bash
# Do something that triggers the plugin
git status
```

### Example 2: Advanced

```bash
# Configure for specific use case
export MY_PLUGIN_MODE=advanced
```

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable/disable plugin |
| `min_url_length` | integer | `50` | Minimum URL length to process |
| `notify_on_match` | boolean | `true` | Show notifications on match |

## Troubleshooting

### Plugin not loading

1. Check that plugin is enabled in config
2. Verify Scarab version compatibility
3. Check logs: `tail -f ~/.local/share/scarab/daemon.log`

### Performance issues

1. Increase debounce time
2. Disable notifications
3. Check system resources

## Development

### Building from Source

```bash
git clone https://github.com/yourname/my-awesome-plugin
cd my-awesome-plugin
just plugin-build my-awesome-plugin
```

### Running Tests

```bash
dotnet test tests/
```

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE)

## Changelog

See [CHANGELOG.md](CHANGELOG.md)
```

### CHANGELOG.md

Track version history:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New feature X

### Changed
- Improved performance of Y

### Fixed
- Bug in Z when...

## [1.2.3] - 2025-01-15

### Added
- Support for custom URL patterns
- Configuration option for debounce time

### Fixed
- Memory leak in long-running sessions
- Crash when terminal size < 80x24

## [1.2.0] - 2025-01-01

### Added
- Initial public release
- URL detection and shortening
- Git status parsing
- Command palette integration

## [1.0.0] - 2024-12-15

### Added
- Initial version
- Basic URL detection
```

### Building the Package

```bash
# Using the just command
just plugin-package my-awesome-plugin

# This creates:
# dist/my-awesome-plugin-1.2.3.tar.gz
```

The package includes:
- Compiled .fzb bytecode (for backend)
- Source .fsx files (for frontend)
- plugin.toml
- README.md
- LICENSE
- CHANGELOG.md

## Part 6: Publishing

### Pre-Publication Checklist

Before publishing, verify:

- [ ] All tests pass
- [ ] Performance benchmarks meet targets
- [ ] No memory leaks
- [ ] Documentation is complete
- [ ] README has examples
- [ ] CHANGELOG is updated
- [ ] Version number follows semver
- [ ] License file exists
- [ ] plugin.toml is valid
- [ ] Works on clean Scarab installation

### Publishing to Registry

```bash
# 1. Build and package
just plugin-package my-awesome-plugin

# 2. Test the package
just plugin-test-package dist/my-awesome-plugin-1.2.3.tar.gz

# 3. Publish (requires authentication)
scarab plugin publish dist/my-awesome-plugin-1.2.3.tar.gz

# Or use the just command
just plugin-publish my-awesome-plugin
```

### Registry Metadata

The registry requires additional metadata:

```toml
# In plugin.toml
[plugin.registry]
# Short description for search results
tagline = "Smart URL detection and shortening for your terminal"

# Detailed description (markdown)
long_description = """
This plugin automatically detects URLs in terminal output,
offers to shorten them, and copies to clipboard.

Perfect for developers who frequently share links.
"""

# Screenshots (URLs or file paths)
screenshots = [
    "https://example.com/screenshot1.png",
    "https://example.com/screenshot2.png"
]

# Demo video
demo_video = "https://example.com/demo.mp4"

# Icon (URL or file path)
icon = "icon.png"
```

### Publishing Process

1. **Submit for Review**
   ```bash
   scarab plugin submit my-awesome-plugin
   ```

2. **Automated Checks**
   - Security scan
   - Performance test
   - Compatibility check
   - Documentation validation

3. **Manual Review** (1-3 days)
   - Code quality
   - Functionality
   - User experience

4. **Approval**
   - Plugin is published
   - Available in registry
   - Users can install

## Part 7: Versioning Strategy

### Semantic Versioning

Follow [semver](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

Examples:
- `1.0.0` → `1.0.1` - Bug fix
- `1.0.1` → `1.1.0` - New feature
- `1.1.0` → `2.0.0` - Breaking change

### Version Tags

```bash
# Tag releases in git
git tag -a v1.2.3 -m "Release version 1.2.3"
git push origin v1.2.3
```

### Deprecation Policy

When making breaking changes:

1. **Announce** in advance (at least one minor version)
2. **Provide migration guide**
3. **Support old API** for transition period
4. **Remove** in next major version

Example:

```fsharp
// v1.5.0 - Deprecate old API
[<Obsolete("Use newFunction instead. Will be removed in v2.0.0")>]
let oldFunction() = ()

let newFunction() = ()

// v2.0.0 - Remove old API
// oldFunction is removed
```

## Part 8: Production Deployment Checklist

### Pre-Deployment

- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Performance benchmarks meet targets
- [ ] Memory usage within limits
- [ ] Tested on multiple terminals (Scarab, others)
- [ ] Tested on multiple OS (Linux, macOS, Windows if applicable)
- [ ] Documentation complete
- [ ] Examples work
- [ ] Error messages are helpful
- [ ] Logging is appropriate (not too verbose)

### Security

- [ ] No hardcoded credentials
- [ ] API keys from config, not code
- [ ] Input validation for all user data
- [ ] No arbitrary code execution
- [ ] Secure external API calls (HTTPS)
- [ ] No sensitive data in logs
- [ ] File permissions correct

### Performance

- [ ] Backend OnOutput < 50μs
- [ ] Frontend hooks < 16ms
- [ ] No blocking operations
- [ ] Regex patterns compiled
- [ ] Memory usage stable
- [ ] No leaks over time

### User Experience

- [ ] Clear error messages
- [ ] Helpful notifications
- [ ] Keyboard shortcuts documented
- [ ] Accessible (keyboard-only navigation)
- [ ] Responsive to terminal resize
- [ ] Works with different color schemes

### Maintenance

- [ ] CI/CD pipeline set up
- [ ] Automated tests on commit
- [ ] Version numbers automated
- [ ] Release notes generated
- [ ] Issue tracker configured
- [ ] Community guidelines posted

## What You Learned

- ✅ Unit testing with MockPluginContext
- ✅ Integration testing strategies
- ✅ Performance profiling and benchmarking
- ✅ Memory usage monitoring and leak detection
- ✅ Packaging with plugin.toml
- ✅ Publishing workflow
- ✅ Versioning strategy with semver
- ✅ Production deployment checklist
- ✅ Security best practices
- ✅ Maintenance planning

## Next Steps

→ **[Plugin API Reference](../api-reference/plugin-context.md)** - Complete API documentation

→ **[Example Plugins](../examples/)** - Study production plugins

→ **[Contributing Guide](../../../CONTRIBUTING.md)** - Contribute to Scarab

## Additional Resources

### Testing Frameworks

- [xUnit](https://xunit.net/) - Unit testing for F#
- [FsUnit](https://fsprojects.github.io/FsUnit/) - F# unit test syntax
- [Expecto](https://github.com/haf/expecto) - F# testing library

### Performance Tools

- [BenchmarkDotNet](https://benchmarkdotnet.org/) - Benchmarking
- [dotMemory](https://www.jetbrains.com/dotmemory/) - Memory profiling
- [PerfView](https://github.com/microsoft/perfview) - Performance analysis

### Publishing

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)

### CI/CD

- [GitHub Actions](https://github.com/features/actions)
- [GitLab CI](https://docs.gitlab.com/ee/ci/)
- [CircleCI](https://circleci.com/)

## Troubleshooting

### Tests failing in CI but passing locally?

1. **Check environment variables** - CI may not have your local config
2. **Verify dependencies** - Ensure all deps in package.json/requirements.txt
3. **Check file paths** - Use absolute or relative-to-root paths
4. **Timezone/locale issues** - Tests may depend on system settings

### Package build fails?

1. **Verify all files exist** - Check plugin.toml sources
2. **Check permissions** - Ensure files are readable
3. **Validate toml** - Use TOML validator
4. **Check disk space** - Build may need temporary space

### Plugin rejected from registry?

Common reasons:
1. **Security issues** - Vulnerable dependencies, unsafe code
2. **Performance** - Doesn't meet benchmarks
3. **Documentation** - Incomplete or unclear
4. **Functionality** - Doesn't work as described
5. **Licensing** - Unclear or restrictive license

Review feedback and resubmit.

## Congratulations!

You've completed the Scarab plugin development tutorial series! You now know how to:

- Create frontend and backend plugins
- Build rich UIs with RemoteUI
- Optimize performance
- Test thoroughly
- Package and publish

**Go build amazing plugins and share them with the community!**

Happy coding!
