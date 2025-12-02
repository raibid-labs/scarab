# Utilities API Reference

This reference documents utility functions and helpers available in the Scarab plugin API. These functions simplify common tasks and provide optimized implementations for terminal-specific operations.

## String Utilities

### Pattern Matching

#### `String.contains(pattern: string, text: string) -> bool`

Case-sensitive substring search.

```fsharp
open Scarab.PluginApi.Utilities

if String.contains "error" line then
    ctx.NotifyError "Error Detected" line
```

**Performance:** O(n) - Fast Boyer-Moore implementation

#### `String.containsIgnoreCase(pattern: string, text: string) -> bool`

Case-insensitive substring search.

```fsharp
if String.containsIgnoreCase "WARNING" line then
    processWarning line
```

**Performance:** O(n) - Optimized for ASCII

#### `String.startsWith(prefix: string, text: string) -> bool`

Check if string starts with prefix.

```fsharp
if String.startsWith "http://" url || String.startsWith "https://" url then
    handleUrl url
```

**Performance:** O(k) where k is prefix length

#### `String.endsWith(suffix: string, text: string) -> bool`

Check if string ends with suffix.

```fsharp
if String.endsWith ".log" filename then
    parseLogFile filename
```

#### `String.matches(regex: string, text: string) -> bool`

Test if regex matches string.

```fsharp
if String.matches @"^\d{4}-\d{2}-\d{2}$" date then
    parseDate date
```

**Note:** Compiles regex on each call. Use `Regex.Compiled` for repeated matches.

### Text Extraction

#### `String.extract(regex: string, text: string) -> string list`

Extract all matches from text.

```fsharp
let urls = String.extract @"https?://\S+" line
urls |> List.iter (fun url -> ctx.Log Info url)
```

**Returns:** List of matched strings

**Example:**
```fsharp
let emails = String.extract @"\b[\w\.-]+@[\w\.-]+\.\w+\b" text
// ["user@example.com"; "admin@test.org"]
```

#### `String.extractGroups(regex: string, text: string) -> (string * string list) list`

Extract matches with capture groups.

```fsharp
let pattern = @"(\w+)@(\w+\.\w+)"
let matches = String.extractGroups pattern text
// [("user@example.com", ["user"; "example.com"]); ...]

matches |> List.iter (fun (full, groups) ->
    let username = groups.[0]
    let domain = groups.[1]
    ctx.Log Info (sprintf "%s at %s" username domain)
)
```

**Returns:** List of (full match, [capture groups])

#### `String.split(separator: string, text: string) -> string list`

Split string by separator.

```fsharp
let parts = String.split "," "one,two,three"
// ["one"; "two"; "three"]
```

#### `String.lines(text: string) -> string list`

Split text into lines.

```fsharp
let lines = String.lines multilineText
lines |> List.iter processLine
```

**Handles:** `\n`, `\r\n`, and `\r` line endings

### Text Formatting

#### `String.trim(text: string) -> string`

Remove leading and trailing whitespace.

```fsharp
let cleaned = String.trim "  hello  "
// "hello"
```

#### `String.trimStart(text: string) -> string`

Remove leading whitespace.

```fsharp
let cleaned = String.trimStart "  hello"
// "hello"
```

#### `String.trimEnd(text: string) -> string`

Remove trailing whitespace.

```fsharp
let cleaned = String.trimEnd "hello  "
// "hello"
```

#### `String.truncate(maxLength: int, text: string) -> string`

Truncate string to maximum length.

```fsharp
let short = String.truncate 20 "This is a very long message"
// "This is a very long..."
```

**Adds:** "..." ellipsis if truncated

#### `String.pad(width: int, align: Align, text: string) -> string`

Pad string to width with spaces.

```fsharp
let left = String.pad 10 Align.Left "hi"      // "hi        "
let right = String.pad 10 Align.Right "hi"    // "        hi"
let center = String.pad 10 Align.Center "hi"  // "    hi    "
```

#### `String.repeat(count: int, text: string) -> string`

Repeat string N times.

```fsharp
let separator = String.repeat 40 "-"
// "----------------------------------------"
```

#### `String.join(separator: string, items: string list) -> string`

Join strings with separator.

```fsharp
let csv = String.join ", " ["one"; "two"; "three"]
// "one, two, three"
```

### Text Manipulation

#### `String.replace(pattern: string, replacement: string, text: string) -> string`

Replace all occurrences.

```fsharp
let cleaned = String.replace "ERROR" "WARN" line
```

#### `String.replaceRegex(regex: string, replacement: string, text: string) -> string`

