// Standard Scarab Configuration Example
// This shows common configuration options
//
// Usage: Copy to ~/.config/scarab/config.fsx
// Note: This is a FUTURE feature - Fusabi config DSL not yet implemented

open Scarab.Config
open Scarab.Themes
open Scarab.Plugins

let config =
    ScarabConfig.create()
        // Terminal settings
        |> withTerminal {
            shell = "/bin/zsh"              // Shell to run (default: $SHELL)
            columns = 120u16                // Terminal width in characters
            rows = 40u16                    // Terminal height in characters
            scrollback = 10000u             // Number of lines to keep in scrollback
            altScreen = true                // Enable alternate screen buffer
            scrollMultiplier = 3.0f         // Mouse wheel scroll speed
            autoScroll = true               // Auto-scroll on new output
        }

        // Font configuration
        |> withFont {
            family = "JetBrains Mono"       // Primary font family
            size = 14.0f                    // Font size in points
            lineHeight = 1.2f               // Line height multiplier
            fallback = [                    // Fallback fonts
                "Fira Code"
                "Hack"
                "monospace"
            ]
        }

        // Color scheme
        |> withTheme (gruvboxDark())

        // Built-in plugins
        |> withPlugins [
            gitPrompt()                     // Git branch in prompt
            urlHighlighter()                // Highlight and click URLs
        ]

// Export configuration
Scarab.export config
