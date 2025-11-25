# Plugin Performance Guide

Performance is critical for plugin development. This guide covers optimization strategies and best practices.

## Performance Budgets

### Frontend Plugins
- **Frame budget:** 16ms @ 60fps
- **Target:** < 10ms per frame
- **Maximum:** 16.67ms (or frame drops)

### Backend Plugins
- **OnOutput:** < 50μs per line
- **OnInput:** < 100μs per input
- **Commands:** < 5ms per command

## Optimization Strategies

### 1. Early Returns

```fsharp
[<OnOutput>]
let onOutput ctx line =
    async {
        // Fast path - no allocation
        if not (line.Contains "http") then
            return Continue

        // Slow path - only if needed
        if urlRegex.IsMatch(line) then
            processUrl line

        return Continue
    }
```

### 2. Compiled Regex

```fsharp
// Good - compiled once
let urlRegex = Regex(@"https?://\S+", RegexOptions.Compiled)

// Bad - compiled on every call
let urlRegex = Regex(@"https?://\S+")
```

### 3. Batching

```fsharp
// Bad - notify on every match
[<OnOutput>]
let onOutput ctx line =
    if hasError line then
        ctx.NotifyError "Error" line  // Too frequent!

// Good - batch notifications
let mutable errorCount = 0
[<OnOutput>]
let onOutput ctx line =
    if hasError line then
        errorCount <- errorCount + 1
        if errorCount % 10 = 0 then
            ctx.NotifyWarning "Errors" (sprintf "%d errors detected" errorCount)
```

### 4. Caching

```fsharp
// Use ctx.SetData/GetData for caching
match ctx.GetData "compiled_pattern" with
| Some pattern -> pattern
| None ->
    let pattern = compileExpensive()
    ctx.SetData "compiled_pattern" pattern
    pattern
```

## Profiling

Use the built-in profiling:

```bash
cargo build --profile profiling
cargo run --profile profiling -p scarab-daemon
```

## Common Pitfalls

1. **Blocking I/O** - Use async
2. **Excessive logging** - Use Debug sparingly
3. **Large allocations** - Reuse objects
4. **Complex regex** - Simplify patterns
5. **Notification spam** - Rate limit

## Benchmarking

Test your plugins:

```bash
cargo bench -p scarab-plugin-api
```

## Next Steps

→ **[Frontend vs Backend](frontend-vs-backend.md)**

→ **[API Reference](../api-reference/plugin-context.md)**