Replace using regex pattern.

```fsharp
let sanitized = String.replaceRegex @"\d{4}-\d{4}-\d{4}-\d{4}" "****-****-****-****" text
// Hide credit card numbers
```

#### `String.toLowerCase(text: string) -> string`

Convert to lowercase.

```fsharp
let lower = String.toLowerCase "HELLO"
// "hello"
```

#### `String.toUpperCase(text: string) -> string`

Convert to uppercase.

```fsharp
let upper = String.toUpperCase "hello"
// "HELLO"
```

#### `String.capitalize(text: string) -> string`

Capitalize first letter.

```fsharp
let capitalized = String.capitalize "hello world"
// "Hello world"
```

## Terminal Utilities

### Coordinate Conversion

#### `Terminal.cellToPixel(x: int, y: int) -> (int, int)`

Convert terminal cell coordinates to pixel coordinates.

```fsharp
let (pixelX, pixelY) = Terminal.cellToPixel 10 5
ctx.Log Debug (sprintf "Cell (10,5) is at pixel (%d,%d)" pixelX pixelY)
```

**Use case:** Positioning UI elements

#### `Terminal.pixelToCell(x: int, y: int) -> (int, int)`

Convert pixel coordinates to terminal cell coordinates.

```fsharp
let (col, row) = Terminal.pixelToCell mouseX mouseY
ctx.Log Debug (sprintf "Mouse at cell (%d,%d)" col row)
```

**Use case:** Mouse event handling

### ANSI Parsing

#### `Terminal.stripAnsi(text: string) -> string`

Remove ANSI escape codes from text.

```fsharp
let plain = Terminal.stripAnsi "\x1b[1;32mGreen Bold\x1b[0m"
// "Green Bold"
```

**Use case:** Getting plain text from styled output

#### `Terminal.parseAnsi(text: string) -> StyledText`

Parse ANSI codes into structured format.

```fsharp
type StyledText = {
    Text: string
    Foreground: Color option
    Background: Color option
    Bold: bool
    Italic: bool
    Underline: bool
}

let styled = Terminal.parseAnsi "\x1b[1;31mError\x1b[0m"
// { Text = "Error"; Foreground = Some Red; Bold = true; ... }
```

#### `Terminal.toAnsi(styled: StyledText) -> string`

Convert styled text to ANSI escape codes.

```fsharp
let ansi = Terminal.toAnsi {
    Text = "Success"
    Foreground = Some Color.Green
    Background = None
    Bold = true
    Italic = false
    Underline = false
}
// "\x1b[1;32mSuccess\x1b[0m"
```

### Terminal State

#### `Terminal.getSize() -> (int, int)`

Get current terminal size.

```fsharp
let (cols, rows) = Terminal.getSize()
ctx.Log Info (sprintf "Terminal is %dx%d" cols rows)
```

**Equivalent to:** `ctx.GetSize()`

#### `Terminal.getCursor() -> (int, int)`

Get current cursor position.

```fsharp
let (x, y) = Terminal.getCursor()
```

**Equivalent to:** `ctx.GetCursor()`

#### `Terminal.isWideChar(char: char) -> bool`

Check if character is wide (CJK, emoji).

```fsharp
if Terminal.isWideChar '中' then
    // Takes 2 cell widths
    adjustLayout()
```

**Use case:** Proper text width calculation

#### `Terminal.charWidth(char: char) -> int`

Get character width in terminal cells.

```fsharp
let width = Terminal.charWidth '😀'  // 2
let width = Terminal.charWidth 'a'   // 1
```

## Async Utilities

### Timers and Delays

#### `Async.delay(milliseconds: int) -> Async<unit>`

Delay execution.

```fsharp
async {
    ctx.NotifyInfo "Starting" "Process will begin in 3 seconds"
    do! Async.delay 3000
    startProcess()
}
```

#### `Async.timeout(milliseconds: int, operation: Async<'T>) -> Async<'T option>`

Run operation with timeout.

```fsharp
async {
    let! result = Async.timeout 5000 (fetchFromApi())
    match result with
    | Some data -> processData data
    | None -> ctx.NotifyWarning "Timeout" "Request took too long"
}
```

**Returns:** `Some result` if completed, `None` if timeout

#### `Async.interval(milliseconds: int, action: unit -> unit) -> unit`

Run action at regular intervals.

```fsharp
// Update status every second
Async.interval 1000 (fun () ->
    updateStatusBar()
)
```

**Note:** Returns immediately, runs in background

