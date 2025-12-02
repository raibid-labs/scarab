# Tutorial 5: Frontend UI with RemoteUI

In this tutorial, you'll learn how to build rich, interactive user interfaces for Scarab using the RemoteUI system. We'll create a command palette plugin that demonstrates all major UI components and interaction patterns.

**What you'll learn:**
- RemoteUI architecture and components
- Building overlays, modals, and notifications
- Handling user input (keyboard and mouse)
- Styling and positioning UI elements
- Creating responsive, performant UIs
- Animation and transition patterns

**Time:** 60 minutes

## Prerequisites

- Completed [Tutorial 3: Plugin API Deep Dive](03-plugin-api-deep-dive.md)
- Understanding of async/await in F#
- Basic knowledge of UI/UX principles

## What is RemoteUI?

RemoteUI is Scarab's framework for building terminal UIs from plugins. Unlike traditional TUI libraries, RemoteUI:

- **Runs in the frontend** - No backend rendering overhead
- **GPU-accelerated** - Uses Bevy's rendering pipeline
- **Declarative** - Describe UI state, not draw commands
- **Event-driven** - Reactive to user input
- **Type-safe** - F# type checking at compile time

### Architecture

```
Frontend Plugin (.fsx)
    ↓
RemoteUI Components
    ↓
Bevy ECS (Entity Component System)
    ↓
GPU Rendering
```

## RemoteUI Components

Scarab provides these UI building blocks:

| Component | Purpose | Example Use |
|-----------|---------|-------------|
| `Overlay` | Non-blocking text at fixed position | Status line, clock, indicators |
| `Modal` | Blocking dialog with focus | Command palette, file picker |
| `Notification` | Temporary message | Success/error alerts |
| `Input` | Text input field | Search box, command entry |
| `List` | Scrollable item list | Command list, file list |
| `Button` | Clickable action | OK/Cancel, menu items |

## Part 1: Building a Command Palette

Let's build a complete command palette plugin that demonstrates all RemoteUI features.

### Step 1: Create the Plugin

```bash
just plugin-new command-palette frontend
cd plugins/command-palette
```

### Step 2: Define Commands

Open `command-palette.fsx` and define the command structure:

```fsharp
module command_palette

open Scarab.PluginApi
open Scarab.RemoteUI
open System

[<Plugin>]
let metadata = {
    Name = "command-palette"
    Version = "1.0.0"
    Description = "Command palette for quick actions"
    Author = "Your Name"
    Emoji = Some "⌘"
    Color = Some "#6366F1"
    Catchphrase = Some "Command at your fingertips!"
}

// Command definition
type Command = {
    Id: string
    Name: string
    Description: string
    Category: string
    Icon: string
    Action: unit -> Async<unit>
}

// Sample commands
let commands = [
    {
        Id = "new-tab"
        Name = "New Tab"
        Description = "Open a new terminal tab"
        Category = "Tab"
        Icon = "➕"
        Action = fun () -> async { (* tab logic *) }
    }
    {
        Id = "split-vertical"
        Name = "Split Vertical"
        Description = "Split the terminal vertically"
        Category = "Window"
        Icon = "┃"
        Action = fun () -> async { (* split logic *) }
    }
    {
        Id = "split-horizontal"
        Name = "Split Horizontal"
        Description = "Split the terminal horizontally"
        Category = "Window"
        Icon = "━"
        Action = fun () -> async { (* split logic *) }
    }
    {
        Id = "settings"
        Name = "Open Settings"
        Description = "Open Scarab settings"
        Category = "System"
        Icon = "⚙"
        Action = fun () -> async { (* settings logic *) }
    }
    {
        Id = "plugins"
        Name = "Manage Plugins"
        Description = "Enable/disable plugins"
        Category = "System"
        Icon = "🔌"
        Action = fun () -> async { (* plugin manager *) }
    }
]
```

### Step 3: Create the Modal UI

