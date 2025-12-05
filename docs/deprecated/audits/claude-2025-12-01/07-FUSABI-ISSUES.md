# Fusabi-lang Feature Requests
**Date:** December 1, 2025
**Context:** Scarab Plugin System vs WezTerm Lua Capabilities

---

## Executive Summary

WezTerm's Lua configuration provides **200+ options and 12+ event hooks** that plugin developers rely on. Fusabi currently provides **basic keybindings only**, creating a significant feature gap.

**These issues track the roadmap to feature parity with wezterm.**

---

## Issue #1: Event System Foundation

### Title
**[Feature Request] Implement event hook system for terminal lifecycle events**

### Description

**Problem:**
Wezterm allows plugins to hook into terminal lifecycle events:
- `gui-startup`, `gui-attached`, `mux-startup`
- `window-focus-changed`, `window-resized`, `window-config-reloaded`
- `bell`, `open-uri`, `user-var-changed`

Fusabi has **no event system**, making it impossible to react to terminal state changes.

**Proposed API:**

```fsharp
// In fusabi-lang core
type Event =
    | Startup
    | Shutdown
    | WindowFocusChanged of gained: bool
    | WindowResized of cols: int * rows: int
    | Bell
    | UrlDetected of url: string
    | CommandStarted of cmd: string
    | CommandCompleted of exitCode: int

type EventHandler<'T> = Event -> 'T -> Async<'T>

module Events =
    val on : Event -> EventHandler<'T> -> unit
```

**Usage in Scarab Plugins:**

```fsharp
open Fusabi.Events

// React to window focus
Events.on WindowFocusChanged (fun (gained, state) ->
    async {
        if gained then
            state.RefreshGitStatus()
        return state
    }
)

// Detect URLs
Events.on (UrlDetected url) (fun (url, state) ->
    async {
        state.AddToLinkHints(url)
        return state
    }
)

// Track commands
Events.on (CommandCompleted exitCode) (fun (code, state) ->
    async {
        if code <> 0 then
            state.ShowErrorNotification()
        return state
    }
)
```

**Benefits:**
- Enables reactive plugins
- Matches wezterm capabilities
- Foundation for advanced features

**Implementation Notes:**
- Use F# `Event<'T>` or `IObservable<'T>`
- Async by default for I/O-heavy handlers
- Consider performance (event frequency)

**Priority:** 🔴 **CRITICAL** (blocks most plugin use cases)

**Effort:** 2-3 weeks for foundation

---

## Issue #2: Terminal State Queries

### Title
**[Feature Request] Add API to query terminal state and process information**

### Description

**Problem:**
WezTerm exposes:
- `pane:get_foreground_process_name()`
- `pane:get_lines_as_text()`
- `pane:get_current_working_dir()`
- Window/tab metadata

Fusabi plugins currently **cannot query terminal state**, limiting functionality.

**Proposed API:**

```fsharp
module TerminalInfo =
    // Process information
    val getForegroundProcess : unit -> Async<ProcessInfo option>
    val getCurrentWorkingDir : unit -> Async<string option>

    // Terminal content
    val getLine : lineNum: int -> Async<string>
    val getLines : startLine: int -> endLine: int -> Async<string[]>
    val getCellAt : x: int -> y: int -> Async<Cell>

    // Window/tab metadata
    val getWindowTitle : unit -> string
    val getTabTitle : unit -> string
    val getTerminalSize : unit -> (int * int) // (cols, rows)

type ProcessInfo = {
    Name: string
    Pid: int
    CommandLine: string option
}
```

**Usage:**

```fsharp
// Git status plugin
let updateGitStatus() =
    async {
        let! cwd = TerminalInfo.getCurrentWorkingDir()
        match cwd with
        | Some dir when isGitRepo dir ->
            let! status = getGitStatus dir
            drawStatusOverlay status
        | _ -> ()
    }

// Smart completion
let getCompletionContext() =
    async {
        let! currentLine = TerminalInfo.getLine -1 // Last line
        let! process = TerminalInfo.getForegroundProcess()

        match process with
        | Some { Name = "git" } -> return GitCompletions currentLine
        | Some { Name = "cargo" } -> return CargoCompletions currentLine
        | _ -> return []
    }
```

**Priority:** 🔴 **CRITICAL** (essential for context-aware plugins)

**Effort:** 2 weeks

---

## Issue #3: Programmatic Control API

### Title
**[Feature Request] Add API for programmatic pane/window control**

### Description

**Problem:**
WezTerm allows plugins to:
- Send text to terminal (`pane:send_text()`)
- Split panes programmatically
- Spawn new tabs/windows
- Control focus

Fusabi has **no programmatic control**, limiting automation capabilities.

**Proposed API:**

