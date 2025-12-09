//! ECS-safe host bindings for Fusabi plugins
//!
//! This module provides host bindings that allow Fusabi-powered plugins to interact
//! with Scarab's ECS (Bevy) architecture without direct World access. All interactions
//! go through message passing and the plugin host system.
//!
//! # Architecture
//!
//! Plugins communicate via `PluginAction` events which are processed by the client's
//! plugin host system. This provides:
//!
//! - **Safety**: No direct ECS/World mutation from plugin code
//! - **Sandboxing**: Per-plugin capability flags and quotas
//! - **Rate limiting**: Protection against runaway plugins
//!
//! # Example (Fusabi Script)
//!
//! ```fsharp
//! module MyPlugin
//!
//! open Scarab.Host
//!
//! [<OnLoad>]
//! let onLoad (ctx: PluginContext) =
//!     // Register a focusable region
//!     Host.registerFocusable ctx {
//!         X = 10us
//!         Y = 5us
//!         Width = 20us
//!         Height = 1us
//!         Label = "Click me"
//!         Action = OpenUrl "https://example.com"
//!     }
//!
//!     // Enter hint mode
//!     Host.enterHintMode ctx
//! ```
//!
//! # Safety Constraints
//!
//! All host bindings enforce safety constraints:
//!
//! | Constraint | Default | Description |
//! |------------|---------|-------------|
//! | `max_focusables` | 50 | Max focusables per plugin |
//! | `max_overlays` | 10 | Max overlays per plugin |
//! | `max_status_items` | 5 | Max status bar items per plugin |
//! | `rate_limit` | 10/sec | Actions per second |
//! | `bounds_check` | enabled | Coordinate validation |
//!
//! See [`HostBindingLimits`] for configuration.

use crate::context::PluginContext;
use crate::error::{PluginError, Result};
use crate::navigation::{
    validate_focusable, PluginFocusable, PluginFocusableAction, PluginNavCapabilities,
};
use crate::types::{JumpDirection, OverlayConfig, StatusBarItem};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

/// Default rate limit (actions per second)
pub const DEFAULT_RATE_LIMIT: u32 = 10;

/// Default maximum focusables per plugin
pub const DEFAULT_MAX_FOCUSABLES: usize = 50;

/// Default maximum overlays per plugin
pub const DEFAULT_MAX_OVERLAYS: usize = 10;

/// Default maximum status items per plugin
pub const DEFAULT_MAX_STATUS_ITEMS: usize = 5;

/// Configuration limits for host bindings
///
/// These limits protect the host from misbehaving plugins by capping
/// resource usage and action rates.
#[derive(Debug, Clone)]
pub struct HostBindingLimits {
    /// Maximum focusable regions a plugin can register
    pub max_focusables: usize,
    /// Maximum overlays a plugin can spawn
    pub max_overlays: usize,
    /// Maximum status bar items a plugin can add
    pub max_status_items: usize,
    /// Actions per second rate limit
    pub rate_limit: u32,
    /// Enable coordinate bounds checking
    pub bounds_check: bool,
    /// Maximum terminal coordinate (x or y)
    pub max_coordinate: u16,
}

impl Default for HostBindingLimits {
    fn default() -> Self {
        Self {
            max_focusables: DEFAULT_MAX_FOCUSABLES,
            max_overlays: DEFAULT_MAX_OVERLAYS,
            max_status_items: DEFAULT_MAX_STATUS_ITEMS,
            rate_limit: DEFAULT_RATE_LIMIT,
            bounds_check: true,
            max_coordinate: 1000,
        }
    }
}

/// Navigation style configuration
///
/// Allows plugins to select their preferred navigation keymap and visual style.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NavStyle {
    /// Default Vimium-style hints (lowercase letters)
    #[default]
    Vimium,
    /// Uppercase letter hints
    VimiumUppercase,
    /// Numeric hints (1, 2, 3, ...)
    Numeric,
    /// Home row keys only (asdfghjkl)
    HomeRow,
    /// Custom character set
    Custom(String),
}

