//! Input handling for Scarab client
//!
//! This module provides key table and modal input handling for the Bevy client.

pub mod key_tables;
pub mod nav_input;

pub use key_tables::{KeyTableStackResource, LeaderKeyResource};
pub use nav_input::{
    route_nav_input, KeyBinding, ModeStack, Modifier, NavAction, NavInputRouter, NavMode, NavStyle,
};