#### `Async.debounce(milliseconds: int, action: unit -> unit) -> (unit -> unit)`

Create debounced function.

```fsharp
let debouncedSearch = Async.debounce 300 (fun () ->
    performSearch query
)

// Call frequently, but only executes after 300ms of quiet
[<OnKeyPress>]
let onKeyPress ctx key =
    updateQuery key
    debouncedSearch()  // Only runs if no more keys for 300ms
```

**Use case:** Rate limiting user input

#### `Async.throttle(milliseconds: int, action: unit -> unit) -> (unit -> unit)`

Create throttled function.

```fsharp
let throttledUpdate = Async.throttle 1000 (fun () ->
    updateDisplay()
)

// Call frequently, but only executes once per second
[<OnOutput>]
let onOutput ctx line =
    processLine line
    throttledUpdate()  // Max once per second
```

**Difference from debounce:** Throttle executes at most once per interval, debounce waits for quiet period

### Concurrency

#### `Async.parallel(operations: Async<'T> list) -> Async<'T list>`

Run operations in parallel.

```fsharp
async {
    let! results = Async.parallel [
        fetchFromGitHub()
        fetchFromJira()
        fetchFromSlack()
    ]
    combineResults results
}
```

**Returns:** All results when all complete

#### `Async.race(operations: Async<'T> list) -> Async<'T>`

Run operations, return first to complete.

```fsharp
async {
    let! fastest = Async.race [
        fetchFromPrimary()
        fetchFromBackup()
    ]
    return fastest
}
```

**Returns:** First result

#### `Async.retry(maxAttempts: int, operation: Async<'T>) -> Async<'T option>`

Retry operation on failure.

```fsharp
async {
    let! result = Async.retry 3 (fetchFromApi())
    match result with
    | Some data -> return data
    | None ->
        ctx.NotifyError "Failed" "All retries exhausted"
        return defaultValue
}
```

## Clipboard Utilities

**Requires:** `clipboard.read` or `clipboard.write` permission

### Reading

#### `Clipboard.read() -> Async<string option>`

Read current clipboard contents.

```fsharp
async {
    let! content = Clipboard.read()
    match content with
    | Some text -> ctx.NotifyInfo "Clipboard" text
    | None -> ctx.NotifyWarning "Empty" "Clipboard is empty"
}
```

**Returns:** `Some text` if clipboard has text, `None` if empty

#### `Clipboard.watch(handler: string -> unit) -> unit`

Watch clipboard for changes.

```fsharp
Clipboard.watch (fun newContent ->
    ctx.Log Info (sprintf "Clipboard changed: %s" newContent)
    addToHistory newContent
)
```

**Use case:** Clipboard history manager

### Writing

#### `Clipboard.write(text: string) -> Async<bool>`

Write text to clipboard.

```fsharp
async {
    let! success = Clipboard.write "Hello from plugin!"
    if success then
        ctx.NotifySuccess "Copied" "Text copied to clipboard"
    else
        ctx.NotifyError "Failed" "Could not access clipboard"
}
```

**Returns:** `true` if successful, `false` if failed

#### `Clipboard.clear() -> Async<bool>`

Clear clipboard contents.

```fsharp
async {
    let! success = Clipboard.clear()
    if success then
        ctx.NotifyInfo "Cleared" "Clipboard cleared"
}
```

## File System Utilities

**Requires:** `filesystem.read` or `filesystem.write` permission

### Reading

#### `FileSystem.readText(path: string) -> Async<string option>`

Read entire file as text.

```fsharp
async {
    let! content = FileSystem.readText "~/.gitconfig"
    match content with
    | Some text -> parseGitConfig text
    | None -> ctx.NotifyError "Error" "Could not read .gitconfig"
}
```

**Returns:** `Some content` if successful, `None` if error

#### `FileSystem.readLines(path: string) -> Async<string list option>`

Read file as lines.

```fsharp
async {
    let! lines = FileSystem.readLines "~/.bash_history"
    match lines with
    | Some history -> processHistory history
    | None -> ctx.NotifyError "Error" "Could not read history"
}
```

#### `FileSystem.exists(path: string) -> Async<bool>`

Check if file exists.

```fsharp
async {
    let! exists = FileSystem.exists "~/.ssh/config"
    if exists then
        parseSshConfig()
    else
        ctx.NotifyWarning "Not Found" "SSH config not found"
}
```

#### `FileSystem.listDirectory(path: string) -> Async<string list option>`

List directory contents.