Now let's create a modal to display the command palette:

```fsharp
// State management
let mutable isOpen = false
let mutable selectedIndex = 0
let mutable searchQuery = ""
let mutable filteredCommands = commands

// Filter commands by search query
let filterCommands (query: string) =
    if String.IsNullOrWhiteSpace(query) then
        commands
    else
        commands
        |> List.filter (fun cmd ->
            cmd.Name.ToLower().Contains(query.ToLower()) ||
            cmd.Description.ToLower().Contains(query.ToLower())
        )

// Build the modal UI
let buildPaletteUI (ctx: PluginContext) =
    // Update filtered list
    filteredCommands <- filterCommands searchQuery

    // Create the modal
    let modal = {
        Id = "command-palette-modal"
        Title = "Command Palette"
        Width = 60
        Height = 20
        X = None  // Centered
        Y = None  // Centered
        Style = ModalStyle.Default
        FocusOnOpen = true
    }

    // Create the search input
    let searchInput = {
        Id = "search-input"
        Placeholder = "Type to search..."
        Value = searchQuery
        Width = 56
        X = 2
        Y = 2
        Style = InputStyle.Default
        OnChange = fun newValue ->
            searchQuery <- newValue
            selectedIndex <- 0
            buildPaletteUI ctx  // Rebuild UI
    }

    // Create the command list
    let listItems =
        filteredCommands
        |> List.mapi (fun i cmd ->
            {
                Id = cmd.Id
                Label = sprintf "%s %s" cmd.Icon cmd.Name
                Description = Some cmd.Description
                IsSelected = (i = selectedIndex)
                OnSelect = fun () ->
                    async {
                        // Close palette and execute command
                        isOpen <- false
                        ctx.QueueCommand (RemoteCommand.CloseModal { Id = modal.Id })
                        do! cmd.Action()
                    }
            }
        )

    let commandList = {
        Id = "command-list"
        Items = listItems
        Width = 56
        Height = 14
        X = 2
        Y = 5
        Style = ListStyle.Default
        SelectedIndex = selectedIndex
    }

    // Draw all components
    ctx.QueueCommand (RemoteCommand.ShowModal modal)
    ctx.QueueCommand (RemoteCommand.DrawInput searchInput)
    ctx.QueueCommand (RemoteCommand.DrawList commandList)
```

### Step 4: Handle Keyboard Input

Add keyboard navigation:

```fsharp
[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    async {
        // Toggle palette with Ctrl+P
        if key.Code = KeyCode.Char 'p' && key.Modifiers.Contains(KeyModifier.Control) then
            if isOpen then
                // Close palette
                isOpen <- false
                ctx.QueueCommand (RemoteCommand.CloseModal { Id = "command-palette-modal" })
            else
                // Open palette
                isOpen <- true
                searchQuery <- ""
                selectedIndex <- 0
                buildPaletteUI ctx

            return Stop  // Consume the key event

        // Handle navigation when palette is open
        elif isOpen then
            match key.Code with
            | KeyCode.Escape ->
                // Close on Escape
                isOpen <- false
                ctx.QueueCommand (RemoteCommand.CloseModal { Id = "command-palette-modal" })
                return Stop

            | KeyCode.Up ->
                // Navigate up
                selectedIndex <- max 0 (selectedIndex - 1)
                buildPaletteUI ctx
                return Stop

            | KeyCode.Down ->
                // Navigate down
                selectedIndex <- min (filteredCommands.Length - 1) (selectedIndex + 1)
                buildPaletteUI ctx
                return Stop

            | KeyCode.Enter ->
                // Execute selected command
                if selectedIndex < filteredCommands.Length then
                    let cmd = filteredCommands.[selectedIndex]
                    isOpen <- false
                    ctx.QueueCommand (RemoteCommand.CloseModal { Id = "command-palette-modal" })
                    do! cmd.Action()
                return Stop

            | _ ->
                return Continue
        else
            return Continue
    }
```

