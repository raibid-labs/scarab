# Tutorial 4: Building a Real Plugin - URL Shortener

In this tutorial, you'll build a complete, production-ready plugin that detects long URLs in terminal output, shortens them using an external API, and provides a seamless user experience with notifications and clipboard integration.

**What you'll learn:**
- Building a hybrid plugin (backend + frontend)
- Making external API calls from plugins
- Communicating between backend and frontend
- Integrating with the system clipboard
- Handling errors gracefully
- Creating a polished user experience

**Time:** 45 minutes

## Prerequisites

- Completed [Tutorial 2: Hello World (Backend)](02-hello-world-backend.md)
- Understanding of async/await in F#
- API key for a URL shortening service (we'll use is.gd - no signup required)

## What We're Building

Our URL shortener plugin will:

1. **Detect long URLs** (> 50 characters) in terminal output
2. **Offer to shorten** via a notification with action buttons
3. **Call the is.gd API** to generate a short URL
4. **Show the result** in a notification
5. **Copy to clipboard** automatically for convenience

## Architecture Overview

This is a **hybrid plugin** with two components:

```
Backend (.fzb)                    Frontend (.fsx)
    ↓                                  ↓
Detects URLs in output         Shows notifications
    ↓                                  ↓
Calls shortening API           Handles user actions
    ↓                                  ↓
Sends result to frontend       Updates clipboard
```

## Part 1: Backend - URL Detection and Shortening

### Step 1: Create the Backend Plugin

```bash
just plugin-new url-shortener backend
cd plugins/url-shortener
```

### Step 2: Implement URL Detection

Open `url-shortener.fsx` and add the detection logic:

```fsharp
module url_shortener

open Scarab.PluginApi
open System
open System.Text.RegularExpressions
open System.Net.Http

[<Plugin>]
let metadata = {
    Name = "url-shortener"
    Version = "1.0.0"
    Description = "Automatically detects and shortens long URLs"
    Author = "Your Name"
    Emoji = Some "🔗"
    Color = Some "#FF6B6B"
    Catchphrase = Some "Short URLs, big impact!"
}

// URL detection regex - matches http/https URLs
let urlPattern = @"https?://[^\s<>""{}|\\^`\[\]]+"
let urlRegex = Regex(urlPattern, RegexOptions.Compiled)

// Minimum length to trigger shortening (50 chars)
let minUrlLength = 50

// Extract URLs from text
let extractUrls (text: string) : string list =
    urlRegex.Matches(text)
    |> Seq.cast<Match>
    |> Seq.map (fun m -> m.Value)
    |> Seq.filter (fun url -> url.Length >= minUrlLength)
    |> Seq.toList

// Check if text contains long URLs
let containsLongUrl (text: string) : bool =
    urlRegex.IsMatch(text) &&
    (extractUrls text |> List.isEmpty |> not)
```

### Step 3: Add URL Shortening API

Add the is.gd API client:

```fsharp
// HTTP client for API calls
let httpClient = new HttpClient()

// Shorten a URL using is.gd API
let shortenUrl (longUrl: string) : Async<Result<string, string>> =
    async {
        try
            // is.gd API endpoint (no auth required)
            let apiUrl = sprintf "https://is.gd/create.php?format=simple&url=%s"
                            (Uri.EscapeDataString(longUrl))

            // Make the API call
            let! response = httpClient.GetAsync(apiUrl) |> Async.AwaitTask

            if response.IsSuccessStatusCode then
                let! shortUrl = response.Content.ReadAsStringAsync() |> Async.AwaitTask
                return Ok shortUrl
            else
                let! errorBody = response.Content.ReadAsStringAsync() |> Async.AwaitTask
                return Error (sprintf "API error: %s" errorBody)
        with
        | ex ->
            return Error (sprintf "Failed to shorten URL: %s" ex.Message)
    }