```fsharp
async {
    let! files = FileSystem.listDirectory "~/.config/scarab/plugins"
    match files with
    | Some fileList ->
        fileList |> List.iter (fun f -> ctx.Log Info f)
    | None ->
        ctx.NotifyError "Error" "Could not list directory"
}
```

**Returns:** List of file/directory names (not full paths)

### Writing

#### `FileSystem.writeText(path: string, content: string) -> Async<bool>`

Write text to file.

```fsharp
async {
    let! success = FileSystem.writeText cachePath data
    if success then
        ctx.Log Info "Cache written"
    else
        ctx.NotifyError "Error" "Could not write cache"
}
```

**Returns:** `true` if successful, `false` if error

#### `FileSystem.appendText(path: string, content: string) -> Async<bool>`

Append text to file.

```fsharp
async {
    let! success = FileSystem.appendText logPath entry
    if not success then
        ctx.NotifyError "Error" "Could not write log"
}
```

#### `FileSystem.delete(path: string) -> Async<bool>`

Delete file.

```fsharp
async {
    let! success = FileSystem.delete tempFile
    if success then
        ctx.Log Debug "Temp file deleted"
}
```

**Warning:** Cannot be undone

### Path Utilities

#### `Path.join(parts: string list) -> string`

Join path components.

```fsharp
let configPath = Path.join [home; ".config"; "scarab"; "config.toml"]
// "/home/user/.config/scarab/config.toml"
```

#### `Path.normalize(path: string) -> string`

Normalize path (resolve `~`, `.`, `..`).

```fsharp
let normalized = Path.normalize "~/.config/../.bashrc"
// "/home/user/.bashrc"
```

#### `Path.basename(path: string) -> string`

Get filename from path.

```fsharp
let name = Path.basename "/path/to/file.txt"
// "file.txt"
```

#### `Path.dirname(path: string) -> string`

Get directory from path.

```fsharp
let dir = Path.dirname "/path/to/file.txt"
// "/path/to"
```

#### `Path.extension(path: string) -> string`

Get file extension.

```fsharp
let ext = Path.extension "file.txt"
// ".txt"
```

## Network Utilities

**Requires:** `network.http` permission

### HTTP Requests

#### `Http.get(url: string) -> Async<HttpResponse option>`

Make GET request.

```fsharp
async {
    let! response = Http.get "https://api.github.com/user"
    match response with
    | Some resp ->
        if resp.Status = 200 then
            ctx.Log Info resp.Body
        else
            ctx.NotifyError "Error" (sprintf "HTTP %d" resp.Status)
    | None ->
        ctx.NotifyError "Error" "Request failed"
}
```

**HttpResponse type:**
```fsharp
type HttpResponse = {
    Status: int
    Headers: Map<string, string>
    Body: string
}
```

#### `Http.post(url: string, body: string) -> Async<HttpResponse option>`

Make POST request.

```fsharp
async {
    let json = """{"title": "Test Issue"}"""
    let! response = Http.post "https://api.github.com/issues" json
    match response with
    | Some resp -> processResponse resp
    | None -> ctx.NotifyError "Error" "POST failed"
}
```

#### `Http.postJson(url: string, data: 'T) -> Async<HttpResponse option>`

POST JSON data (automatic serialization).

```fsharp
type Issue = { Title: string; Body: string }

async {
    let issue = { Title = "Bug"; Body = "Description" }
    let! response = Http.postJson "https://api.github.com/issues" issue
}
```

#### `Http.withHeaders(headers: (string * string) list, request: Async<HttpResponse option>) -> Async<HttpResponse option>`

Add headers to request.

```fsharp
async {
    let! response =
        Http.get "https://api.github.com/user"
        |> Http.withHeaders [
            ("Authorization", "Bearer " + token)
            ("Accept", "application/json")
        ]
}
```

### JSON Utilities

#### `Json.parse<'T>(json: string) -> 'T option`

Parse JSON string.

```fsharp
type User = { Name: string; Email: string }

match Json.parse<User> jsonString with
| Some user ->
    ctx.Log Info (sprintf "User: %s (%s)" user.Name user.Email)
| None ->
    ctx.NotifyError "Error" "Invalid JSON"
```

#### `Json.stringify(data: 'T) -> string`

Convert to JSON string.

```fsharp
let user = { Name = "Alice"; Email = "alice@example.com" }
let json = Json.stringify user
// """{"name":"Alice","email":"alice@example.com"}"""
```

## Date/Time Utilities

### Formatting

#### `DateTime.format(format: string, date: DateTime) -> string`

Format date/time.

