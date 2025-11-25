# Tutorial 2: Hello World (Backend Plugin)

In this tutorial, you'll create a backend plugin that detects URLs in terminal output and highlights them.

**What you'll learn:**
- Creating a backend (.fzb) plugin
- Processing terminal output
- Pattern matching and text detection
- Returning actions from hooks

**Time:** 20 minutes

## Prerequisites

- Completed [Tutorial 1: Hello World (Frontend)](01-hello-world-frontend.md)
- Understanding of async/await in F#

## Frontend vs Backend: Key Differences

| Aspect | Frontend (.fsx) | Backend (.fzb) |
|--------|----------------|----------------|
| **Runs in** | Client (Bevy) | Daemon |
| **Compilation** | Interpreted | Compiled to bytecode |
| **Performance** | Good for UI | Optimized for processing |
| **Access** | UI components | Terminal output/input |
| **Hot reload** | Yes | No (requires restart) |

**Use backend plugins when:**
- Processing terminal output
- Detecting patterns in text
- Modifying terminal state
- Performance is critical

## Step 1: Create the Backend Plugin

```bash
just plugin-new url-detector backend
```

This creates the plugin structure. Notice the `runtime = "backend"` in `plugin.toml`.

## Step 2: Understand Backend Hooks

Open `plugins/url-detector/url-detector.fsx`:

```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (text: string) =
    // TODO: Process terminal output
    async { return Continue }
```

The `OnOutput` hook receives every line of terminal output **before** it's displayed. You can:

1. **Continue** - Pass the output unchanged to the next plugin
2. **Stop** - Block the output from being displayed
3. **Modify** - Change the output before displaying

## Step 3: Detect URLs

Let's write a function to detect URLs. Add this above the `onOutput` function:

```fsharp
open System.Text.RegularExpressions

// URL detection regex
let urlPattern = @"https?://[^\s<>""{}|\\^`\[\]]+"
let urlRegex = Regex(urlPattern, RegexOptions.Compiled)

// Check if line contains a URL
let containsUrl (text: string) : bool =
    urlRegex.IsMatch(text)

// Extract all URLs from text
let extractUrls (text: string) : string list =
    urlRegex.Matches(text)
    |> Seq.cast<Match>
    |> Seq.map (fun m -> m.Value)
    |> Seq.toList
```

## Step 4: Process Output

Now update the `onOutput` hook to detect URLs:

```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (text: string) =
    async {
        // Check if this line contains a URL
        if containsUrl text then
            let urls = extractUrls text
            let urlCount = List.length urls

            // Log the detection
            ctx.Log Info (sprintf "Detected %d URL(s) in output" urlCount)

            // Send a notification (will appear in client)
            if urlCount = 1 then
                ctx.Notify "URL Detected" (List.head urls) NotifyLevel.Info
            else
                ctx.Notify "URLs Detected" (sprintf "Found %d URLs" urlCount) NotifyLevel.Info

        // Always continue - don't block output
        return Continue
    }
```

## Step 5: Update Plugin Metadata

```fsharp
[<Plugin>]
let metadata = {
    Name = "url-detector"
    Version = "0.1.0"
    Description = "Detects URLs in terminal output and highlights them"
    Author = "Your Name"
    Emoji = Some "🔗"
    Color = Some "#2196F3"
    Catchphrase = Some "Never miss a link!"
}
```

## Step 6: Enable the Hook

Update `plugins/url-detector/plugin.toml`:

```toml
[plugin]
name = "url-detector"
version = "0.1.0"
runtime = "backend"

[plugin.metadata]
description = "Detects URLs in terminal output and highlights them"
author = "Your Name"
license = "MIT"

[hooks]
on_load = true
on_output = true
```

Note: `on_output = true` enables the output hook.

## Step 7: Build and Test

Backend plugins must be compiled (no hot reload):

```bash
# Build the plugin
just plugin-build url-detector

# Start Scarab
just run-bg
```

Now test it:

```bash
# In your terminal, output a URL
echo "Check out https://github.com/raibid-labs/scarab"
```

You should see:
1. A log message in the daemon logs
2. A notification in the Scarab client

## Step 8: Handle Multiple URLs

Let's improve the notification to show all URLs. Update the `onOutput` function:

```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (text: string) =
    async {
        if containsUrl text then
            let urls = extractUrls text
            let urlCount = List.length urls

            ctx.Log Info (sprintf "Detected %d URL(s): %s" urlCount (String.concat ", " urls))

            // Show detailed notification
            let message =
                match urls with
                | [url] -> url
                | multiple -> String.concat "\n" (List.take (min 3 (List.length multiple)) multiple)

            ctx.Notify "URL Detected" message NotifyLevel.Info

        return Continue
    }
```

## Step 9: Add Filtering

Not all URLs are interesting. Let's add a filter:

```fsharp
// URLs to ignore (localhost, common false positives)
let ignoreUrls = Set.ofList [
    "http://localhost"
    "https://localhost"
    "http://127.0.0.1"
]