```

**Note:** The is.gd API is free and doesn't require registration. For production use, consider using a service with authentication (bit.ly, TinyURL, etc.) and storing the API key in `plugin.toml`.

### Step 4: Implement the Output Hook

Now connect everything in the `OnOutput` hook:

```fsharp
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Quick pre-filter before regex
        if not (line.Contains "http") then
            return Continue

        // Extract long URLs
        let urls = extractUrls line

        if not (List.isEmpty urls) then
            // Process each URL
            for url in urls do
                ctx.Log Info (sprintf "Found long URL: %s" (url.Substring(0, min 50 url.Length)))

                // Attempt to shorten
                let! result = shortenUrl url

                match result with
                | Ok shortUrl ->
                    ctx.Log Info (sprintf "Shortened to: %s" shortUrl)

                    // Send notification with both URLs
                    ctx.Notify
                        "URL Shortened"
                        (sprintf "Long: %s...\nShort: %s"
                            (url.Substring(0, min 40 url.Length))
                            shortUrl)
                        NotifyLevel.Success

                    // Send short URL to frontend for clipboard
                    ctx.SendToFrontend (sprintf "copy:%s" shortUrl)

                | Error errMsg ->
                    ctx.Log Error errMsg
                    ctx.Notify "Shortening Failed" errMsg NotifyLevel.Error

        return Continue
    }
```

### Step 5: Add Plugin Lifecycle

Add the initialization and cleanup hooks:

```fsharp
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "URL Shortener plugin loaded"

        // Verify internet connectivity
        try
            let! response = httpClient.GetAsync("https://is.gd/") |> Async.AwaitTask
            if response.IsSuccessStatusCode then
                ctx.NotifySuccess "URL Shortener Ready" "Watching for long URLs..."
            else
                ctx.NotifyWarning "URL Shortener" "May not be able to reach API"
        with
        | ex ->
            ctx.Log Warn (sprintf "Connectivity check failed: %s" ex.Message)

        return Ok ()
    }

[<OnUnload>]
let onUnload (ctx: PluginContext) =
    async {
        ctx.Log Info "URL Shortener plugin unloading"

        // Clean up HTTP client
        httpClient.Dispose()

        return Ok ()
    }
```

## Part 2: Frontend - Clipboard Integration

Now let's create the frontend component to handle clipboard operations.

### Step 6: Create Frontend Plugin

Create a new file `url-shortener-frontend.fsx` in the same directory:

```fsharp
module url_shortener_frontend

open Scarab.PluginApi
open System

[<Plugin>]
let metadata = {
    Name = "url-shortener-frontend"
    Version = "1.0.0"
    Description = "Frontend companion for URL shortener"
    Author = "Your Name"
    Emoji = Some "📋"
    Color = Some "#4ECDC4"
    Catchphrase = Some "Copied to clipboard!"
}

// Platform-specific clipboard access
// Note: In production, use a proper clipboard library
let copyToClipboard (text: string) : Result<unit, string> =
    try
        // Use xclip on Linux
        let psi = System.Diagnostics.ProcessStartInfo()
        psi.FileName <- "xclip"
        psi.Arguments <- "-selection clipboard"
        psi.RedirectStandardInput <- true
        psi.UseShellExecute <- false

        let proc = System.Diagnostics.Process.Start(psi)
        proc.StandardInput.Write(text)
        proc.StandardInput.Close()
        proc.WaitForExit()

        if proc.ExitCode = 0 then
            Ok ()
        else
            Error "xclip failed"
    with
    | ex ->
        Error ex.Message

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "URL Shortener Frontend loaded"
        return Ok ()
    }

[<OnRemoteCommand>]
let onRemoteCommand (ctx: PluginContext) (command: string) =
    async {
        // Commands from backend are prefixed with action type
        if command.StartsWith("copy:") then
            let url = command.Substring(5)

            match copyToClipboard url with
            | Ok () ->
                ctx.Log Info (sprintf "Copied to clipboard: %s" url)
                ctx.NotifySuccess "Copied!" url
            | Error err ->
                ctx.Log Error (sprintf "Clipboard error: %s" err)
                ctx.NotifyError "Copy Failed" "Could not access clipboard"
        else
            ctx.Log Warn (sprintf "Unknown command: %s" command)

        return ()
    }
