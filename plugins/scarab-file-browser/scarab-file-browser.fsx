module scarab_file_browser

open Scarab.PluginApi
open System.IO
open System.Diagnostics

[<Plugin>]
let metadata = {
    Name = "scarab-file-browser"
    Version = "0.1.0"
    Description = "Interactive file and directory browser with hints"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "📁"
    Color = Some "#4CAF50"
    Catchphrase = Some "Navigate your filesystem at the speed of thought!"
}

// Hint keys for directory entries (a-l for 12 items)
let hintKeys = [| "a"; "s"; "d"; "f"; "g"; "h"; "j"; "k"; "l" |]

// File browser state
type BrowserState = {
    CurrentPath: string
    Entries: (string * bool) array  // (name, isDirectory)
    SelectedIndex: int
    Visible: bool
}

// Initialize browser state in plugin data
let initBrowserState (ctx: PluginContext) (path: string) =
    ctx.SetData "browser_path" path
    ctx.SetData "browser_visible" "true"
    ctx.SetData "browser_selected" "0"

// Get current browser state
let getBrowserState (ctx: PluginContext) : BrowserState option =
    match ctx.GetData "browser_visible" with
    | Some "true" ->
        let path = ctx.GetData "browser_path" |> Option.defaultValue (Environment.GetEnvironmentVariable("HOME"))
        let selectedStr = ctx.GetData "browser_selected" |> Option.defaultValue "0"
        let selected = int selectedStr

        // Read directory entries
        try
            let entries =
                if Directory.Exists(path) then
                    let dirs = Directory.GetDirectories(path)
                                |> Array.map (fun d -> (Path.GetFileName(d), true))
                    let files = Directory.GetFiles(path)
                                |> Array.map (fun f -> (Path.GetFileName(f), false))
                    Array.append dirs files
                else
                    [| |]

            Some {
                CurrentPath = path
                Entries = entries
                SelectedIndex = selected
                Visible = true
            }
        with
        | ex ->
            ctx.Log Error (sprintf "Failed to read directory %s: %s" path ex.Message)
            None
    | _ -> None

// Close the browser
let closeBrowser (ctx: PluginContext) =
    ctx.SetData "browser_visible" "false"

// Navigate to a directory
let navigateToDirectory (ctx: PluginContext) (path: string) =
    if Directory.Exists(path) then
        ctx.SetData "browser_path" path
        ctx.SetData "browser_selected" "0"
        ctx.Log Info (sprintf "Navigated to: %s" path)

        // Send cd command to terminal
        ctx.SendCommand (sprintf "cd \"%s\"" path)
        ctx.NotifyInfo "Directory" (sprintf "📁 %s" path)
    else
        ctx.NotifyError "Error" (sprintf "Directory does not exist: %s" path)

// Open a file in $EDITOR
let openFile (ctx: PluginContext) (filePath: string) =
    if File.Exists(filePath) then
        let editor = Environment.GetEnvironmentVariable("EDITOR") |> Option.ofObj |> Option.defaultValue "nvim"
        ctx.Log Info (sprintf "Opening file in %s: %s" editor filePath)

        // Send command to open file in editor
        ctx.SendCommand (sprintf "%s \"%s\"" editor filePath)
        closeBrowser ctx
    else
        ctx.NotifyError "Error" (sprintf "File does not exist: %s" filePath)

// Handle hint selection
let handleHintSelection (ctx: PluginContext) (hintKey: string) =
    match getBrowserState ctx with
    | Some state ->
        // Find index of hint key
        let hintIndex = Array.tryFindIndex ((=) hintKey) hintKeys
        match hintIndex with
        | Some idx when idx < state.Entries.Length ->
            let (name, isDir) = state.Entries.[idx]
            let fullPath = Path.Combine(state.CurrentPath, name)

            if isDir then
                navigateToDirectory ctx fullPath
            else
                openFile ctx fullPath
        | _ ->
            ctx.Log Warning (sprintf "Invalid hint key: %s" hintKey)
    | None ->
        ctx.Log Warning "Browser not visible"

// Render browser UI using fusabi-tui-runtime
let renderBrowser (ctx: PluginContext) (state: BrowserState) =
    // TODO: This will use fusabi-tui-runtime widgets when integrated
    // For now, we'll use notifications to show the browser state

    let entries = state.Entries |> Array.take (min state.Entries.Length hintKeys.Length)
    let lines =
        entries
        |> Array.mapi (fun i (name, isDir) ->
            let icon = if isDir then "📁" else "📄"
            let hint = hintKeys.[i]
            sprintf "[%s] %s %s" hint icon name
        )
        |> String.concat "\n"

    ctx.Log Debug (sprintf "Browser: %s\n%s" state.CurrentPath lines)

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "File Browser plugin loaded!"

        // Initialize with home directory
        let home = Environment.GetEnvironmentVariable("HOME")
        ctx.SetData "browser_path" home
        ctx.SetData "browser_visible" "false"
        ctx.SetData "browser_selected" "0"

        return Ok ()
    }

[<OnCommand("browse")>]
let onBrowseCommand (ctx: PluginContext) (args: string array) =
    async {
        // Open browser at current directory or specified path
        let path =
            if args.Length > 0 then
                args.[0]
            else
                ctx.GetData "browser_path" |> Option.defaultValue (Environment.GetEnvironmentVariable("HOME"))

        initBrowserState ctx path

        match getBrowserState ctx with
        | Some state ->
            renderBrowser ctx state
            ctx.NotifyInfo "File Browser" (sprintf "Browsing: %s" state.CurrentPath)
        | None ->
            ctx.NotifyError "Error" "Failed to open file browser"

        return Continue
    }

[<OnCommand("picker")>]
let onPickerCommand (ctx: PluginContext) (args: string array) =
    async {
        // Open directory picker at specified path
        let path =
            if args.Length > 0 then
                args.[0]
            else
                Environment.CurrentDirectory

        if Directory.Exists(path) then
            initBrowserState ctx path

            match getBrowserState ctx with
            | Some state ->
                renderBrowser ctx state
                ctx.NotifyInfo "Directory Picker" (sprintf "📁 %s" path)
            | None ->
                ctx.NotifyError "Error" "Failed to open directory picker"
        else
            ctx.NotifyError "Error" (sprintf "Directory not found: %s" path)

        return Continue
    }

[<OnCommand("hint")>]
let onHintCommand (ctx: PluginContext) (args: string array) =
    async {
        // Handle hint selection
        if args.Length > 0 then
            handleHintSelection ctx args.[0]

        return Continue
    }

[<OnStatusBar>]
let onStatusBar (ctx: PluginContext) =
    async {
        // Show browser state in status bar if visible
        match getBrowserState ctx with
        | Some state when state.Visible ->
            return [
                RenderItem.Icon "📁"
                RenderItem.Text " "
                RenderItem.Text (Path.GetFileName(state.CurrentPath))
                RenderItem.Text (sprintf " (%d items)" state.Entries.Length)
            ]
        | _ ->
            return []
    }