// Check if URL should be reported
let shouldReport (url: string) : bool =
    not (Set.exists (fun ignore -> url.StartsWith(ignore)) ignoreUrls)

// Update onOutput to use filter
[<OnOutput>]
let onOutput (ctx: PluginContext) (text: string) =
    async {
        if containsUrl text then
            let urls =
                extractUrls text
                |> List.filter shouldReport

            if not (List.isEmpty urls) then
                let urlCount = List.length urls
                ctx.Log Info (sprintf "Detected %d URL(s): %s" urlCount (String.concat ", " urls))

                let message =
                    match urls with
                    | [url] -> url
                    | multiple -> String.concat "\n" (List.take (min 3 (List.length multiple)) multiple)

                ctx.Notify "URL Detected" message NotifyLevel.Info

        return Continue
    }
```

## Complete Plugin

Here's your complete backend plugin:

```fsharp
module url_detector

open Scarab.PluginApi
open System.Text.RegularExpressions

[<Plugin>]
let metadata = {
    Name = "url-detector"
    Version = "0.1.0"
    Description = "Detects URLs in terminal output and highlights them"
    Author = "Your Name"
    Emoji = Some "🔗"
    Color = Some "#2196F3"
    Catchphrase = Some "Never miss a link!"
}

// URL detection regex
let urlPattern = @"https?://[^\s<>""{}|\\^`\[\]]+"
let urlRegex = Regex(urlPattern, RegexOptions.Compiled)

// URLs to ignore
let ignoreUrls = Set.ofList [
    "http://localhost"
    "https://localhost"
    "http://127.0.0.1"
]

// Check if line contains a URL
let containsUrl (text: string) : bool =
    urlRegex.IsMatch(text)

// Extract all URLs from text
let extractUrls (text: string) : string list =
    urlRegex.Matches(text)
    |> Seq.cast<Match>
    |> Seq.map (fun m -> m.Value)
    |> Seq.toList

// Check if URL should be reported
let shouldReport (url: string) : bool =
    not (Set.exists (fun ignore -> url.StartsWith(ignore)) ignoreUrls)

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    ctx.Log Info "URL Detector plugin loaded!"
    async { return Ok () }

[<OnOutput>]
let onOutput (ctx: PluginContext) (text: string) =
    async {
        if containsUrl text then
            let urls =
                extractUrls text
                |> List.filter shouldReport

            if not (List.isEmpty urls) then
                let urlCount = List.length urls
                ctx.Log Info (sprintf "Detected %d URL(s): %s" urlCount (String.concat ", " urls))

                let message =
                    match urls with
                    | [url] -> url
                    | multiple -> String.concat "\n" (List.take (min 3 (List.length multiple)) multiple)

                ctx.Notify "URL Detected" message NotifyLevel.Info

        return Continue
    }
```

## What You Learned

- ✅ Creating backend plugins for output processing
- ✅ Using regex for pattern matching
- ✅ Filtering and validating detected data
- ✅ Returning actions from hooks (Continue, Stop, Modify)
- ✅ Backend plugins run in the daemon for performance
- ✅ No hot reload - backend plugins require rebuild

## Performance Considerations

Backend plugins run on **every line of output**. Keep them fast:

1. **Compile regex patterns once** (use `RegexOptions.Compiled`)
2. **Return early** if no match is possible
3. **Avoid blocking operations** (file I/O, network calls)
4. **Use async properly** - don't block the daemon

## Next Steps

→ **[Tutorial 3: Plugin API Deep Dive](03-plugin-api-deep-dive.md)** - Learn all available hooks and methods

→ **[Tutorial 4: Real Plugin (URL Shortener)](04-real-plugin-url-shortener.md)** - Build a complete plugin with external API calls

→ **[Backend Processing with Hooks](06-backend-hooks.md)** - Advanced backend patterns

## Troubleshooting

### Plugin not processing output?

1. Check `on_output = true` in `plugin.toml`
2. Verify plugin is enabled in `~/.config/scarab/config.toml`
3. Rebuild: `just plugin-build url-detector`
4. Restart daemon: `just kill && just run-bg`

### Notifications not appearing?

Remember: Backend plugins run in the **daemon**, but notifications appear in the **client**.
- Ensure the client is running
- Check client logs for errors
- Verify `ctx.Notify()` is being called (check daemon logs)

### Performance issues?

- Use `ctx.Log Debug` sparingly (only for debugging)
- Avoid regex on every line - pre-filter with simple checks
- Profile with `cargo flamegraph`

## Challenge

Try these enhancements:

1. **Click to open** - Make URLs clickable in the terminal
2. **Domain highlighting** - Different colors for different domains
3. **URL shortening** - Automatically shorten long URLs
4. **History tracking** - Keep a list of all detected URLs

Happy coding!