```

### Step 7: Configure Both Plugins

Update `plugin.toml` to register both components:

```toml
[plugin]
name = "url-shortener"
version = "1.0.0"

# Backend component
[[component]]
name = "url-shortener"
runtime = "backend"
source = "url-shortener.fsx"

[component.hooks]
on_load = true
on_output = true
on_unload = true

# Frontend component
[[component]]
name = "url-shortener-frontend"
runtime = "frontend"
source = "url-shortener-frontend.fsx"

[component.hooks]
on_load = true
on_remote_command = true

[plugin.metadata]
description = "Automatically detects and shortens long URLs"
author = "Your Name"
license = "MIT"
homepage = "https://github.com/your-username/url-shortener"

# Optional: Configure minimum URL length
[plugin.config]
min_url_length = 50
```

## Step 8: Build and Test

Build the plugin:

```bash
just plugin-build url-shortener
```

Start Scarab:

```bash
just run-bg
```

Test it by outputting a long URL:

```bash
echo "Check out this article: https://github.com/raibid-labs/scarab/blob/main/docs/plugin-development/tutorials/04-real-plugin-url-shortener.md"
```

You should see:
1. A log message detecting the URL
2. A "URL Shortened" notification with both URLs
3. The short URL copied to your clipboard
4. A "Copied!" confirmation

## Step 9: Add Rate Limiting

To avoid API abuse, let's add rate limiting:

```fsharp
// Add at the top of the backend plugin
let mutable lastShortenTime = DateTime.MinValue
let minIntervalMs = 1000.0  // 1 second between requests

// Update the onOutput hook
[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        if not (line.Contains "http") then
            return Continue

        let urls = extractUrls line

        if not (List.isEmpty urls) then
            // Check rate limit
            let now = DateTime.Now
            let elapsed = (now - lastShortenTime).TotalMilliseconds

            if elapsed < minIntervalMs then
                ctx.Log Debug "Rate limit: skipping URL shortening"
                return Continue

            // Update timestamp
            lastShortenTime <- now

            // Process first URL only
            let url = List.head urls

            ctx.Log Info (sprintf "Found long URL: %s" (url.Substring(0, min 50 url.Length)))

            let! result = shortenUrl url

            match result with
            | Ok shortUrl ->
                ctx.Log Info (sprintf "Shortened to: %s" shortUrl)
                ctx.Notify
                    "URL Shortened"
                    (sprintf "Short: %s\nCopied to clipboard!" shortUrl)
                    NotifyLevel.Success
                ctx.SendToFrontend (sprintf "copy:%s" shortUrl)

            | Error errMsg ->
                ctx.Log Error errMsg
                ctx.Notify "Shortening Failed" errMsg NotifyLevel.Error

        return Continue
    }
```

## Step 10: Add Configuration Support

Make the plugin configurable via `plugin.toml`:

```fsharp
// Add configuration parsing in onLoad
[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "URL Shortener plugin loading..."

        // Read configuration
        let minLength =
            match ctx.Config.GetOpt "min_url_length" with
            | Some value -> int value
            | None -> 50

        // Update the global setting
        minUrlLength <- minLength

        ctx.Log Info (sprintf "Configured: min URL length = %d" minLength)

        // Test API connectivity
        try
            let! response = httpClient.GetAsync("https://is.gd/") |> Async.AwaitTask
            if response.IsSuccessStatusCode then
                ctx.NotifySuccess "URL Shortener Ready"
                    (sprintf "Watching for URLs > %d chars" minLength)
            else
                ctx.NotifyWarning "URL Shortener" "May not be able to reach API"
        with
        | ex ->
            ctx.Log Warn (sprintf "Connectivity check failed: %s" ex.Message)

        return Ok ()
    }
