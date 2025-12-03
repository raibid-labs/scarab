// Developer Tools Module
//
// This module provides debugging and inspection tools for Scarab development.
// All tools in this module are debug-only and should not be included in release builds.
//
// # Available Tools
//
// - `inspector`: Bevy ECS entity and component browser using Ratatui overlay
//
// # Usage
//
// The dev module is conditionally compiled only in debug builds:
//
// ```ignore
// #[cfg(debug_assertions)]
// pub mod dev;
// ```
//
// Enable the inspector in main.rs:
//
// ```ignore
// #[cfg(debug_assertions)]
// use scarab_client::dev::BevyInspectorPlugin;
//
// #[cfg(debug_assertions)]
// app.add_plugins(BevyInspectorPlugin);
// ```

#[cfg(debug_assertions)]
pub mod inspector;

#[cfg(debug_assertions)]
pub use inspector::{BevyInspectorPlugin, BevyInspectorState};
