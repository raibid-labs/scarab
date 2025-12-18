// Advanced UI/UX module for Scarab terminal emulator
// Provides power-user features: link hints, command palette, leader keys, etc.

pub mod animations;
pub mod breadcrumb;
pub mod command_palette;
pub mod dashboard;
pub mod dock;
pub mod fusabi_widgets;
pub mod grid_utils;
pub mod keybindings;
pub mod leader_key;
pub mod link_hints;
pub mod modes;
pub mod omnibar;
pub mod overlays;
pub mod pickers;
pub mod plugin_menu;
pub mod scroll_indicator;
pub mod scrollback_selection;
pub mod search_overlay;
pub mod status_bar;
pub mod tab_animations;
pub mod visual_selection;

pub use animations::{AnimationState, AnimationsPlugin, FadeAnimation};
pub use breadcrumb::{
    BreadcrumbContainer, BreadcrumbPlugin, BreadcrumbSegmentSelectedEvent, BreadcrumbState,
    BreadcrumbText, OpenDirectoryPickerEvent, PathSegment, BREADCRUMB_BAR_HEIGHT,
};
// Keep command_palette for backwards compatibility but prefer omnibar
pub use command_palette::{Command, CommandPalettePlugin, CommandRegistry};
pub use dashboard::{DashboardConfig, DashboardPlugin, DashboardState, DashboardWidget};
pub use dock::{DockConfig, DockPlugin, DockState};
pub use fusabi_widgets::{FusabiTuiPlugin, FusabiWidgetExamples};
pub use grid_utils::{
    grid_cell_bounds, grid_cell_center, grid_region_bounds, grid_to_pixel,
    grid_to_pixel_with_renderer, pixel_to_grid,
};
pub use keybindings::{KeyBinding, KeyBindingConfig, KeybindingsPlugin};
pub use leader_key::{LeaderKeyPlugin, LeaderKeyState};
pub use link_hints::{LinkDetector, LinkHint, LinkHintsPlugin};
pub use modes::{ModesPlugin, ScarabMode, ScarabModeState};
// Export new omnibar types
pub use omnibar::{
    OmnibarContext, OmnibarExecuteEvent, OmnibarPlugin, OmnibarProvider, OmnibarResult,
    OmnibarState, ProviderRegistry,
};
pub use overlays::RemoteUiPlugin;
pub use pickers::{
    ColorschemePickerState, FontPickerState, PickerItem, PickersPlugin, ShowColorschemePickerEvent,
    ShowFontPickerEvent,
};
pub use plugin_menu::{MenuPosition, MenuState, PluginMenuPlugin, ShowPluginMenuEvent};
pub use scroll_indicator::{ScrollIndicatorConfig, ScrollIndicatorPlugin};
pub use scrollback_selection::{ScrollbackSelectionPlugin, ScrollbackSelectionState};
pub use search_overlay::{SearchOverlayConfig, SearchOverlayPlugin};
pub use status_bar::{
    StatusBarContainer, StatusBarLeft, StatusBarPlugin, StatusBarRight, StatusBarState,
    TabContainer, TabLabel, TabState, TabSwitchEvent, BOTTOM_UI_HEIGHT, DOCK_HEIGHT,
    STATUS_BAR_HEIGHT,
};
pub use tab_animations::{
    TabAnimationConfig, TabAnimationsPlugin, TabEasingFunction, TabFade, TabHover, TabTransition,
};
pub use visual_selection::{SelectionMode, SelectionRegion, VisualSelectionPlugin};

use bevy::prelude::*;

/// Main plugin that bundles all UI features
pub struct AdvancedUIPlugin;

impl Plugin for AdvancedUIPlugin {
    fn build(&self, app: &mut App) {
        // Split plugins into multiple groups to avoid Bevy's tuple limit (16)
        app.add_plugins((
            BreadcrumbPlugin,
            LinkHintsPlugin,
            OmnibarPlugin, // Use new omnibar instead of command palette
            LeaderKeyPlugin,
            KeybindingsPlugin,
            ModesPlugin,
            PickersPlugin,
            AnimationsPlugin,
            TabAnimationsPlugin,
            DashboardPlugin,
            VisualSelectionPlugin,
            RemoteUiPlugin,
            PluginMenuPlugin,
            ScrollIndicatorPlugin,
            ScrollbackSelectionPlugin,
        ))
        .add_plugins((
            SearchOverlayPlugin,
            // DockPlugin disabled - was showing all plugins instead of just status items
            StatusBarPlugin,
        ))
        .insert_resource(UIConfig::default())
        .insert_resource(TabAnimationConfig::default());
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
    pub dock_enabled: bool,
    pub breadcrumb_enabled: bool,
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
            dock_enabled: true,
            breadcrumb_enabled: true,
        }
    }
}