```

## Complete Backend Plugin

Here's the complete backend plugin code:

```fsharp
module url_shortener

open Scarab.PluginApi
open System
open System.Text.RegularExpressions
open System.Net.Http

[<Plugin>]
let metadata = {
    Name = "url-shortener"
    Version = "1.0.0"
    Description = "Automatically detects and shortens long URLs"
    Author = "Your Name"
    Emoji = Some "🔗"
    Color = Some "#FF6B6B"
    Catchphrase = Some "Short URLs, big impact!"
}

// Configuration
let mutable minUrlLength = 50
let mutable lastShortenTime = DateTime.MinValue
let minIntervalMs = 1000.0

// URL detection regex
let urlPattern = @"https?://[^\s<>""{}|\\^`\[\]]+"
let urlRegex = Regex(urlPattern, RegexOptions.Compiled)

// HTTP client for API calls
let httpClient = new HttpClient()

// Extract long URLs from text
let extractUrls (text: string) : string list =
    urlRegex.Matches(text)
    |> Seq.cast<Match>
    |> Seq.map (fun m -> m.Value)
    |> Seq.filter (fun url -> url.Length >= minUrlLength)
    |> Seq.toList

// Shorten a URL using is.gd API
let shortenUrl (longUrl: string) : Async<Result<string, string>> =
    async {
        try
            let apiUrl = sprintf "https://is.gd/create.php?format=simple&url=%s"
                            (Uri.EscapeDataString(longUrl))

            let! response = httpClient.GetAsync(apiUrl) |> Async.AwaitTask

            if response.IsSuccessStatusCode then
                let! shortUrl = response.Content.ReadAsStringAsync() |> Async.AwaitTask
                return Ok shortUrl
            else
                let! errorBody = response.Content.ReadAsStringAsync() |> Async.AwaitTask
                return Error (sprintf "API error: %s" errorBody)
        with
        | ex ->
            return Error (sprintf "Failed to shorten URL: %s" ex.Message)
    }

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "URL Shortener plugin loading..."

        // Read configuration
        let minLength =
            match ctx.Config.GetOpt "min_url_length" with
            | Some value -> int value
            | None -> 50

        minUrlLength <- minLength
        ctx.Log Info (sprintf "Configured: min URL length = %d" minLength)

        // Test API connectivity
        try
            let! response = httpClient.GetAsync("https://is.gd/") |> Async.AwaitTask
            if response.IsSuccessStatusCode then
                ctx.NotifySuccess "URL Shortener Ready"
                    (sprintf "Watching for URLs > %d chars" minLength)
            else
                ctx.NotifyWarning "URL Shortener" "May not be able to reach API"
        with
        | ex ->
            ctx.Log Warn (sprintf "Connectivity check failed: %s" ex.Message)

        return Ok ()
    }

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Quick pre-filter
        if not (line.Contains "http") then
            return Continue

        let urls = extractUrls line

        if not (List.isEmpty urls) then
            // Check rate limit
            let now = DateTime.Now
            let elapsed = (now - lastShortenTime).TotalMilliseconds

            if elapsed < minIntervalMs then
                ctx.Log Debug "Rate limit: skipping URL shortening"
                return Continue

            lastShortenTime <- now

            // Process first URL only
            let url = List.head urls

            ctx.Log Info (sprintf "Found long URL: %s..." (url.Substring(0, min 40 url.Length)))

            let! result = shortenUrl url

            match result with
            | Ok shortUrl ->
                ctx.Log Info (sprintf "Shortened to: %s" shortUrl)
                ctx.Notify
                    "URL Shortened"
                    (sprintf "Short: %s\nCopied to clipboard!" shortUrl)
                    NotifyLevel.Success
                ctx.SendToFrontend (sprintf "copy:%s" shortUrl)

            | Error errMsg ->
                ctx.Log Error errMsg
                ctx.Notify "Shortening Failed" errMsg NotifyLevel.Error

        return Continue
    }