impl NavStyle {
    /// Get the hint characters for this style
    pub fn hint_chars(&self) -> &str {
        match self {
            NavStyle::Vimium => "sadfjklewcmpgh",
            NavStyle::VimiumUppercase => "SADFJKLEWCMPGH",
            NavStyle::Numeric => "1234567890",
            NavStyle::HomeRow => "asdfghjkl",
            NavStyle::Custom(chars) => chars,
        }
    }
}

/// Navigation keymap configuration
///
/// Allows plugins to select or customize the navigation keybindings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NavKeymap {
    /// Default keymap (f=hints, Esc=cancel, Enter=confirm)
    #[default]
    Default,
    /// Vim-style keymap
    Vim,
    /// Emacs-style keymap
    Emacs,
    /// Custom keymap (key -> action mappings)
    Custom(Vec<(String, String)>),
}

/// Rate limiter state for a single plugin
#[derive(Debug)]
pub struct PluginRateLimiter {
    limit: u32,
    count: AtomicU32,
    window_start: Mutex<Instant>,
}

impl PluginRateLimiter {
    /// Create a new rate limiter
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            count: AtomicU32::new(0),
            window_start: Mutex::new(Instant::now()),
        }
    }

    /// Check if an action is allowed, incrementing the counter if so
    pub fn check(&self) -> Result<()> {
        let now = Instant::now();

        // Check if we need to reset the window
        {
            let mut window = self.window_start.lock();
            if now.duration_since(*window).as_secs() >= 1 {
                *window = now;
                self.count.store(0, Ordering::SeqCst);
            }
        }

        let current = self.count.fetch_add(1, Ordering::SeqCst);
        if current >= self.limit {
            self.count.fetch_sub(1, Ordering::SeqCst); // Undo the increment
            return Err(PluginError::RateLimitExceeded {
                current: current + 1,
                limit: self.limit,
            });
        }

        Ok(())
    }

    /// Reset the rate limiter
    pub fn reset(&self) {
        *self.window_start.lock() = Instant::now();
        self.count.store(0, Ordering::SeqCst);
    }
}

/// Resource counter for tracking plugin resource usage
#[derive(Debug)]
pub struct ResourceCounter {
    focusables: AtomicU64,
    overlays: AtomicU64,
    status_items: AtomicU64,
}

impl Default for ResourceCounter {
    fn default() -> Self {
        Self {
            focusables: AtomicU64::new(0),
            overlays: AtomicU64::new(0),
            status_items: AtomicU64::new(0),
        }
    }
}

impl ResourceCounter {
    /// Get current focusable count
    pub fn focusables(&self) -> u64 {
        self.focusables.load(Ordering::SeqCst)
    }

    /// Get current overlay count
    pub fn overlays(&self) -> u64 {
        self.overlays.load(Ordering::SeqCst)
    }

    /// Get current status item count
    pub fn status_items(&self) -> u64 {
        self.status_items.load(Ordering::SeqCst)
    }

