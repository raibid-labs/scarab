# RemoteUI Components Reference

RemoteUI is Scarab's declarative UI framework for building plugin interfaces. It provides a set of components that can be rendered as overlays, modals, and notifications on top of the terminal.

## Introduction

RemoteUI allows plugins to create rich, interactive interfaces without direct access to rendering code. Components are defined declaratively in F# and sent via IPC to the client for rendering.

**Key features:**
- Declarative component model
- Reactive updates
- Event handling
- Flexible styling
- Terminal-optimized layouts

**Rendering model:**
1. Plugin creates component tree
2. Backend serializes to IPC message
3. Client deserializes and renders
4. User interactions sent back to plugin

## Quick Start

```fsharp
open Scarab.PluginApi
open Scarab.RemoteUI

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Create a simple overlay
        let ui =
            Container(
                padding = 2,
                background = Color.DarkGray,
                children = [
                    Text("Hello from RemoteUI!",
                         color = Color.White,
                         bold = true)
                    Button("Click me!",
                           onClick = fun () -> ctx.Log Info "Button clicked!")
                ]
            )

        ctx.ShowOverlay "welcome" ui (x = 10, y = 1)
        return Ok ()
    }
```

## Base Components

### Container

Layout container with flexible box model.

**Type Signature:**
```fsharp
type Container = {
    Id: string option
    Children: Component list
    Layout: Layout
    Padding: Spacing
    Margin: Spacing
    Background: Color option
    Border: Border option
    Width: Size option
    Height: Size option
    Position: Position
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `id` | `string option` | `None` | Unique identifier |
| `children` | `Component list` | `[]` | Child components |
| `layout` | `Layout` | `Vertical` | Layout direction |
| `padding` | `Spacing` | `0` | Internal spacing |
| `margin` | `Spacing` | `0` | External spacing |
| `background` | `Color option` | `None` | Background color |
| `border` | `Border option` | `None` | Border style |
| `width` | `Size option` | `Auto` | Width constraint |
| `height` | `Size option` | `Auto` | Height constraint |
| `position` | `Position` | `Relative` | Positioning mode |

**Layout Types:**

```fsharp
type Layout =
    | Vertical       // Stack children vertically
    | Horizontal     // Stack children horizontally
    | Grid of cols: int * rows: int  // Grid layout
    | Flex of justify: Justify * align: Align  // Flexbox layout