[<OnUnload>]
let onUnload (ctx: PluginContext) =
    async {
        ctx.Log Info "URL Shortener plugin unloading"
        httpClient.Dispose()
        return Ok ()
    }
```

## What You Learned

- ✅ Building hybrid plugins with backend + frontend components
- ✅ Making HTTP API calls from plugins
- ✅ Backend → Frontend communication with `SendToFrontend`
- ✅ Handling external API errors gracefully
- ✅ Rate limiting to prevent API abuse
- ✅ Reading plugin configuration from `plugin.toml`
- ✅ Integrating with system clipboard
- ✅ Creating a polished user experience

## Troubleshooting

### API calls not working?

1. **Check internet connection**
   ```bash
   curl https://is.gd/
   ```

2. **Enable debug logging**
   ```fsharp
   ctx.Log Debug "Making API call..."
   ```

3. **Check API rate limits** - is.gd allows 1 request/second

### Clipboard not working?

1. **Install xclip (Linux)**
   ```bash
   sudo apt install xclip
   ```

2. **macOS alternative** - Use `pbcopy` instead of `xclip`
   ```fsharp
   psi.FileName <- "pbcopy"
   psi.Arguments <- ""
   ```

3. **Windows alternative** - Use `clip.exe`
   ```fsharp
   psi.FileName <- "clip.exe"
   ```

### URLs not being detected?

1. **Check minimum length** - Default is 50 chars
2. **Test regex pattern** - Some URLs might not match
3. **Check logs** - Look for "Found long URL" messages
4. **Try debug mode** - `ctx.Log Debug` for more detail

## Production Considerations

### Security

1. **Validate URLs** before shortening (prevent malicious sites)
2. **Use authenticated APIs** with stored credentials
3. **Sanitize API responses** before displaying
4. **Rate limit per-user** in multi-user environments

### Performance

1. **Cache shortened URLs** to avoid duplicate API calls
2. **Queue API calls** instead of blocking output
3. **Use connection pooling** for HTTP client
4. **Set reasonable timeouts** (2-5 seconds)

### User Experience

1. **Make it optional** - Add enable/disable toggle
2. **Show progress** - "Shortening..." notification
3. **Allow customization** - Choice of shortening service
4. **Keyboard shortcuts** - Manual shortening trigger

## Enhancement Ideas

Try these improvements:

1. **Custom shortener support** - Allow users to configure bit.ly, TinyURL, etc.
2. **Caching** - Store shortened URLs to avoid duplicate API calls
3. **QR code generation** - Show QR code for mobile scanning
4. **URL preview** - Show URL metadata before shortening
5. **Statistics** - Track how many URLs shortened per session
6. **Blocklist** - Don't shorten specific domains (github.com, localhost, etc.)
7. **Batch mode** - Shorten all URLs in current buffer
8. **History** - Keep track of all shortened URLs

## Next Steps

→ **[Tutorial 5: Frontend UI with RemoteUI](05-frontend-ui-remoteui.md)** - Build rich user interfaces

→ **[Backend Processing with Hooks](../architecture/performance.md)** - Optimize for performance

→ **[Plugin Testing Guide](../testing/plugin-testing.md)** - Write comprehensive tests

## Additional Resources

- **URL Shortening APIs:**
  - [is.gd API Documentation](https://is.gd/apishorteningreference.php)
  - [bit.ly API](https://dev.bitly.com/)
  - [TinyURL API](https://tinyurl.com/app/dev)

- **Clipboard Libraries:**
  - [arboard](https://github.com/1Password/arboard) - Cross-platform clipboard (Rust)
  - [clipboard-rs](https://github.com/Slackadays/Clipboard) - Rust clipboard library

Happy coding!
