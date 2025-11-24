// Minimal Scarab Configuration Example
// This demonstrates the simplest possible Fusabi config
//
// Usage: Copy to ~/.config/scarab/config.fsx
// Note: This is a FUTURE feature - Fusabi config DSL not yet implemented

open Scarab.Config
open Scarab.Themes

// Create a minimal config with just a theme
let config =
    ScarabConfig.create()
        |> withTheme (gruvboxDark())

// Export the configuration
// This makes it available to the Scarab runtime
Scarab.export config