```fsharp
module TerminalControl =
    // Input injection
    val sendText : string -> Async<unit>
    val sendKeys : Key[] -> Async<unit>

    // Pane management
    val splitHorizontal : unit -> Async<PaneId>
    val splitVertical : unit -> Async<PaneId>
    val closePane : PaneId -> Async<unit>
    val focusPane : PaneId -> Async<unit>

    // Tab management
    val createTab : unit -> Async<TabId>
    val closeTab : TabId -> Async<unit>
    val setTabTitle : TabId -> string -> Async<unit>

    // Notifications
    val showToast : string -> Async<unit>
    val showModal : ModalConfig -> Async<ModalResult>
```

**Usage:**

```fsharp
// Command runner plugin
let runInNewTab cmd =
    async {
        let! tabId = TerminalControl.createTab()
        do! TerminalControl.focusTab tabId
        do! TerminalControl.sendText (cmd + "\n")
        do! TerminalControl.setTabTitle tabId cmd
    }

// Auto-split for tools
let openToolsSplit() =
    async {
        let! paneId = TerminalControl.splitVertical()
        do! TerminalControl.sendText "htop\n"
    }

// Notifications
let notifyLongCommand duration =
    async {
        do! TerminalControl.showToast $"Command took {duration}s"
    }
```

**Priority:** 🟡 **HIGH** (needed for advanced automation)

**Effort:** 3 weeks

---

## Issue #4: Status Bar and UI Formatting API

### Title
**[Feature Request] Add API for status bar and UI element formatting**

### Description

**Problem:**
WezTerm allows custom formatting of:
- Tab titles (`format-tab-title` event)
- Window titles (`format-window-title` event)
- Status line (`update-status`, `update-right-status` events)

Fusabi has **no UI formatting API**, limiting visual customization.

**Proposed API:**

```fsharp
module UIFormatting =
    type StatusSegment = {
        Text: string
        FgColor: Color option
        BgColor: Color option
        Bold: bool
    }

    type TabFormat = {
        Title: string
        Active: bool
        Segments: StatusSegment[]
    }

    // Callbacks
    val onFormatTab : (TabInfo -> TabFormat) -> unit
    val onFormatStatusLeft : (StatusInfo -> StatusSegment[]) -> unit
    val onFormatStatusRight : (StatusInfo -> StatusSegment[]) -> unit

type TabInfo = {
    Index: int
    Title: string
    Active: bool
    HasActivity: bool
}

type StatusInfo = {
    CurrentTab: int
    TotalTabs: int
    Time: DateTime
    CustomData: Map<string, string>
}
```

**Usage:**

```fsharp
// Git-aware tab formatting
UIFormatting.onFormatTab (fun tabInfo ->
    let gitBranch = getGitBranch()

    { Title = tabInfo.Title
      Active = tabInfo.Active
      Segments = [
          { Text = tabInfo.Title; FgColor = None; BgColor = None; Bold = tabInfo.Active }
          if gitBranch.IsSome then
              { Text = $" [{gitBranch.Value}]"; FgColor = Some Green; BgColor = None; Bold = false }
      ]
    }
)

// Custom status bar
UIFormatting.onFormatStatusRight (fun info ->
    [
        { Text = info.Time.ToString("HH:mm"); FgColor = Some White; BgColor = None; Bold = false }
        { Text = $" [{info.CurrentTab + 1}/{info.TotalTabs}]"; FgColor = Some Cyan; BgColor = None; Bold = false }
    ]
)
```

**Priority:** 🟡 **HIGH** (important for UX)

**Effort:** 2 weeks

---

## Issue #5: Configuration Schema and Validation

### Title
**[Feature Request] Add typed configuration schema with validation**

### Description

**Problem:**
WezTerm has 200+ typed configuration options. Fusabi needs a clean way to define and validate plugin configurations.

**Proposed API:**

```fsharp
module Config =
    type ConfigValue =
        | String of string
        | Int of int
        | Bool of bool
        | List of ConfigValue[]
        | Map of Map<string, ConfigValue>

    type ConfigSchema = {
        Name: string
        Type: ConfigValueType
        Default: ConfigValue option
        Validator: (ConfigValue -> Result<unit, string>) option
    }

    val define : ConfigSchema[] -> unit
    val get : string -> ConfigValue option
    val getOr : string -> ConfigValue -> ConfigValue
```

**Usage:**

```fsharp
// Define plugin config
Config.define [
    { Name = "git_status.enabled"
      Type = BoolType
      Default = Some (Bool true)
      Validator = None }

    { Name = "git_status.refresh_interval_ms"
      Type = IntType
      Default = Some (Int 1000)
      Validator = Some (fun v ->
          match v with
          | Int n when n >= 100 -> Ok ()
          | _ -> Error "Interval must be >= 100ms")
    }
]

// Use config
let enabled = Config.getOr "git_status.enabled" (Bool true) |> asBool
let interval = Config.getOr "git_status.refresh_interval_ms" (Int 1000) |> asInt
```