## Part 2: UI Component Deep Dive

Let's explore each RemoteUI component in detail.

### Overlays

Overlays are **non-blocking** UI elements positioned at fixed coordinates.

```fsharp
// Simple text overlay
let statusOverlay = {
    Id = 1UL
    X = 0us
    Y = 0us
    Text = "Status: Connected"
    Style = OverlayStyle.Info
}

ctx.QueueCommand (RemoteCommand.DrawOverlay statusOverlay)

// Clear specific overlay
ctx.QueueCommand (RemoteCommand.ClearOverlays { Id = Some 1UL })

// Clear all overlays
ctx.QueueCommand (RemoteCommand.ClearOverlays { Id = None })
```

**Use cases:**
- Status indicators
- Clock display
- Resource monitors (CPU, memory)
- Connection status
- Mode indicators (vim-style)

**Best practices:**
- Keep text short (< 40 chars)
- Update sparingly (not every frame)
- Use consistent positioning
- Clear on plugin unload

### Modals

Modals are **blocking** dialogs that capture focus.

```fsharp
// Create a modal
let modal = {
    Id = "my-modal"
    Title = "Confirmation"
    Width = 50
    Height = 10
    X = Some 10  // Fixed position
    Y = Some 5
    Style = ModalStyle.Warning  // Info | Success | Warning | Error
    FocusOnOpen = true
}

ctx.QueueCommand (RemoteCommand.ShowModal modal)

// Close modal
ctx.QueueCommand (RemoteCommand.CloseModal { Id = "my-modal" })
```

**Modal Styles:**

```fsharp
type ModalStyle =
    | Default    // Blue border
    | Info       // Blue border, info icon
    | Success    // Green border, checkmark
    | Warning    // Orange border, warning icon
    | Error      // Red border, error icon
```

**Use cases:**
- Command palettes
- File pickers
- Confirmation dialogs
- Settings panels
- Search interfaces

### Input Fields

Text input fields for user input.

```fsharp
let input = {
    Id = "search-box"
    Placeholder = "Search..."
    Value = currentValue
    Width = 40
    X = 10
    Y = 5
    Style = InputStyle.Default
    OnChange = fun newValue ->
        // Handle input change
        ctx.Log Debug (sprintf "Input changed: %s" newValue)
        handleSearch newValue
}

ctx.QueueCommand (RemoteCommand.DrawInput input)
```

**Input Styles:**

```fsharp
type InputStyle =
    | Default      // Normal border
    | Focused      // Highlighted border
    | Error        // Red border
    | Disabled     // Grayed out
```

**Features:**
- Real-time change events
- Placeholder text
- Validation state (error styling)
- Cursor positioning
- Selection support

### Lists

Scrollable lists with selection.

```fsharp
let list = {
    Id = "file-list"
    Items = [
        { Id = "1"; Label = "file1.txt"; Description = Some "Modified today"; IsSelected = true; OnSelect = selectFile1 }
        { Id = "2"; Label = "file2.txt"; Description = Some "Modified yesterday"; IsSelected = false; OnSelect = selectFile2 }
        { Id = "3"; Label = "file3.txt"; Description = None; IsSelected = false; OnSelect = selectFile3 }
    ]
    Width = 40
    Height = 10
    X = 10
    Y = 5
    Style = ListStyle.Default
    SelectedIndex = 0
}

ctx.QueueCommand (RemoteCommand.DrawList list)
```

**List Features:**
- Keyboard navigation (up/down arrows)
- Mouse click selection
- Optional descriptions
- Custom icons
- Scrolling for long lists
- Multi-line item support

### Buttons

Clickable action buttons.

