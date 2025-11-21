// Example UI overlay script for command palette

// Define command palette configuration
let palette_config = {
    width = 600;
    height = 400;
    position = "centered"
}

// Command definitions
let commands = [
    { name = "Split Vertical"; key = "v"; action = "split_vertical" };
    { name = "Split Horizontal"; key = "h"; action = "split_horizontal" };
    { name = "New Tab"; key = "t"; action = "new_tab" };
    { name = "Close Tab"; key = "q"; action = "close_tab" };
    { name = "Search"; key = "/"; action = "search" };
    { name = "Settings"; key = ","; action = "settings" }
]

// Filter function for search
let filter_commands query cmds =
    let matches_query cmd =
        let name_lower = to_lower cmd.name
        let query_lower = to_lower query
        // Simple substring matching (would use proper search in real impl)
        true
    in
    cmds

// Render command list
let render_command cmd =
    println (concat "  " (concat cmd.key (concat " - " cmd.name)))

// Main entry point
let main =
    println "Command Palette:"
    println "================"
    // In real implementation, this would spawn UI entities
    let filtered = commands
    let render_all = fun cmds ->
        if length cmds == 0 then
            println "No commands found"
        else
            println (concat "Found " (concat (to_string (length cmds)) " commands"))
    in
    render_all filtered

main