**Priority:** 🟢 **MEDIUM** (quality of life)

**Effort:** 1-2 weeks

---

## Issue #6: Command Palette Extension API

### Title
**[Feature Request] Allow plugins to register custom commands in command palette**

### Description

**Problem:**
WezTerm's `augment-command-palette` event allows plugins to add commands. Fusabi plugins **cannot extend the command palette**, limiting discoverability.

**Proposed API:**

```fsharp
module Commands =
    type Command = {
        Id: string
        Name: string
        Description: string
        Category: string
        Handler: unit -> Async<unit>
    }

    val register : Command -> unit
    val registerMany : Command[] -> unit
```

**Usage:**

```fsharp
// Git plugin commands
Commands.registerMany [
    { Id = "git.status"
      Name = "Git: Show Status"
      Description = "Display current git status"
      Category = "Git"
      Handler = fun () -> showGitStatus() }

    { Id = "git.commit"
      Name = "Git: Commit Changes"
      Description = "Open commit message editor"
      Category = "Git"
      Handler = fun () -> openCommitEditor() }

    { Id = "git.push"
      Name = "Git: Push to Remote"
      Description = "Push changes to remote repository"
      Category = "Git"
      Handler = fun () -> gitPush() }
]
```

**Priority:** 🟢 **MEDIUM** (nice to have)

**Effort:** 1 week

---

## Issue #7: Standard Library Enhancements

### Title
**[Feature Request] Add standard library modules for common plugin operations**

### Description

**Problem:**
WezTerm includes modules for:
- `wezterm.time` - Time/date utilities
- `wezterm.json` - JSON parsing
- `wezterm.url` - URL parsing
- `wezterm.glob` - Pattern matching

Fusabi should provide similar utilities to avoid plugin boilerplate.

**Proposed Modules:**

```fsharp
// Fusabi.Time
module Time =
    val now : unit -> DateTime
    val sleep : milliseconds: int -> Async<unit>
    val debounce : milliseconds: int -> (unit -> Async<unit>) -> (unit -> Async<unit>)

// Fusabi.Json
module Json =
    val parse : string -> Result<JsonValue, string>
    val stringify : JsonValue -> string
    val tryGet : path: string -> JsonValue -> JsonValue option

// Fusabi.Url
module Url =
    val parse : string -> Result<UrlParts, string>
    val isValid : string -> bool
    val encode : string -> string

// Fusabi.Process
module Process =
    val run : cmd: string -> args: string[] -> Async<ProcessResult>
    val runShell : script: string -> Async<ProcessResult>

type ProcessResult = {
    ExitCode: int
    Stdout: string
    Stderr: string
}
```

**Priority:** 🟢 **MEDIUM** (developer experience)

**Effort:** 2-3 weeks for all modules

---

## Summary Table

| Issue | Priority | Effort | Blocks Scarab? |
|-------|----------|--------|----------------|
| #1 Event System | 🔴 CRITICAL | 2-3 weeks | YES |
| #2 Terminal Queries | 🔴 CRITICAL | 2 weeks | YES |
| #3 Programmatic Control | 🟡 HIGH | 3 weeks | PARTIALLY |
| #4 UI Formatting | 🟡 HIGH | 2 weeks | NO (workaround exists) |
| #5 Config Schema | 🟢 MEDIUM | 1-2 weeks | NO |
| #6 Command Palette | 🟢 MEDIUM | 1 week | NO |
| #7 Stdlib Enhancements | 🟢 MEDIUM | 2-3 weeks | NO |

**Total Effort for MVP (Issues #1-4):** ~9-10 weeks

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-5)
- Issue #1: Event System (2-3 weeks)
- Issue #2: Terminal Queries (2 weeks)

**Deliverable:** Reactive plugins possible

### Phase 2: Control & UI (Week 6-11)
- Issue #3: Programmatic Control (3 weeks)
- Issue #4: UI Formatting (2 weeks)
- Issue #5: Config Schema (1 week)

**Deliverable:** Feature parity with wezterm basics

### Phase 3: Polish (Week 12-15)
- Issue #6: Command Palette (1 week)
- Issue #7: Stdlib (2-3 weeks)

**Deliverable:** Complete plugin ecosystem

---

## Contribution Offer

The Scarab team can contribute:
- ✅ Use case documentation
- ✅ Example plugins for each feature
- ✅ Integration testing
- ✅ Migration guides for plugin developers

**Contact:** https://github.com/fusabi-lang/fusabi/discussions

---

**Document:** 07-FUSABI-ISSUES.md
**Status:** Ready to file upstream
**Next Step:** File issues in fusabi-lang/fusabi repository
