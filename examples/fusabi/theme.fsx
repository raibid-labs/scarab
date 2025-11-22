// Theme configuration example

// Color palette
let colors = {
    background = "#1e1e2e";
    foreground = "#cdd6f4";
    black = "#45475a";
    red = "#f38ba8";
    green = "#a6e3a1";
    yellow = "#f9e2af";
    blue = "#89b4fa";
    magenta = "#f5c2e7";
    cyan = "#94e2d5";
    white = "#bac2de"
}

// UI element styles
let styles = {
    terminal = {
        background = colors.background;
        foreground = colors.foreground;
        cursor = colors.blue;
        selection = "#313244"
    };
    tabs = {
        active_bg = colors.blue;
        active_fg = colors.background;
        inactive_bg = "#313244";
        inactive_fg = colors.foreground
    };
    statusline = {
        normal_bg = colors.blue;
        normal_fg = colors.background;
        insert_bg = colors.green;
        insert_fg = colors.background
    }
}

// Font configuration
let fonts = {
    family = "JetBrains Mono";
    size = 14;
    ligatures = true
}

// Export theme
let theme = {
    name = "Catppuccin Mocha";
    colors = colors;
    styles = styles;
    fonts = fonts
}

println "Theme loaded: Catppuccin Mocha"
println (concat "Background: " colors.background)
println (concat "Font: " (concat fonts.family (concat " " (to_string fonts.size))))