```fsharp
let okButton = {
    Id = "ok-button"
    Label = "OK"
    Width = 10
    X = 10
    Y = 15
    Style = ButtonStyle.Primary
    OnClick = fun () ->
        async {
            ctx.Log Info "OK clicked"
            // Perform action
        }
}

let cancelButton = {
    Id = "cancel-button"
    Label = "Cancel"
    Width = 10
    X = 22
    Y = 15
    Style = ButtonStyle.Secondary
    OnClick = fun () ->
        async {
            ctx.Log Info "Cancel clicked"
            // Close dialog
        }
}

ctx.QueueCommand (RemoteCommand.DrawButton okButton)
ctx.QueueCommand (RemoteCommand.DrawButton cancelButton)
```

**Button Styles:**

```fsharp
type ButtonStyle =
    | Primary      // Blue, bold
    | Secondary    // Gray, normal
    | Danger       // Red, for destructive actions
    | Success      // Green, for positive actions
    | Link         // No border, underlined
```

## Part 3: Styling and Layout

### Positioning

RemoteUI uses **character coordinates** (not pixels):

```fsharp
// Absolute positioning
let overlay = {
    X = 10us  // 10 columns from left
    Y = 5us   // 5 rows from top
    // ...
}

// Centered positioning (for modals)
let modal = {
    X = None  // Horizontally centered
    Y = None  // Vertically centered
    Width = 50
    Height = 20
    // ...
}

// Calculate dynamic position
let (cols, rows) = ctx.GetSize()
let rightAligned = {
    X = cols - 30us  // 30 chars from right
    Y = 0us
    // ...
}
```

### Colors

RemoteUI supports RGB colors:

```fsharp
type Color = {
    R: uint8
    G: uint8
    B: uint8
}

// Common colors
let colorRed = { R = 255uy; G = 0uy; B = 0uy }
let colorGreen = { R = 0uy; G = 255uy; B = 0uy }
let colorBlue = { R = 0uy; G = 0uy; B = 255uy }

// Use in styles
let styledOverlay = {
    Id = 1UL
    X = 0us
    Y = 0us
    Text = "Error!"
    Style = OverlayStyle.Custom {
        ForegroundColor = colorRed
        BackgroundColor = { R = 30uy; G = 30uy; B = 30uy }
        Bold = true
        Italic = false
    }
}
```

### Responsive Layout

Adapt UI to terminal size:

```fsharp
[<OnResize>]
let onResize (ctx: PluginContext) (cols: u16) (rows: u16) =
    async {
        // Recalculate modal size
        let modalWidth = min 80 (cols - 4)
        let modalHeight = min 30 (rows - 4)

        // Reposition overlays
        let statusOverlay = {
            Id = 1UL
            X = 0us
            Y = 0us
            Text = sprintf "Size: %dx%d" cols rows
            Style = OverlayStyle.Info
        }

        ctx.QueueCommand (RemoteCommand.DrawOverlay statusOverlay)

        return ()
    }
```

## Part 4: Animations and Transitions

RemoteUI doesn't have built-in animations, but you can create smooth transitions:

```fsharp
// Fade-in effect (simulated)
let fadeIn (ctx: PluginContext) (text: string) =
    async {
        let chars = text.ToCharArray()

        for i in 0 .. chars.Length - 1 do
            let partial = new String(chars, 0, i + 1)

            let overlay = {
                Id = 1UL
                X = 10us
                Y = 5us
                Text = partial
                Style = OverlayStyle.Info
            }

            ctx.QueueCommand (RemoteCommand.DrawOverlay overlay)

            // Small delay
            do! Async.Sleep 50

        return ()
    }

// Slide-in effect
let slideIn (ctx: PluginContext) (targetX: uint16) =
    async {
        let startX = 0us

        for x in startX .. targetX do
            let overlay = {
                Id = 1UL
                X = x
                Y = 5us
                Text = "Sliding..."
                Style = OverlayStyle.Info
            }

            ctx.QueueCommand (RemoteCommand.DrawOverlay overlay)
            do! Async.Sleep 20

        return ()
    }
```

## Part 5: Complete Command Palette

