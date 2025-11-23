//! Plugin context providing access to terminal state

use crate::{error::Result, types::Cell};
use parking_lot::Mutex;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

/// Shared state accessible to plugins
///
/// This wraps the protocol's SharedState with a simpler interface for plugins.
/// The protocol's SharedState uses #[repr(C)] for IPC, while this one provides
/// a high-level API for plugin development.
#[derive(Debug)]
pub struct PluginSharedState {
    /// Terminal grid cells
    pub cells: Vec<Cell>,
    /// Grid width in columns
    pub cols: u16,
    /// Grid rows
    pub rows: u16,
    /// Current cursor position
    pub cursor: (u16, u16),
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Custom plugin-specific data storage
    pub data: HashMap<String, String>,
}

impl PluginSharedState {
    /// Create new shared state
    pub fn new(cols: u16, rows: u16) -> Self {
        let size = (cols as usize) * (rows as usize);
        Self {
            cells: vec![Cell::default(); size],
            cols,
            rows,
            cursor: (0, 0),
            env: std::env::vars().collect(),
            data: HashMap::new(),
        }
    }

    /// Get cell at position
    pub fn get_cell(&self, x: u16, y: u16) -> Option<Cell> {
        if x >= self.cols || y >= self.rows {
            return None;
        }
        let idx = (y as usize) * (self.cols as usize) + (x as usize);
        self.cells.get(idx).copied()
    }

    /// Set cell at position
    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) -> bool {
        if x >= self.cols || y >= self.rows {
            return false;
        }
        let idx = (y as usize) * (self.cols as usize) + (x as usize);
        if let Some(c) = self.cells.get_mut(idx) {
            *c = cell;
            true
        } else {
            false
        }
    }

    /// Get line of text
    pub fn get_line(&self, y: u16) -> Option<String> {
        if y >= self.rows {
            return None;
        }
        let start = (y as usize) * (self.cols as usize);
        let end = start + (self.cols as usize);
        Some(
            self.cells[start..end]
                .iter()
                .map(|c| c.c)
                .collect::<String>()
                .trim_end()
                .to_string(),
        )
    }
}

/// Context provided to plugins for interacting with the terminal
#[derive(Clone)]
pub struct PluginContext {
    /// Plugin-specific configuration
    pub config: PluginConfigData,
    /// Shared terminal state
    pub state: Arc<Mutex<PluginSharedState>>,
    /// Logger name for this plugin
    pub logger_name: String,
}

impl PluginContext {
    /// Create new plugin context
    pub fn new(
        config: PluginConfigData,
        state: Arc<Mutex<PluginSharedState>>,
        logger_name: impl Into<String>,
    ) -> Self {
        Self {
            config,
            state,
            logger_name: logger_name.into(),
        }
    }

    /// Get cell at position
    pub fn get_cell(&self, x: u16, y: u16) -> Option<Cell> {
        self.state.lock().get_cell(x, y)
    }

    /// Set cell at position
    pub fn set_cell(&self, x: u16, y: u16, cell: Cell) -> bool {
        self.state.lock().set_cell(x, y, cell)
    }

    /// Get line of text at row
    pub fn get_line(&self, y: u16) -> Option<String> {
        self.state.lock().get_line(y)
    }

    /// Get terminal size
    pub fn get_size(&self) -> (u16, u16) {
        let state = self.state.lock();
        (state.cols, state.rows)
    }

    /// Get cursor position
    pub fn get_cursor(&self) -> (u16, u16) {
        self.state.lock().cursor
    }

    /// Get environment variable
    pub fn get_env(&self, key: &str) -> Option<String> {
        self.state.lock().env.get(key).cloned()
    }

    /// Store plugin-specific data
    pub fn set_data(&self, key: impl Into<String>, value: impl Into<String>) {
        self.state.lock().data.insert(key.into(), value.into());
    }

    /// Retrieve plugin-specific data
    pub fn get_data(&self, key: &str) -> Option<String> {
        self.state.lock().data.get(key).cloned()
    }

    /// Log a message (to be implemented with actual logging)
    pub fn log(&self, level: LogLevel, message: &str) {
        // TODO: Integrate with actual logging system
        match level {
            LogLevel::Error => eprintln!("[{}] ERROR: {}", self.logger_name, message),
            LogLevel::Warn => eprintln!("[{}] WARN: {}", self.logger_name, message),
            LogLevel::Info => println!("[{}] INFO: {}", self.logger_name, message),
            LogLevel::Debug => println!("[{}] DEBUG: {}", self.logger_name, message),
        }
    }

    /// Send notification to user (placeholder)
    pub fn notify(&self, message: &str) {
        // TODO: Implement actual notification system
        println!("[NOTIFICATION] {}", message);
    }
}

/// Log levels for plugin logging
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

/// Plugin-specific configuration data
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
pub struct PluginConfigData {
    #[serde(flatten)]
    pub data: HashMap<String, toml::Value>,
}

impl PluginConfigData {
    /// Get configuration value
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T> {
        let value = self
            .data
            .get(key)
            .ok_or_else(|| crate::error::PluginError::ConfigError(format!("Missing key: {}", key)))?;
        T::deserialize(value.clone())
            .map_err(|e| crate::error::PluginError::ConfigError(e.to_string()))
    }

    /// Get optional configuration value
    pub fn get_opt<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| T::deserialize(v.clone()).ok())
    }
}