```

**Example: Vertical Stack**

```fsharp
Container(
    layout = Vertical,
    padding = 1,
    children = [
        Text("Line 1")
        Text("Line 2")
        Text("Line 3")
    ]
)
```

**Example: Horizontal Layout**

```fsharp
Container(
    layout = Horizontal,
    padding = 1,
    children = [
        Text("Left", width = Fixed 20)
        Text("Center", width = Flex 1)
        Text("Right", width = Fixed 20)
    ]
)
```

**Example: Grid Layout**

```fsharp
Container(
    layout = Grid(cols = 2, rows = 2),
    padding = 1,
    children = [
        Text("Cell 1,1")
        Text("Cell 1,2")
        Text("Cell 2,1")
        Text("Cell 2,2")
    ]
)
```

**Example: Flexbox**

```fsharp
Container(
    layout = Flex(justify = SpaceBetween, align = Center),
    width = Fixed 80,
    children = [
        Text("Start")
        Text("Middle")
        Text("End")
    ]
)
```

### Text

Display styled text.

**Type Signature:**
```fsharp
type Text = {
    Content: string
    Color: Color option
    Background: Color option
    Bold: bool
    Italic: bool
    Underline: bool
    FontSize: FontSize
    FontFamily: FontFamily option
    Wrap: TextWrap
    Align: TextAlign
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `content` | `string` | Required | Text to display |
| `color` | `Color option` | `None` | Text color |
| `background` | `Color option` | `None` | Background color |
| `bold` | `bool` | `false` | Bold text |
| `italic` | `bool` | `false` | Italic text |
| `underline` | `bool` | `false` | Underline text |
| `fontSize` | `FontSize` | `Normal` | Text size |
| `fontFamily` | `FontFamily option` | `None` | Font family |
| `wrap` | `TextWrap` | `Wrap` | Text wrapping |
| `align` | `TextAlign` | `Left` | Text alignment |

**Example: Styled Text**

```fsharp
Text(
    content = "Important message",
    color = Color.Red,
    bold = true,
    underline = true
)
```

**Example: Large Header**

```fsharp
Text(
    content = "Command Palette",
    fontSize = Large,
    bold = true,
    align = Center
)
```

**Example: Multiline with Wrapping**

```fsharp
Text(
    content = "This is a long message that will automatically wrap to fit the container width.",
    wrap = Wrap,
    width = Fixed 40
)
```

### Button

Interactive button with click handler.

**Type Signature:**
```fsharp
type Button = {
    Label: string
    OnClick: unit -> unit
    Style: ButtonStyle
    Disabled: bool
    Width: Size option
    Shortcut: KeyBinding option
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `label` | `string` | Required | Button text |
| `onClick` | `unit -> unit` | Required | Click handler |
| `style` | `ButtonStyle` | `Default` | Visual style |
| `disabled` | `bool` | `false` | Disabled state |
| `width` | `Size option` | `Auto` | Button width |
| `shortcut` | `KeyBinding option` | `None` | Keyboard shortcut |

**Button Styles:**

```fsharp
type ButtonStyle =
    | Default     // Gray background
    | Primary     // Blue, prominent
    | Success     // Green, positive action
    | Warning     // Orange, caution
    | Danger      // Red, destructive action
    | Link        // No background, underlined
```

**Example: Basic Button**

```fsharp
Button(
    label = "OK",
    onClick = fun () -> ctx.Log Info "OK clicked"
)
```

**Example: Styled Buttons**

```fsharp
Container(
    layout = Horizontal,
    children = [
        Button("Save", onClick = save, style = Success)
        Button("Cancel", onClick = cancel, style = Default)
        Button("Delete", onClick = delete, style = Danger)
    ]
)
```

**Example: Button with Shortcut**

```fsharp
Button(
    label = "Search",
    onClick = openSearch,
    style = Primary,
    shortcut = Some (KeyBinding.Ctrl 'p')
)
```

### Input

Text input field.

**Type Signature:**
```fsharp
type Input = {
    Value: string
    Placeholder: string option
    OnChange: string -> unit
    OnSubmit: string -> unit option
    InputType: InputType
    Disabled: bool
    Width: Size option
    MaxLength: int option
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `value` | `string` | Required | Current value |
| `placeholder` | `string option` | `None` | Placeholder text |
| `onChange` | `string -> unit` | Required | Change handler |
| `onSubmit` | `string -> unit option` | `None` | Submit handler (Enter key) |
| `inputType` | `InputType` | `Text` | Input type |
| `disabled` | `bool` | `false` | Disabled state |
| `width` | `Size option` | `Flex 1` | Input width |
| `maxLength` | `int option` | `None` | Max character count |

**Input Types:**

```fsharp
type InputType =
    | Text          // Single-line text
    | Password      // Masked input
    | Number        // Numeric input
    | Search        // Search with clear button
```

**Example: Basic Input**

```fsharp
let mutable searchQuery = ""

Input(
    value = searchQuery,
    placeholder = Some "Search...",
    onChange = fun v -> searchQuery <- v
)
```

**Example: Search with Submit**

```fsharp
Input(
    value = query,
    placeholder = Some "Enter command...",
    inputType = Search,
    onChange = updateQuery,
    onSubmit = Some executeSearch
)
```

**Example: Password Input**

```fsharp
Input(
    value = password,
    placeholder = Some "Enter password",
    inputType = Password,
    onChange = fun v -> password <- v,
    maxLength = Some 128
)
```

### List

Scrollable list of items.

**Type Signature:**
```fsharp
type List<'T> = {
    Items: 'T list
    Render: 'T -> Component
    OnSelect: 'T -> unit option
    Selected: int option
    Height: Size option
    EmptyMessage: string option
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `items` | `'T list` | Required | List items |
| `render` | `'T -> Component` | Required | Item renderer |
| `onSelect` | `'T -> unit option` | `None` | Selection handler |
| `selected` | `int option` | `None` | Selected index |
| `height` | `Size option` | `Auto` | List height |
| `emptyMessage` | `string option` | `None` | Message when empty |

**Example: Simple List**

```fsharp
List(
    items = ["Option 1"; "Option 2"; "Option 3"],
    render = fun item -> Text(item),
    onSelect = Some (fun item -> ctx.Log Info (sprintf "Selected: %s" item))
)
```

**Example: Rich List Items**

```fsharp
type Command = { Name: string; Description: string; Shortcut: string }

let commands = [
    { Name = "Search"; Description = "Open search"; Shortcut = "Ctrl+P" }
    { Name = "Theme"; Description = "Change theme"; Shortcut = "Ctrl+T" }
]

List(
    items = commands,
    render = fun cmd ->
        Container(
            layout = Horizontal,
            children = [
                Text(cmd.Name, bold = true, width = Fixed 20)
                Text(cmd.Description, width = Flex 1)
                Text(cmd.Shortcut, color = Color.Gray)
            ]
        ),
    onSelect = Some executeCommand,
    height = Fixed 10
)
```

**Example: Searchable List**

```fsharp
let mutable query = ""
let filtered =
    items
    |> List.filter (fun i -> i.ToLower().Contains(query.ToLower()))

Container(
    layout = Vertical,
    children = [
        Input(
            value = query,
            placeholder = Some "Filter...",
            onChange = fun v -> query <- v
        )
        List(
            items = filtered,
            render = fun item -> Text(item),
            emptyMessage = Some "No matches found"
        )
    ]
)
```

## Overlay Components

### Overlay

Floating overlay positioned at terminal coordinates.

**Type Signature:**
```fsharp
type Overlay = {
    Id: string
    X: int
    Y: int
    Content: Component
    Style: OverlayStyle
    AutoClose: AutoClose option
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `id` | `string` | Required | Unique identifier |
| `x` | `int` | Required | Column position (0-based) |
| `y` | `int` | Required | Row position (0-based) |
| `content` | `Component` | Required | Overlay content |
| `style` | `OverlayStyle` | `Default` | Visual style |
| `autoClose` | `AutoClose option` | `None` | Auto-close behavior |

**Example: Status Overlay**

```fsharp
ctx.ShowOverlay "git-status" (
    Overlay(
        id = "git-status",
        x = 10,
        y = 1,
        content = Container(
            padding = 1,
            background = Color.DarkGray,
            border = Some (Border.Single Color.Blue),
            children = [
                Text("⎇ main", color = Color.Green)
                Text("↑1 ↓0", color = Color.Yellow)
            ]
        )
    )
)
```

**Example: Auto-Closing Overlay**

```fsharp
ctx.ShowOverlay "notification" (
    Overlay(
        id = "notification",
        x = 80,
        y = 1,
        content = Text("Build complete!", color = Color.Green, bold = true),
        autoClose = Some (AutoClose.After (TimeSpan.FromSeconds 3.0))
    )
)
```

**Auto-Close Options:**

```fsharp
type AutoClose =
    | After of TimeSpan              // Close after duration
    | OnClick                        // Close on any click
    | OnEscape                       // Close on Escape key
    | OnClickOutside                 // Close when clicking outside
    | Multiple of AutoClose list     // Multiple conditions
```

### Modal

Centered modal dialog.

**Type Signature:**
```fsharp
type Modal = {
    Id: string
    Title: string option
    Content: Component
    Width: Size
    Height: Size option
    OnClose: (unit -> unit) option
    CloseOnEscape: bool
    CloseOnClickOutside: bool
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `id` | `string` | Required | Unique identifier |
| `title` | `string option` | `None` | Modal title |
| `content` | `Component` | Required | Modal content |
| `width` | `Size` | `Fixed 60` | Modal width |
| `height` | `Size option` | `Auto` | Modal height |
| `onClose` | `unit -> unit option` | `None` | Close handler |
| `closeOnEscape` | `bool` | `true` | Close with Escape |
| `closeOnClickOutside` | `bool` | `true` | Close when clicking outside |

**Example: Simple Modal**

```fsharp
ctx.ShowModal (
    Modal(
        id = "about",
        title = Some "About Plugin",
        content = Container(
            layout = Vertical,
            padding = 2,
            children = [
                Text("My Plugin v1.0.0")
                Text("Author: Your Name")
                Button("Close", onClick = fun () -> ctx.CloseModal "about")
            ]
        )
    )
)
```

**Example: Command Palette**

```fsharp
let mutable query = ""
let filtered = filterCommands commands query

ctx.ShowModal (
    Modal(
        id = "command-palette",
        title = Some "Command Palette",
        width = Fixed 80,
        height = Fixed 20,
        content = Container(
            layout = Vertical,
            children = [
                Input(
                    value = query,
                    placeholder = Some "Search commands...",
                    onChange = fun v -> query <- v,
                    inputType = Search
                )
                List(
                    items = filtered,
                    render = renderCommand,
                    onSelect = Some (fun cmd ->
                        ctx.CloseModal "command-palette"
                        executeCommand cmd
                    )
                )
            ]
        ),
        closeOnEscape = true
    )
)
```

**Example: Confirmation Dialog**

```fsharp
ctx.ShowModal (
    Modal(
        id = "confirm",
        title = Some "Confirm Action",
        width = Fixed 50,
        content = Container(
            layout = Vertical,
            padding = 2,
            children = [
                Text("Are you sure you want to delete this?")
                Container(
                    layout = Horizontal,
                    margin = { Top = 2; Right = 0; Bottom = 0; Left = 0 },
                    children = [
                        Button("Cancel",
                               onClick = fun () -> ctx.CloseModal "confirm",
                               style = Default)
                        Button("Delete",
                               onClick = fun () ->
                                   performDelete()
                                   ctx.CloseModal "confirm",
                               style = Danger)
                    ]
                )
            ]
        ),
        closeOnEscape = true,
        closeOnClickOutside = false  // Require explicit choice
    )
)
```

### Notification

Toast notification that appears briefly.

**Type Signature:**
```fsharp
type Notification = {
    Title: string
    Body: string option
    Level: NotifyLevel
    Duration: TimeSpan option
    Position: NotificationPosition
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `title` | `string` | Required | Notification title |
| `body` | `string option` | `None` | Notification body |
| `level` | `NotifyLevel` | `Info` | Severity level |
| `duration` | `TimeSpan option` | `Some 3s` | Display duration |
| `position` | `NotificationPosition` | `TopRight` | Screen position |

**Notification Levels:**

```fsharp
type NotifyLevel =
    | Success    // Green with checkmark
    | Info       // Blue with info icon
    | Warning    // Orange with warning icon
    | Error      // Red with error icon
```

**Notification Positions:**

```fsharp
type NotificationPosition =
    | TopLeft
    | TopCenter
    | TopRight      // Default
    | BottomLeft
    | BottomCenter
    | BottomRight
```

**Example: Success Notification**

```fsharp
ctx.Notify (
    Notification(
        title = "Build Complete",
        body = Some "All tests passed",
        level = Success,
        duration = Some (TimeSpan.FromSeconds 5.0)
    )
)

// Or use convenience method
ctx.NotifySuccess "Build Complete" "All tests passed"
```

**Example: Error Notification**

```fsharp
ctx.NotifyError "Compilation Failed" "3 errors, 5 warnings"
```

**Example: Persistent Notification**

```fsharp
ctx.Notify (
    Notification(
        title = "Action Required",
        body = Some "Please review the changes",
        level = Warning,
        duration = None,  // Stays until dismissed
        position = BottomCenter
    )
)
```

### Tooltip

Contextual tooltip attached to another component.

**Type Signature:**
```fsharp
type Tooltip = {
    Content: string
    Target: Component
    Position: TooltipPosition
    Delay: TimeSpan option
}
```

**Properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `content` | `string` | Required | Tooltip text |
| `target` | `Component` | Required | Component to attach to |
| `position` | `TooltipPosition` | `Auto` | Tooltip position |
| `delay` | `TimeSpan option` | `Some 500ms` | Show delay |

**Example: Button with Tooltip**

```fsharp
Tooltip(
    content = "Opens the command palette (Ctrl+P)",
    target = Button(
        label = "Commands",
        onClick = openCommandPalette
    )
)
```

**Example: Icon with Tooltip**

```fsharp
Tooltip(
    content = "Build successful",
    target = Text("✓", color = Color.Green, bold = true),
    position = TooltipPosition.Above
)
```

## Styling

### Colors

Built-in color constants and RGB support:

```fsharp
type Color =
    // Basic colors
    | Black | White | Gray | DarkGray | LightGray
    // Primary colors
    | Red | Green | Blue | Yellow | Cyan | Magenta
    // Extended colors
    | Orange | Purple | Pink | Teal | Lime | Indigo
    // Custom
    | RGB of r: byte * g: byte * b: byte
    | Hex of string
```

**Example: Using Colors**

```fsharp
Text("Error", color = Color.Red, background = Color.DarkGray)
Text("Success", color = Color.RGB(0, 255, 0))
Text("Link", color = Color.Hex "#0066CC")
```

### Spacing

Control padding and margins:

```fsharp
type Spacing = {
    Top: int
    Right: int
    Bottom: int
    Left: int
}

// Convenience constructors
Spacing.All(n)              // All sides equal
Spacing.Uniform(v, h)       // Vertical, horizontal
Spacing.Custom(t, r, b, l)  // Individual sides
```

**Example: Spacing Usage**

```fsharp
Container(
    padding = Spacing.All(2),
    margin = Spacing.Uniform(1, 0),
    children = [...]
)
```

### Borders

Add borders to containers:

```fsharp
type Border = {
    Style: BorderStyle
    Color: Color option
    Width: int
}

type BorderStyle =
    | Single     // ┌─┐
    | Double     // ╔═╗
    | Rounded    // ╭─╮
    | Bold       // ┏━┓
    | None
```

**Example: Bordered Container**

```fsharp
Container(
    border = Some {
        Style = BorderStyle.Single
        Color = Some Color.Blue
        Width = 1
    },
    children = [...]
)
```

### Sizing

Control component dimensions:

```fsharp
type Size =
    | Auto          // Size to content
    | Fixed of int  // Fixed size in columns/rows
    | Flex of int   // Flexible proportion
    | Percent of int // Percentage of parent
```

**Example: Size Usage**

```fsharp
Container(
    layout = Horizontal,
    children = [
        Text("Sidebar", width = Fixed 20)
        Text("Content", width = Flex 1)     // Takes remaining space
        Text("Right", width = Fixed 15)
    ]
)
```

### Positioning

Control component positioning:

```fsharp
type Position =
    | Relative              // Normal flow
    | Absolute of x: int * y: int  // Absolute position
    | Fixed of x: int * y: int     // Fixed, doesn't scroll
```

**Example: Absolute Positioning**

```fsharp
Container(
    position = Position.Absolute(10, 5),
    children = [
        Text("Positioned at 10, 5")
    ]
)
```

## Event Handling

### OnClick

Handle click events:

```fsharp
Button("Click me", onClick = fun () ->
    ctx.Log Info "Button clicked"
    ctx.NotifySuccess "Clicked" "Button was clicked!"
)
```

### OnKeyPress

Handle keyboard input:

```fsharp
// In frontend plugin
[<OnKeyPress>]
let onKeyPress ctx key =
    async {
        if key.Ctrl && key.Code = KeyCode.P then
            showCommandPalette()
            return Stop  // Prevent default
        return Continue
    }
```

### OnFocus / OnBlur

Handle focus events:

```fsharp
Input(
    value = text,
    onChange = updateText,
    onFocus = Some (fun () -> ctx.Log Debug "Input focused"),
    onBlur = Some (fun () -> validateInput text)
)
```

### OnChange

Handle value changes:

```fsharp
Input(
    value = query,
    onChange = fun newValue ->
        query <- newValue
        updateSearchResults newValue
)
```

## Layout Patterns

### Centering Content

```fsharp
Container(
    layout = Flex(justify = Center, align = Center),
    width = Percent 100,
    height = Percent 100,
    children = [
        Text("Centered!", fontSize = Large)
    ]
)
```

### Responsive Layout

```fsharp
let (cols, rows) = ctx.GetSize()
let width = if cols > 100 then Fixed 80 else Percent 90

Container(
    width = width,
    children = [...]
)
```

### Scrolling Lists

```fsharp
List(
    items = items,
    render = renderItem,
    height = Fixed 10,  // Fixed height enables scrolling
    onSelect = Some handleSelect
)
```

### Split View

```fsharp
Container(
    layout = Horizontal,
    width = Percent 100,
    height = Percent 100,
    children = [
        Container(
            width = Flex 1,
            children = [Text("Left pane")]
        )
        Container(
            width = Fixed 1,
            background = Color.Gray,
            children = []  // Divider
        )
        Container(
            width = Flex 1,
            children = [Text("Right pane")]
        )
    ]
)
```

## Animation

Animations are not yet fully supported but planned for future versions.

**Planned features:**
- Fade in/out
- Slide transitions
- Expand/collapse
- Smooth scrolling

**Current workaround:**

```fsharp
// Manual animation with updates
let rec animate opacity =
    async {
        if opacity > 0.0 then
            updateOverlay opacity
            do! Async.Sleep 16  // ~60fps
            return! animate (opacity - 0.05)
    }

animate 1.0 |> Async.Start
```

## Performance Considerations

### Component Reuse

Reuse component definitions:

```fsharp
// Bad - creates new function on every render
List(items, render = fun i -> Text(i))

// Good - reuse function
let renderItem i = Text(i)
List(items, render = renderItem)
```

### Minimize Updates

Only update when necessary:

```fsharp
let mutable lastQuery = ""

[<OnKeyPress>]
let onKeyPress ctx key =
    if query <> lastQuery then
        lastQuery <- query
        updateUI()  // Only update if changed
```

### Batch Operations

Group multiple UI updates:

```fsharp
// Bad - multiple IPC roundtrips
ctx.ShowOverlay "status1" ui1
ctx.ShowOverlay "status2" ui2
ctx.ShowOverlay "status3" ui3

// Good - single batched operation
ctx.ShowOverlays [
    ("status1", ui1)
    ("status2", ui2)
    ("status3", ui3)
]
```

### Virtual Scrolling

For large lists, use virtual scrolling (automatic when height is fixed):

```fsharp
List(
    items = largeList,  // Can be 1000+ items
    render = renderItem,
    height = Fixed 10   // Only renders visible items
)
```

## Complete Examples

### Command Palette

```fsharp
type Command = {
    Name: string
    Description: string
    Action: unit -> unit
}

let mutable query = ""
let commands = [/* ... */]

let renderCommand cmd =
    Container(
        layout = Horizontal,
        padding = Spacing.All(1),
        children = [
            Text(cmd.Name, bold = true, width = Fixed 25)
            Text(cmd.Description, color = Color.Gray, width = Flex 1)
        ]
    )

let filtered =
    commands
    |> List.filter (fun c ->
        c.Name.ToLower().Contains(query.ToLower()))

let modal = Modal(
    id = "palette",
    title = Some "Command Palette",
    width = Fixed 80,
    height = Fixed 20,
    content = Container(
        layout = Vertical,
        children = [
            Input(
                value = query,
                placeholder = Some "Search commands...",
                inputType = Search,
                onChange = fun v -> query <- v
            )
            List(
                items = filtered,
                render = renderCommand,
                onSelect = Some (fun cmd ->
                    ctx.CloseModal "palette"
                    cmd.Action()
                ),
                height = Fixed 15
            )
        ]
    )
)

ctx.ShowModal modal
```

### Status Bar

```fsharp
let statusBar gitInfo =
    Overlay(
        id = "statusbar",
        x = 0,
        y = 0,
        content = Container(
            layout = Horizontal,
            width = Percent 100,
            padding = Spacing.Uniform(0, 1),
            background = Color.DarkGray,
            children = [
                Text("⎇ " + gitInfo.Branch,
                     color = Color.Green,
                     width = Fixed 20)
                Text("↑" + string gitInfo.Ahead,
                     color = Color.Yellow,
                     width = Fixed 5)
                Text("↓" + string gitInfo.Behind,
                     color = Color.Yellow,
                     width = Fixed 5)
                Text("", width = Flex 1)  // Spacer
                Text(DateTime.Now.ToString("HH:mm"),
                     color = Color.White,
                     width = Fixed 10)
            ]
        )
    )
```

### Quick Notes

```fsharp
let mutable notes = []
let mutable noteText = ""

let notesUI =
    Modal(
        id = "notes",
        title = Some "Quick Notes",
        width = Fixed 60,
        height = Fixed 30,
        content = Container(
            layout = Vertical,
            children = [
                Container(
                    layout = Horizontal,
                    children = [
                        Input(
                            value = noteText,
                            placeholder = Some "New note...",
                            onChange = fun v -> noteText <- v,
                            onSubmit = Some (fun text ->
                                notes <- text :: notes
                                noteText <- ""
                            ),
                            width = Flex 1
                        )
                        Button("Add",
                               onClick = fun () ->
                                   notes <- noteText :: notes
                                   noteText <- "",
                               style = Primary)
                    ]
                )
                List(
                    items = notes,
                    render = fun note ->
                        Container(
                            layout = Horizontal,
                            children = [
                                Text(note, width = Flex 1)
                                Button("×",
                                       onClick = fun () ->
                                           notes <- notes |> List.filter ((<>) note),
                                       style = Danger)
                            ]
                        ),
                    height = Fixed 20,
                    emptyMessage = Some "No notes yet"
                )
            ]
        )
    )
```

## Next Steps

- **[Utilities Reference](utilities.md)** - Helper functions
- **[Plugin Context](plugin-context.md)** - Context API methods
- **[Tutorial 5: Frontend UI](../tutorials/05-frontend-ui-with-remoteui.md)** - Build complex UIs

## Getting Help

- Examples: `plugins/examples/`
- Source: `crates/scarab-remote-ui/`
- Issues: GitHub Issues
- Discussions: GitHub Discussions

---

**Remember:** RemoteUI is optimized for terminal rendering. Keep layouts simple, use appropriate component types, and test at different terminal sizes.