Here's the complete, production-ready command palette plugin:

```fsharp
module command_palette

open Scarab.PluginApi
open Scarab.RemoteUI
open System

[<Plugin>]
let metadata = {
    Name = "command-palette"
    Version = "1.0.0"
    Description = "Quick command access with fuzzy search"
    Author = "Your Name"
    Emoji = Some "⌘"
    Color = Some "#6366F1"
    Catchphrase = Some "Command at your fingertips!"
}

// Command definition
type Command = {
    Id: string
    Name: string
    Description: string
    Category: string
    Icon: string
    Action: PluginContext -> Async<unit>
}

// State
let mutable isOpen = false
let mutable selectedIndex = 0
let mutable searchQuery = ""

// Commands registry
let commands = [
    {
        Id = "new-tab"
        Name = "New Tab"
        Description = "Open a new terminal tab"
        Category = "Tab"
        Icon = "➕"
        Action = fun ctx ->
            async {
                ctx.NotifySuccess "New Tab" "Opening new tab..."
                // Implementation depends on Scarab's tab API
            }
    }
    {
        Id = "split-vertical"
        Name = "Split Vertical"
        Description = "Split the terminal vertically"
        Category = "Window"
        Icon = "┃"
        Action = fun ctx ->
            async {
                ctx.NotifySuccess "Split Vertical" "Splitting..."
            }
    }
    {
        Id = "split-horizontal"
        Name = "Split Horizontal"
        Description = "Split the terminal horizontally"
        Category = "Window"
        Icon = "━"
        Action = fun ctx ->
            async {
                ctx.NotifySuccess "Split Horizontal" "Splitting..."
            }
    }
    {
        Id = "close-window"
        Name = "Close Window"
        Description = "Close the current window"
        Category = "Window"
        Icon = "✕"
        Action = fun ctx ->
            async {
                ctx.NotifyInfo "Close Window" "Closing current window..."
            }
    }
    {
        Id = "settings"
        Name = "Open Settings"
        Description = "Open Scarab settings"
        Category = "System"
        Icon = "⚙"
        Action = fun ctx ->
            async {
                ctx.NotifyInfo "Settings" "Opening settings..."
            }
    }
    {
        Id = "plugins"
        Name = "Manage Plugins"
        Description = "Enable/disable plugins"
        Category = "System"
        Icon = "🔌"
        Action = fun ctx ->
            async {
                ctx.NotifyInfo "Plugins" "Opening plugin manager..."
            }
    }
    {
        Id = "reload"
        Name = "Reload Configuration"
        Description = "Reload Scarab configuration"
        Category = "System"
        Icon = "🔄"
        Action = fun ctx ->
            async {
                ctx.NotifySuccess "Reload" "Configuration reloaded"
            }
    }
]

// Fuzzy search scoring
let fuzzyScore (query: string) (text: string) : int =
    let query = query.ToLower()
    let text = text.ToLower()
    let mutable score = 0
    let mutable lastIndex = -1

    for c in query do
        let index = text.IndexOf(c, lastIndex + 1)
        if index >= 0 then
            score <- score + 1
            if index = lastIndex + 1 then
                score <- score + 1  // Bonus for consecutive matches
            lastIndex <- index

    score

// Filter and sort commands
let filterCommands (query: string) =
    if String.IsNullOrWhiteSpace(query) then
        commands
    else
        commands
        |> List.map (fun cmd ->
            let nameScore = fuzzyScore query cmd.Name
            let descScore = fuzzyScore query cmd.Description
            (cmd, max nameScore descScore)
        )
        |> List.filter (fun (_, score) -> score > 0)
        |> List.sortByDescending snd
        |> List.map fst

// Build the UI
let buildPaletteUI (ctx: PluginContext) =
    let filteredCommands = filterCommands searchQuery

    // Terminal dimensions for centering
    let (cols, rows) = ctx.GetSize()

    // Modal dimensions
    let modalWidth = min 70 (cols - 10)
    let modalHeight = min 25 (rows - 6)

    // Create modal
    let modal = {
        Id = "command-palette-modal"
        Title = sprintf "Command Palette (%d commands)" filteredCommands.Length
        Width = int modalWidth
        Height = int modalHeight
        X = None  // Centered
        Y = None  // Centered
        Style = ModalStyle.Default
        FocusOnOpen = true
    }

    // Search input
    let searchInput = {
        Id = "search-input"
        Placeholder = "Type to search commands..."
        Value = searchQuery
        Width = int modalWidth - 4
        X = 2
        Y = 2
        Style = if String.IsNullOrEmpty(searchQuery) then InputStyle.Default else InputStyle.Focused
        OnChange = fun newValue ->
            searchQuery <- newValue
            selectedIndex <- 0
            buildPaletteUI ctx
    }

    // Command list
    let listItems =
        filteredCommands
        |> List.mapi (fun i cmd ->
            {
                Id = cmd.Id
                Label = sprintf "%s  %s" cmd.Icon cmd.Name
                Description = Some (sprintf "[%s] %s" cmd.Category cmd.Description)
                IsSelected = (i = selectedIndex)
                OnSelect = fun () ->
                    async {
                        isOpen <- false
                        ctx.QueueCommand (RemoteCommand.CloseModal { Id = modal.Id })
                        do! cmd.Action ctx
                    }
            }
        )

    let commandList = {
        Id = "command-list"
        Items = listItems
        Width = int modalWidth - 4
        Height = int modalHeight - 7
        X = 2
        Y = 5
        Style = ListStyle.Default
        SelectedIndex = selectedIndex
    }

    // Hint text
    let hintOverlay = {
        Id = 2UL
        X = 2us
        Y = uint16 (modalHeight - 2)
        Text = "↑↓: Navigate | Enter: Execute | Esc: Close"
        Style = OverlayStyle.Info
    }

    // Queue all commands
    ctx.QueueCommand (RemoteCommand.ShowModal modal)
    ctx.QueueCommand (RemoteCommand.DrawInput searchInput)
    ctx.QueueCommand (RemoteCommand.DrawList commandList)
    ctx.QueueCommand (RemoteCommand.DrawOverlay hintOverlay)

// Open palette
let openPalette (ctx: PluginContext) =
    isOpen <- true
    searchQuery <- ""
    selectedIndex <- 0
    buildPaletteUI ctx

// Close palette
let closePalette (ctx: PluginContext) =
    isOpen <- false
    ctx.QueueCommand (RemoteCommand.CloseModal { Id = "command-palette-modal" })
    ctx.QueueCommand (RemoteCommand.ClearOverlays { Id = Some 2UL })

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Command Palette loaded"
        ctx.NotifySuccess "Command Palette Ready" "Press Ctrl+P to open"
        return Ok ()
    }

[<OnKeyPress>]
let onKeyPress (ctx: PluginContext) (key: KeyEvent) =
    async {
        // Toggle palette with Ctrl+P
        if key.Code = KeyCode.Char 'p' && key.Modifiers.Contains(KeyModifier.Control) then
            if isOpen then
                closePalette ctx
            else
                openPalette ctx
            return Stop

        // Handle navigation when open
        elif isOpen then
            let filteredCommands = filterCommands searchQuery

            match key.Code with
            | KeyCode.Escape ->
                closePalette ctx
                return Stop

            | KeyCode.Up ->
                selectedIndex <- max 0 (selectedIndex - 1)
                buildPaletteUI ctx
                return Stop

            | KeyCode.Down ->
                selectedIndex <- min (filteredCommands.Length - 1) (selectedIndex + 1)
                buildPaletteUI ctx
                return Stop

            | KeyCode.Enter ->
                if selectedIndex < filteredCommands.Length then
                    let cmd = filteredCommands.[selectedIndex]
                    closePalette ctx
                    do! cmd.Action ctx
                return Stop

            | _ ->
                return Continue
        else
            return Continue
    }

[<OnUnload>]
let onUnload (ctx: PluginContext) =
    async {
        if isOpen then
            closePalette ctx
        ctx.Log Info "Command Palette unloaded"
        return Ok ()
    }
```