    /// Increment focusable count, returns new value
    pub fn add_focusable(&self) -> u64 {
        self.focusables.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Decrement focusable count, returns new value
    pub fn remove_focusable(&self) -> u64 {
        self.focusables
            .fetch_sub(1, Ordering::SeqCst)
            .saturating_sub(1)
    }

    /// Increment overlay count
    pub fn add_overlay(&self) -> u64 {
        self.overlays.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Decrement overlay count
    pub fn remove_overlay(&self) -> u64 {
        self.overlays
            .fetch_sub(1, Ordering::SeqCst)
            .saturating_sub(1)
    }

    /// Increment status item count
    pub fn add_status_item(&self) -> u64 {
        self.status_items.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Decrement status item count
    pub fn remove_status_item(&self) -> u64 {
        self.status_items
            .fetch_sub(1, Ordering::SeqCst)
            .saturating_sub(1)
    }
}

/// ECS-safe host bindings for Fusabi plugins
///
/// This struct provides the bridge between Fusabi scripts and Scarab's ECS.
/// All operations are validated against capability flags, quotas, and rate limits
/// before being queued as `RemoteCommand`s for the plugin host to process.
///
/// # Thread Safety
///
/// All methods are thread-safe and can be called from async Fusabi contexts.
///
/// # Example
///
/// ```ignore
/// let bindings = HostBindings::new(limits, capabilities);
///
/// // Register a focusable (checks quotas and validates)
/// bindings.register_focusable(&ctx, PluginFocusable {
///     x: 10, y: 5, width: 20, height: 1,
///     label: "GitHub".into(),
///     action: PluginFocusableAction::OpenUrl("https://github.com".into()),
/// })?;
///
/// // Enter hint mode (checks capability and rate limit)
/// bindings.enter_hint_mode(&ctx)?;
/// ```
#[derive(Debug)]
pub struct HostBindings {
    /// Configuration limits
    pub limits: HostBindingLimits,
    /// Navigation capabilities
    pub capabilities: PluginNavCapabilities,
    /// Rate limiter
    rate_limiter: PluginRateLimiter,
    /// Resource counters
    resources: ResourceCounter,
    /// Next focusable ID
    next_focusable_id: AtomicU64,
    /// Next overlay ID
    next_overlay_id: AtomicU64,
    /// Next status item ID
    next_status_item_id: AtomicU64,
    /// Selected nav style
    nav_style: Mutex<NavStyle>,
    /// Selected nav keymap
    nav_keymap: Mutex<NavKeymap>,
}

impl HostBindings {
    /// Create new host bindings with the specified limits and capabilities
    pub fn new(limits: HostBindingLimits, capabilities: PluginNavCapabilities) -> Self {
        Self {
            rate_limiter: PluginRateLimiter::new(limits.rate_limit),
            limits,
            capabilities,
            resources: ResourceCounter::default(),
            next_focusable_id: AtomicU64::new(1),
            next_overlay_id: AtomicU64::new(1),
            next_status_item_id: AtomicU64::new(1),
            nav_style: Mutex::new(NavStyle::default()),
            nav_keymap: Mutex::new(NavKeymap::default()),
        }
    }

    /// Create with default limits and capabilities
    pub fn with_defaults() -> Self {
        Self::new(
            HostBindingLimits::default(),
            PluginNavCapabilities::default(),
        )
    }

    /// Check rate limit before action
    fn check_rate_limit(&self) -> Result<()> {
        self.rate_limiter.check()
    }

    /// Enter hint mode
    ///
    /// Triggers the navigation hint mode UI, displaying labels for all
    /// focusable elements.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Plugin doesn't have `can_enter_hint_mode` capability
    /// - Rate limit exceeded
    pub fn enter_hint_mode(&self, ctx: &PluginContext) -> Result<()> {
        if !self.capabilities.can_enter_hint_mode {
            return Err(PluginError::CapabilityDenied("enter_hint_mode".into()));
        }

        self.check_rate_limit()?;

        ctx.queue_command(crate::types::RemoteCommand::NavEnterHintMode {
            plugin_name: ctx.logger_name.clone(),
        });

        Ok(())
    }

    /// Exit navigation mode
    ///
    /// Exits hint mode and returns to normal input handling.
    ///
    /// # Errors
    ///
    /// Returns error if rate limit exceeded
    pub fn exit_nav_mode(&self, ctx: &PluginContext) -> Result<()> {
        self.check_rate_limit()?;

        ctx.queue_command(crate::types::RemoteCommand::NavExitMode {
            plugin_name: ctx.logger_name.clone(),
        });

        Ok(())
    }

    /// Register a focusable region
    ///
    /// Registers a custom navigation target that will appear in hint mode.
    /// The region is validated for bounds and the action is checked for safety.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `region` - Focusable region to register
    ///
    /// # Returns
    ///
    /// Unique ID for this focusable (can be used to unregister)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Plugin doesn't have `can_register_focusables` capability
    /// - Plugin has reached `max_focusables` quota
    /// - Region fails validation (coordinates, dimensions, URL safety)
    /// - Rate limit exceeded
    pub fn register_focusable(&self, ctx: &PluginContext, region: PluginFocusable) -> Result<u64> {
        if !self.capabilities.can_register_focusables {
            return Err(PluginError::CapabilityDenied("register_focusables".into()));
        }

        // Check quota
        let current = self.resources.focusables();
        if current >= self.capabilities.max_focusables as u64 {
            return Err(PluginError::QuotaExceeded {
                resource: "focusables".into(),
                current: current as usize,
                limit: self.capabilities.max_focusables,
            });
        }

        // Validate region
        if self.limits.bounds_check {
            validate_focusable(&region).map_err(|e| PluginError::ValidationError(e.to_string()))?;
        }

        self.check_rate_limit()?;

        let focusable_id = self.next_focusable_id.fetch_add(1, Ordering::SeqCst);
        self.resources.add_focusable();

        // Convert action to protocol format
        let action = match &region.action {
            PluginFocusableAction::OpenUrl(url) => {
                scarab_protocol::NavFocusableAction::OpenUrl(url.clone().into())
            }
            PluginFocusableAction::OpenFile(path) => {
                scarab_protocol::NavFocusableAction::OpenFile(path.clone().into())
            }
            PluginFocusableAction::Custom(name) => {
                scarab_protocol::NavFocusableAction::Custom(name.clone().into())
            }
        };

        ctx.queue_command(crate::types::RemoteCommand::NavRegisterFocusable {
            plugin_name: ctx.logger_name.clone(),
            x: region.x,
            y: region.y,
            width: region.width,
            height: region.height,
            label: region.label.clone(),
            action,
        });

        Ok(focusable_id)
    }

    /// Unregister a focusable region
    ///
    /// Removes a previously registered focusable from the navigation system.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `focusable_id` - ID returned from `register_focusable`
    ///
    /// # Errors
    ///
    /// Returns error if rate limit exceeded
    pub fn unregister_focusable(&self, ctx: &PluginContext, focusable_id: u64) -> Result<()> {
        self.check_rate_limit()?;

        self.resources.remove_focusable();

        ctx.queue_command(crate::types::RemoteCommand::NavUnregisterFocusable {
            plugin_name: ctx.logger_name.clone(),
            focusable_id,
        });

        Ok(())
    }

    /// Set navigation style
    ///
    /// Configures the visual style of navigation hints (character set, appearance).
    pub fn set_nav_style(&self, style: NavStyle) {
        *self.nav_style.lock() = style;
    }

    /// Get current navigation style
    pub fn nav_style(&self) -> NavStyle {
        self.nav_style.lock().clone()
    }

    /// Set navigation keymap
    ///
    /// Configures the keybindings for navigation mode.
    pub fn set_nav_keymap(&self, keymap: NavKeymap) {
        *self.nav_keymap.lock() = keymap;
    }

    /// Get current navigation keymap
    pub fn nav_keymap(&self) -> NavKeymap {
        self.nav_keymap.lock().clone()
    }

    /// Get current resource usage
    pub fn resource_usage(&self) -> ResourceUsage {
        ResourceUsage {
            focusables: self.resources.focusables() as usize,
            overlays: self.resources.overlays() as usize,
            status_items: self.resources.status_items() as usize,
            max_focusables: self.capabilities.max_focusables,
            max_overlays: self.limits.max_overlays,
            max_status_items: self.limits.max_status_items,
        }
    }

    /// Reset rate limiter (useful for testing)
    pub fn reset_rate_limit(&self) {
        self.rate_limiter.reset();
    }

    // ========================================================================
    // New ECS-safe UI/Nav Bindings (Fusabi 0.21.0)
    // ========================================================================

    /// Spawn an overlay at the given position
    ///
    /// Creates a floating overlay element at the specified terminal coordinates.
    /// Overlays are useful for tooltips, popups, and other transient UI elements.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `config` - Overlay configuration (position, content, style)
    ///
    /// # Returns
    ///
    /// Unique ID for this overlay (can be used to remove it later)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Plugin has reached `max_overlays` quota
    /// - Rate limit exceeded
    /// - Overlay position is out of bounds
    pub fn spawn_overlay(&self, ctx: &PluginContext, config: OverlayConfig) -> Result<u64> {
        let current = self.resources.overlays();
        if current >= self.limits.max_overlays as u64 {
            return Err(PluginError::QuotaExceeded {
                resource: "overlays".into(),
                current: current as usize,
                limit: self.limits.max_overlays,
            });
        }

        if self.limits.bounds_check
            && (config.x >= self.limits.max_coordinate || config.y >= self.limits.max_coordinate)
        {
            return Err(PluginError::ValidationError(format!(
                "Overlay position ({}, {}) exceeds max coordinate {}",
                config.x, config.y, self.limits.max_coordinate
            )));
        }

        self.check_rate_limit()?;

        let overlay_id = self.next_overlay_id.fetch_add(1, Ordering::SeqCst);
        self.resources.add_overlay();

        ctx.queue_command(crate::types::RemoteCommand::SpawnOverlay {
            plugin_name: ctx.logger_name.clone(),
            overlay_id,
            config,
        });

        Ok(overlay_id)
    }

    /// Remove a previously spawned overlay
    ///
    /// Removes an overlay by its ID. If the overlay doesn't exist, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `overlay_id` - ID returned from `spawn_overlay`
    ///
    /// # Errors
    ///
    /// Returns error if rate limit exceeded
    pub fn remove_overlay(&self, ctx: &PluginContext, overlay_id: u64) -> Result<()> {
        self.check_rate_limit()?;

        self.resources.remove_overlay();

        ctx.queue_command(crate::types::RemoteCommand::RemoveOverlay {
            plugin_name: ctx.logger_name.clone(),
            overlay_id,
        });

        Ok(())
    }

    /// Add a status bar item
    ///
    /// Adds an item to the terminal status bar. Status items are positioned
    /// based on their priority (higher priority = further right).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `item` - Status bar item configuration
    ///
    /// # Returns
    ///
    /// Unique ID for this status item (can be used to remove it later)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Plugin has reached `max_status_items` quota
    /// - Rate limit exceeded
    pub fn add_status_item(&self, ctx: &PluginContext, item: StatusBarItem) -> Result<u64> {
        let current = self.resources.status_items();
        if current >= self.limits.max_status_items as u64 {
            return Err(PluginError::QuotaExceeded {
                resource: "status_items".into(),
                current: current as usize,
                limit: self.limits.max_status_items,
            });
        }

        self.check_rate_limit()?;

        let item_id = self.next_status_item_id.fetch_add(1, Ordering::SeqCst);
        self.resources.add_status_item();

        ctx.queue_command(crate::types::RemoteCommand::AddStatusItem {
            plugin_name: ctx.logger_name.clone(),
            item_id,
            item,
        });

        Ok(item_id)
    }

    /// Remove a status bar item
    ///
    /// Removes a status bar item by its ID. If the item doesn't exist, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `item_id` - ID returned from `add_status_item`
    ///
    /// # Errors
    ///
    /// Returns error if rate limit exceeded
    pub fn remove_status_item(&self, ctx: &PluginContext, item_id: u64) -> Result<()> {
        self.check_rate_limit()?;

        self.resources.remove_status_item();

        ctx.queue_command(crate::types::RemoteCommand::RemoveStatusItem {
            plugin_name: ctx.logger_name.clone(),
            item_id,
        });

        Ok(())
    }

    /// Trigger prompt jump navigation
    ///
    /// Navigates the terminal viewport to the previous/next command prompt
    /// in the scrollback buffer. Useful for quickly navigating command history.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `direction` - Direction to jump (Up, Down, First, Last)
    ///
    /// # Errors
    ///
    /// Returns error if rate limit exceeded
    pub fn prompt_jump(&self, ctx: &PluginContext, direction: JumpDirection) -> Result<()> {
        self.check_rate_limit()?;

        ctx.queue_command(crate::types::RemoteCommand::PromptJump {
            plugin_name: ctx.logger_name.clone(),
            direction,
        });

        Ok(())
    }

    // ========================================================================
    // Theme Manipulation Bindings
    // ========================================================================

    /// Apply a named theme
    ///
    /// Dynamically applies a color theme to the terminal. The theme must be
    /// one of the built-in themes or a custom theme registered with the
    /// configuration system.
    ///
    /// # Built-in Themes
    ///
    /// - `slime` - Vibrant green tones (default)
    /// - `dracula` - Dark purple theme
    /// - `nord` - Arctic, north-bluish color palette
    /// - `monokai` - Classic dark theme with warm accents
    /// - `gruvbox_dark` - Retro groove color scheme
    /// - `solarized_dark` - Precision colors for machines
    /// - `solarized_light` - Light variant of solarized
    /// - `tokyo_night` - A clean, dark theme from Tokyo
    /// - `catppuccin` - Soothing pastel theme
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `theme_name` - Name of the theme to apply
    ///
    /// # Errors
    ///
    /// Returns error if rate limit exceeded
    ///
    /// # Example
    ///
    /// ```fsharp
    /// Host.applyTheme ctx "dracula"
    /// ```
    pub fn apply_theme(&self, ctx: &PluginContext, theme_name: &str) -> Result<()> {
        self.check_rate_limit()?;

        ctx.queue_command(crate::types::RemoteCommand::ApplyTheme {
            plugin_name: ctx.logger_name.clone(),
            theme_name: theme_name.to_string(),
        });

        Ok(())
    }

    /// Set a specific palette color
    ///
    /// Modifies a single color in the current palette. This allows fine-grained
    /// customization of terminal colors without changing the entire theme.
    ///
    /// # Color Names
    ///
    /// - `foreground`, `background`
    /// - `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
    /// - `bright_black`, `bright_red`, `bright_green`, `bright_yellow`
    /// - `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`
    /// - `cursor`, `selection`
    ///
    /// # Color Values
    ///
    /// Colors can be specified as:
    /// - Hex: `#RRGGBB` or `RRGGBB`
    /// - RGB: `rgb(255, 0, 128)`
    /// - Named: `red`, `green`, `blue`, etc.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    /// * `color_name` - Name of the color to set
    /// * `value` - New color value (hex, rgb, or named)
    ///
    /// # Errors
    ///
    /// Returns error if rate limit exceeded
    ///
    /// # Example
    ///
    /// ```fsharp
    /// Host.setPaletteColor ctx "foreground" "#00FF00"
    /// Host.setPaletteColor ctx "background" "rgb(30, 30, 30)"
    /// ```
    pub fn set_palette_color(
        &self,
        ctx: &PluginContext,
        color_name: &str,
        value: &str,
    ) -> Result<()> {
        self.check_rate_limit()?;

        ctx.queue_command(crate::types::RemoteCommand::SetPaletteColor {
            plugin_name: ctx.logger_name.clone(),
            color_name: color_name.to_string(),
            value: value.to_string(),
        });

        Ok(())
    }

    /// Request the current theme name
    ///
    /// Queries the current active theme name. Since this is an async operation,
    /// the result will be delivered via a callback or event.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Plugin context
    ///
    /// # Errors
    ///
    /// Returns error if rate limit exceeded
    ///
    /// # Note
    ///
    /// The theme name is returned asynchronously. Plugins should listen for
    /// the `ThemeInfoResponse` event to receive the result.
    pub fn get_current_theme(&self, ctx: &PluginContext) -> Result<()> {
        self.check_rate_limit()?;

        ctx.queue_command(crate::types::RemoteCommand::GetCurrentTheme {
            plugin_name: ctx.logger_name.clone(),
        });

        Ok(())
    }
}

/// Current resource usage snapshot
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Current focusable count
    pub focusables: usize,
    /// Current overlay count
    pub overlays: usize,
    /// Current status item count
    pub status_items: usize,
    /// Maximum focusables allowed
    pub max_focusables: usize,
    /// Maximum overlays allowed
    pub max_overlays: usize,
    /// Maximum status items allowed
    pub max_status_items: usize,
}

impl ResourceUsage {
    /// Check if any resource is at its limit
    pub fn any_at_limit(&self) -> bool {
        self.focusables >= self.max_focusables
            || self.overlays >= self.max_overlays
            || self.status_items >= self.max_status_items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{PluginConfigData, PluginSharedState};
    use std::sync::Arc;

    fn make_test_ctx() -> PluginContext {
        PluginContext::new(
            PluginConfigData::default(),
            Arc::new(Mutex::new(PluginSharedState::new(80, 24))),
            "test_plugin",
        )
    }

    #[test]
    fn test_host_bindings_creation() {
        let bindings = HostBindings::with_defaults();
        assert_eq!(bindings.limits.max_focusables, DEFAULT_MAX_FOCUSABLES);
        assert_eq!(bindings.limits.rate_limit, DEFAULT_RATE_LIMIT);
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = PluginRateLimiter::new(3);

        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_err());

        limiter.reset();
        assert!(limiter.check().is_ok());
    }

    #[test]
    fn test_resource_counter() {
        let counter = ResourceCounter::default();

        assert_eq!(counter.focusables(), 0);
        assert_eq!(counter.add_focusable(), 1);
        assert_eq!(counter.add_focusable(), 2);
        assert_eq!(counter.focusables(), 2);
        assert_eq!(counter.remove_focusable(), 1);
        assert_eq!(counter.focusables(), 1);
    }

    #[test]
    fn test_nav_style_hint_chars() {
        assert_eq!(NavStyle::Vimium.hint_chars(), "sadfjklewcmpgh");
        assert_eq!(NavStyle::Numeric.hint_chars(), "1234567890");
        assert_eq!(NavStyle::Custom("abc".into()).hint_chars(), "abc");
    }

    #[test]
    fn test_capability_denied() {
        let ctx = make_test_ctx();
        let caps = PluginNavCapabilities {
            can_enter_hint_mode: false,
            ..Default::default()
        };
        let bindings = HostBindings::new(HostBindingLimits::default(), caps);

        let result = bindings.enter_hint_mode(&ctx);
        assert!(matches!(result, Err(PluginError::CapabilityDenied(_))));
    }

    #[test]
    fn test_quota_exceeded() {
        let ctx = make_test_ctx();
        let caps = PluginNavCapabilities {
            max_focusables: 1,
            ..Default::default()
        };
        let bindings = HostBindings::new(HostBindingLimits::default(), caps);

        // First should succeed
        let region = PluginFocusable {
            x: 0,
            y: 0,
            width: 10,
            height: 1,
            label: "Test".into(),
            action: PluginFocusableAction::OpenUrl("https://example.com".into()),
        };
        assert!(bindings.register_focusable(&ctx, region.clone()).is_ok());

        // Second should fail quota
        let result = bindings.register_focusable(&ctx, region);
        assert!(matches!(result, Err(PluginError::QuotaExceeded { .. })));
    }

    #[test]
    fn test_resource_usage() {
        let bindings = HostBindings::with_defaults();
        let usage = bindings.resource_usage();

        assert_eq!(usage.focusables, 0);
        assert!(!usage.any_at_limit());
    }
}
