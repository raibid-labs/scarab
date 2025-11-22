// Advanced UI/UX module for Scarab terminal emulator
// Provides power-user features: link hints, command palette, leader keys, etc.

pub mod link_hints;
pub mod command_palette;
pub mod leader_key;
pub mod keybindings;
pub mod animations;
pub mod visual_selection;

pub use link_hints::{LinkHintsPlugin, LinkHint, LinkDetector};
pub use command_palette::{CommandPalettePlugin, Command, CommandRegistry};
pub use leader_key::{LeaderKeyPlugin, LeaderKeyState};
pub use keybindings::{KeybindingsPlugin, KeyBinding, KeyBindingConfig};
pub use animations::{AnimationsPlugin, FadeAnimation, AnimationState};
pub use visual_selection::{VisualSelectionPlugin, SelectionMode, SelectionRegion};

use bevy::prelude::*;

/// Main plugin that bundles all UI features
pub struct AdvancedUIPlugin;

impl Plugin for AdvancedUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                LinkHintsPlugin,
                CommandPalettePlugin,
                LeaderKeyPlugin,
                KeybindingsPlugin,
                AnimationsPlugin,
                VisualSelectionPlugin,
            ))
            .insert_resource(UIConfig::default());
    }
}

/// Global UI configuration
#[derive(Resource, Clone)]
pub struct UIConfig {
    pub link_hints_enabled: bool,
    pub command_palette_enabled: bool,
    pub leader_key_enabled: bool,
    pub animations_enabled: bool,
    pub leader_key_timeout_ms: u64,
    pub fuzzy_search_threshold: f64,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            link_hints_enabled: true,
            command_palette_enabled: true,
            leader_key_enabled: true,
            animations_enabled: true,
            leader_key_timeout_ms: 1000,
            fuzzy_search_threshold: 0.3,
        }
    }
}