## What You Learned

- ✅ RemoteUI architecture and component model
- ✅ Building modals, overlays, and input fields
- ✅ Handling keyboard and mouse events
- ✅ Creating responsive, adaptive layouts
- ✅ Implementing fuzzy search
- ✅ State management in frontend plugins
- ✅ Professional UI/UX patterns

## Performance Considerations

### Do's

- ✅ **Batch UI updates** - Queue multiple commands at once
- ✅ **Debounce input** - Don't rebuild UI on every keystroke
- ✅ **Cache computed values** - Store filtered results
- ✅ **Use OnResize** - Adapt to terminal size changes
- ✅ **Clear old UI** - Remove unused overlays/modals

### Don'ts

- ❌ **Don't update every frame** - Only on state changes
- ❌ **Don't create deep nesting** - Keep UI flat
- ❌ **Don't block rendering** - Keep hooks fast (< 16ms)
- ❌ **Don't leak modals** - Always close on unload
- ❌ **Don't ignore terminal size** - Always check bounds

## Troubleshooting

### UI not appearing?

1. **Check frontend runtime** - RemoteUI only works in frontend plugins
2. **Verify component IDs** - Must be unique
3. **Check terminal size** - UI might be off-screen
4. **Enable debug logging** - See what commands are queued

### Input not working?

