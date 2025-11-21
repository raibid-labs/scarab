// Example keybindings configuration

// Leader key configuration
let leader = "<Space>"

// Define keybinding groups
let window_bindings = [
    { keys = [leader; "w"; "v"]; command = "split_vertical"; desc = "Split window vertically" };
    { keys = [leader; "w"; "h"]; command = "split_horizontal"; desc = "Split window horizontally" };
    { keys = [leader; "w"; "q"]; command = "close_window"; desc = "Close current window" };
    { keys = [leader; "w"; "o"]; command = "close_others"; desc = "Close other windows" }
]

let tab_bindings = [
    { keys = [leader; "t"; "n"]; command = "new_tab"; desc = "New tab" };
    { keys = [leader; "t"; "q"]; command = "close_tab"; desc = "Close tab" };
    { keys = [leader; "t"; "l"]; command = "next_tab"; desc = "Next tab" };
    { keys = [leader; "t"; "h"]; command = "prev_tab"; desc = "Previous tab" }
]

let search_bindings = [
    { keys = [leader; "/"]; command = "search"; desc = "Search in buffer" };
    { keys = [leader; "?"]; command = "search_reverse"; desc = "Search backward" };
    { keys = [leader; "f"; "f"]; command = "find_file"; desc = "Find file" };
    { keys = [leader; "f"; "r"]; command = "recent_files"; desc = "Recent files" }
]

// Combine all bindings
let all_bindings = append window_bindings (append tab_bindings search_bindings)

// Print keybindings by category
let print_category name bindings =
    println (concat "\n" (concat name ":"))
    println "=================="

print_category "Window Management" window_bindings
print_category "Tab Management" tab_bindings
print_category "Search" search_bindings

println (concat "\nTotal bindings: " (to_string (length all_bindings)))