```fsharp
let formatted = DateTime.format "yyyy-MM-dd HH:mm:ss" DateTime.Now
// "2025-12-02 14:30:45"
```

**Common formats:**
- `"yyyy-MM-dd"` - Date: 2025-12-02
- `"HH:mm:ss"` - Time: 14:30:45
- `"MMM dd, yyyy"` - Month: Dec 02, 2025

#### `DateTime.toIso(date: DateTime) -> string`

Convert to ISO 8601 format.

```fsharp
let iso = DateTime.toIso DateTime.Now
// "2025-12-02T14:30:45.123Z"
```

#### `DateTime.fromIso(iso: string) -> DateTime option`

Parse ISO 8601 date.

```fsharp
match DateTime.fromIso "2025-12-02T14:30:45Z" with
| Some date -> useDate date
| None -> ctx.NotifyError "Error" "Invalid date format"
```

### Relative Time

#### `DateTime.relative(date: DateTime) -> string`

Get human-readable relative time.

```fsharp
let ago = DateTime.relative pastDate
// "5 minutes ago"
// "2 hours ago"
// "3 days ago"
```

#### `DateTime.duration(start: DateTime, end: DateTime) -> string`

Get human-readable duration.

```fsharp
let duration = DateTime.duration startTime endTime
// "2h 34m 12s"
```

## Complete Example

Here's a plugin using various utilities:

```fsharp
module url_monitor

open Scarab.PluginApi
open Scarab.PluginApi.Utilities

let urlRegex = @"https?://[^\s]+"
let mutable detectedUrls = []
let mutable lastNotification = DateTime.MinValue

// Debounced notification to avoid spam
let notifyUrls = Async.debounce 1000 (fun () ->
    let count = List.length detectedUrls
    if count > 0 then
        ctx.NotifyInfo "URLs Detected" (sprintf "Found %d URLs" count)
)

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Early return if no URLs
        if not (String.contains "http" line) then
            return Continue

        // Extract URLs
        let urls = String.extract urlRegex line

        if urls.Length > 0 then
            // Add to list
            detectedUrls <- detectedUrls @ urls

            // Truncate for display
            let truncated = urls |> List.map (String.truncate 50)

            // Log with timestamp
            let timestamp = DateTime.format "HH:mm:ss" DateTime.Now
            ctx.Log Info (sprintf "[%s] Found %d URLs" timestamp urls.Length)

            // Debounced notification
            notifyUrls()

            // Copy first URL to clipboard if permission granted
            match urls with
            | first :: _ ->
                let! success = Clipboard.write first
                if success then
                    ctx.Log Debug "URL copied to clipboard"
            | [] -> ()

        return Continue
    }

[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    async {
        // Ctrl+U - Show URL list
        if key.Ctrl && key.Code = KeyCode.U then
            let uniqueUrls = detectedUrls |> List.distinct

            ctx.ShowModal (
                Modal(
                    id = "url-list",
                    title = Some "Detected URLs",
                    width = Fixed 80,
                    content = List(
                        items = uniqueUrls,
                        render = fun url ->
                            Container(
                                layout = Horizontal,
                                children = [
                                    Text(String.truncate 60 url, width = Flex 1)
                                    Button("Copy",
                                           onClick = fun () ->
                                               async {
                                                   let! _ = Clipboard.write url
                                                   ctx.NotifySuccess "Copied" "URL copied"
                                               } |> Async.Start,
                                           style = Primary)
                                ]
                            ),
                        height = Fixed 15,
                        emptyMessage = Some "No URLs detected yet"
                    )
                )
            )

            return Stop  // Prevent default Ctrl+U

        return Continue
    }
```

## Performance Tips

1. **String operations** - Use built-in utilities for better performance
2. **Regex** - Cache compiled regex patterns
3. **Async** - Use `Async.parallel` for independent operations
4. **Debouncing** - Rate limit expensive operations
5. **File I/O** - Cache file contents when possible
6. **HTTP** - Implement retry logic and timeouts

## Next Steps

- **[RemoteUI Components](remote-ui.md)** - Build user interfaces
- **[Plugin Context](plugin-context.md)** - Core API reference
- **[Hooks Reference](hooks.md)** - Available hooks
- **[Examples](../../plugins/examples/)** - Working examples

## Getting Help

- API Source: `crates/scarab-plugin-api/src/utilities.rs`
- Examples: `plugins/examples/`
- Issues: GitHub Issues
- Discussions: GitHub Discussions

---

**Remember:** These utilities are optimized for terminal plugin use cases. Always check performance in realistic scenarios and use appropriate rate limiting.