1. **Check OnKeyPress return value** - Return `Stop` to consume events
2. **Verify modal has focus** - Set `FocusOnOpen = true`
3. **Check event handlers** - Ensure callbacks are defined
4. **Test with simple input** - Start with basic key detection

### Layout issues?

1. **Check coordinates** - Must be within terminal bounds
2. **Use OnResize** - Recalculate on terminal size change
3. **Test different sizes** - Terminals vary widely
4. **Add bounds checking** - Validate X/Y before drawing

## Best Practices

### UX Patterns

1. **Keyboard-first** - All actions should be keyboard-accessible
2. **Clear feedback** - Show loading states, confirmations
3. **Consistent styling** - Use the same colors/fonts throughout
4. **Esc to close** - Always allow Escape to dismiss modals
5. **Help text** - Show keyboard hints

### Accessibility

1. **High contrast** - Ensure text is readable
2. **Clear focus indicators** - Show what's selected
3. **Keyboard navigation** - No mouse-only actions
4. **Screen reader support** - Use descriptive labels
5. **Consistent behavior** - Follow terminal conventions

### Code Organization

1. **Separate concerns** - UI logic vs business logic
2. **Reusable components** - Extract common patterns
3. **State management** - Centralize state mutations
4. **Error boundaries** - Handle UI errors gracefully
5. **Testing** - Unit test UI logic separately

## Next Steps

→ **[Backend Processing with Hooks](../architecture/performance.md)** - Optimize backend plugins

→ **[Plugin Testing Guide](../testing/plugin-testing.md)** - Test your plugins thoroughly

→ **[Deployment Guide](../deployment/publishing.md)** - Share your plugins

## Additional Resources

- **UI Frameworks for inspiration:**
  - [lazygit](https://github.com/jesseduffield/lazygit) - Git TUI
  - [bottom](https://github.com/ClementTsang/bottom) - System monitor
  - [zellij](https://github.com/zellij-org/zellij) - Terminal multiplexer

- **Design Resources:**
  - [Nord Theme](https://www.nordtheme.com/) - Popular color scheme
  - [Nerd Fonts](https://www.nerdfonts.com/) - Icon fonts for terminals
  - [Terminal Colors](https://terminal.sexy/) - Color scheme designer

Happy coding!
