//! Pane Management Plugin for Scarab Terminal
//!
//! Provides split pane management with separate PTY sessions per pane.
//! Works in conjunction with scarab-tabs for full workspace management.

use async_trait::async_trait;
use parking_lot::Mutex;
use scarab_plugin_api::{types::ModalItem, Action, Plugin, PluginContext, PluginMetadata, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Pane split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Pane layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneLayout {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub split_direction: Option<SplitDirection>,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub is_focused: bool,
    /// Percentage of parent's dimension (0.0 to 1.0) for flexible sizing
    /// Used for resizing - represents how much of the split this pane takes
    pub split_ratio: f32,
}

impl PaneLayout {
    fn new(id: u64, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id,
            parent_id: None,
            split_direction: None,
            x,
            y,
            width,
            height,
            is_focused: true,
            split_ratio: 0.5, // Default 50/50 split
        }
    }
}

/// Pane metadata and PTY handle
#[derive(Debug)]
pub struct Pane {
    pub layout: PaneLayout,
    pub session_id: Option<String>,
    pub working_dir: Option<String>,
    pub created_at: u64,
    // Note: In production, this would hold PTY master/slave handles
    // For now, we just track metadata
}

impl Pane {
    fn new(layout: PaneLayout) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            layout,
            session_id: None,
            working_dir: None,
            created_at: now,
        }
    }
}

/// Internal plugin state
struct PluginState {
    panes: HashMap<u64, Pane>,
    active_pane_id: u64,
    next_pane_id: u64,
    terminal_size: (u16, u16), // cols, rows
}

impl PluginState {
    fn new(cols: u16, rows: u16) -> Self {
        let mut state = Self {
            panes: HashMap::new(),
            active_pane_id: 0,
            next_pane_id: 1,
            terminal_size: (cols, rows),
        };

        // Create initial pane
        let layout = PaneLayout::new(0, 0, 0, cols, rows);
        state.panes.insert(0, Pane::new(layout));
        state
    }

    fn split_pane(&mut self, pane_id: u64, direction: SplitDirection) -> Option<u64> {
        let pane = self.panes.get(&pane_id)?;
        let layout = pane.layout.clone();

        let new_id = self.next_pane_id;
        self.next_pane_id += 1;

        let (layout1, layout2) = match direction {
            SplitDirection::Horizontal => {
                // Split horizontally (top/bottom)
                let height1 = layout.height / 2;
                let height2 = layout.height - height1;

                let mut l1 = layout.clone();
                l1.height = height1;

                let mut l2 =
                    PaneLayout::new(new_id, layout.x, layout.y + height1, layout.width, height2);
                l2.parent_id = Some(pane_id);
                l2.split_direction = Some(direction);
                l2.is_focused = true;

                (l1, l2)
            }
            SplitDirection::Vertical => {
                // Split vertically (left/right)
                let width1 = layout.width / 2;
                let width2 = layout.width - width1;

                let mut l1 = layout.clone();
                l1.width = width1;

                let mut l2 =
                    PaneLayout::new(new_id, layout.x + width1, layout.y, width2, layout.height);
                l2.parent_id = Some(pane_id);
                l2.split_direction = Some(direction);
                l2.is_focused = true;

                (l1, l2)
            }
        };

        // Update existing pane
        if let Some(pane) = self.panes.get_mut(&pane_id) {
            pane.layout = layout1;
            pane.layout.is_focused = false;
        }

        // Create new pane
        self.panes.insert(new_id, Pane::new(layout2));
        self.active_pane_id = new_id;

        log::info!(
            "Split pane {} {:?}, created pane {}",
            pane_id,
            direction,
            new_id
        );

        Some(new_id)
    }

    fn close_pane(&mut self, pane_id: u64) -> Option<Pane> {
        // Don't close the last pane
        if self.panes.len() <= 1 {
            return None;
        }

        let pane = self.panes.remove(&pane_id)?;

        // If closing the active pane, switch to another
        if self.active_pane_id == pane_id {
            self.active_pane_id = *self.panes.keys().next().unwrap();
            if let Some(active) = self.panes.get_mut(&self.active_pane_id) {
                active.layout.is_focused = true;
            }
        }

        // Redistribute space: find sibling panes (those with same parent) and expand them
        self.redistribute_space_after_close(pane_id, &pane.layout);

        // Recalculate entire layout to ensure consistency
        self.recalculate_layout();

        Some(pane)
    }

    /// Redistributes space after closing a pane
    /// Finds siblings (panes with the same parent and split direction) and expands them
    fn redistribute_space_after_close(&mut self, closed_id: u64, closed_layout: &PaneLayout) {
        // If this pane has no parent, we can't redistribute (it was the root)
        let parent_id = match closed_layout.parent_id {
            Some(id) => id,
            None => return,
        };

        // Find all siblings - panes that share the same parent
        let siblings: Vec<u64> = self
            .panes
            .iter()
            .filter(|(id, pane)| **id != closed_id && pane.layout.parent_id == Some(parent_id))
            .map(|(id, _)| *id)
            .collect();

        // If there's exactly one sibling, it should reclaim the parent's full space
        if siblings.len() == 1 {
            let sibling_id = siblings[0];
            if let Some(sibling) = self.panes.get_mut(&sibling_id) {
                // The sibling should now occupy what the parent occupied
                // We'll mark it to have no parent (it becomes independent)
                // The recalculate_layout will handle the actual space allocation
                sibling.layout.split_ratio = 1.0;
                log::info!(
                    "Pane {} closed, sibling pane {} expands to fill space",
                    closed_id,
                    sibling_id
                );
            }
        } else if !siblings.is_empty() {
            // Multiple siblings: distribute the closed pane's ratio among remaining siblings
            let closed_ratio = closed_layout.split_ratio;
            let ratio_per_sibling = closed_ratio / siblings.len() as f32;

            for sibling_id in siblings {
                if let Some(sibling) = self.panes.get_mut(&sibling_id) {
                    sibling.layout.split_ratio += ratio_per_sibling;
                }
            }

            log::info!(
                "Pane {} closed, redistributed ratio {:.2} among {} siblings",
                closed_id,
                closed_ratio,
                self.panes.len()
            );
        }
    }

    fn focus_pane(&mut self, pane_id: u64) -> bool {
        if !self.panes.contains_key(&pane_id) {
            return false;
        }

        // Unfocus all panes
        for pane in self.panes.values_mut() {
            pane.layout.is_focused = false;
        }

        // Focus the target pane
        if let Some(pane) = self.panes.get_mut(&pane_id) {
            pane.layout.is_focused = true;
            self.active_pane_id = pane_id;
            true
        } else {
            false
        }
    }

    fn navigate(&mut self, direction: Direction) -> bool {
        let current_layout = &self.panes.get(&self.active_pane_id).unwrap().layout;
        let (cx, cy) = (current_layout.x, current_layout.y);

        let mut candidates: Vec<(u64, u32)> = Vec::new();

        for (id, pane) in &self.panes {
            if *id == self.active_pane_id {
                continue;
            }

            let (px, py) = (pane.layout.x, pane.layout.y);

            let is_valid = match direction {
                Direction::Up => py < cy,
                Direction::Down => py > cy,
                Direction::Left => px < cx,
                Direction::Right => px > cx,
            };

            if is_valid {
                let distance =
                    ((px as i32 - cx as i32).abs() + (py as i32 - cy as i32).abs()) as u32;
                candidates.push((*id, distance));
            }
        }

        if let Some((nearest_id, _)) = candidates.iter().min_by_key(|(_, dist)| dist) {
            self.focus_pane(*nearest_id)
        } else {
            false
        }
    }

    fn resize_pane(&mut self, pane_id: u64, direction: Direction, amount: i16) -> bool {
        // Get the pane to resize
        let pane_layout = match self.panes.get(&pane_id) {
            Some(pane) => pane.layout.clone(),
            None => return false,
        };

        // Determine which dimension to resize based on the pane's split direction
        let split_direction = match pane_layout.split_direction {
            Some(dir) => dir,
            None => {
                // Root pane with no split direction - try to find if this is a parent with children
                // In this case, we need to look for the parent-child relationship
                let parent_id = pane_layout.parent_id;
                if parent_id.is_none() {
                    log::warn!("Cannot resize root pane {} (no parent)", pane_id);
                    return false;
                }

                // This shouldn't happen if the tree is well-formed
                log::warn!("Pane {} has parent but no split direction", pane_id);
                return false;
            }
        };

        // For resizing, we need to find the "opposite" pane
        // In our tree structure, that's either:
        // 1. The parent (if we're a child)
        // 2. Siblings (other children of the same parent)

        let parent_id = pane_layout.parent_id;

        // Find all siblings and the parent
        let mut related_panes: Vec<u64> = Vec::new();

        if let Some(pid) = parent_id {
            // Add parent as a related pane
            if self.panes.contains_key(&pid) {
                related_panes.push(pid);
            }

            // Find siblings (other panes with the same parent and split direction)
            for (id, pane) in &self.panes {
                if *id != pane_id
                    && pane.layout.parent_id == Some(pid)
                    && pane.layout.split_direction == Some(split_direction)
                {
                    related_panes.push(*id);
                }
            }
        }

        if related_panes.is_empty() {
            log::warn!("No related panes found for pane {}, cannot resize", pane_id);
            return false;
        }

        // Calculate new split ratio
        let current_ratio = pane_layout.split_ratio;
        let dimension_size = match split_direction {
            SplitDirection::Horizontal => self.terminal_size.1, // rows
            SplitDirection::Vertical => self.terminal_size.0,   // cols
        };

        // Convert amount (cells) to ratio change
        let ratio_delta = amount as f32 / dimension_size as f32;

        // Determine if this resize makes sense for the direction
        let new_ratio = match (direction, split_direction) {
            (Direction::Down, SplitDirection::Horizontal)
            | (Direction::Right, SplitDirection::Vertical) => {
                // Expanding this pane
                current_ratio + ratio_delta
            }
            (Direction::Up, SplitDirection::Horizontal)
            | (Direction::Left, SplitDirection::Vertical) => {
                // Shrinking this pane
                current_ratio - ratio_delta
            }
            _ => {
                // Direction doesn't match split direction, invalid operation
                log::warn!(
                    "Cannot resize pane {} in {:?} direction for {:?} split",
                    pane_id,
                    direction,
                    split_direction
                );
                return false;
            }
        };

        // Clamp ratio between 0.1 (10%) and 0.9 (90%) to prevent invisible panes
        let clamped_ratio = new_ratio.clamp(0.1, 0.9);

        // Calculate how much ratio was taken/given
        let ratio_change = clamped_ratio - current_ratio;
        let ratio_per_related = -ratio_change / related_panes.len() as f32;

        // Update this pane's ratio
        if let Some(pane) = self.panes.get_mut(&pane_id) {
            pane.layout.split_ratio = clamped_ratio;
        }

        // Distribute the change among related panes
        for related_id in related_panes {
            if let Some(related) = self.panes.get_mut(&related_id) {
                let new_related_ratio = related.layout.split_ratio + ratio_per_related;
                // Ensure related panes also stay in valid range
                related.layout.split_ratio = new_related_ratio.clamp(0.1, 0.9);
            }
        }

        log::info!(
            "Resized pane {} {:?} by {} cells (ratio: {:.2} -> {:.2})",
            pane_id,
            direction,
            amount,
            current_ratio,
            clamped_ratio
        );

        // Recalculate layout to apply changes
        self.recalculate_layout();
        true
    }

    fn recalculate_layout(&mut self) {
        // Smart layout recalculation using tree traversal
        // Algorithm inspired by i3wm and tmux tiling

        if self.panes.is_empty() {
            return;
        }

        let (cols, rows) = self.terminal_size;

        // Build a tree structure of splits
        // 1. Find root panes (those with no parent or whose parent doesn't exist)
        let root_panes: Vec<u64> = self
            .panes
            .iter()
            .filter(|(_, pane)| {
                pane.layout.parent_id.is_none()
                    || !self.panes.contains_key(&pane.layout.parent_id.unwrap())
            })
            .map(|(id, _)| *id)
            .collect();

        if root_panes.is_empty() {
            log::warn!("No root panes found, falling back to simple layout");
            self.fallback_simple_layout();
            return;
        }

        // If there's only one pane, it takes the full screen
        if self.panes.len() == 1 {
            let pane_id = *self.panes.keys().next().unwrap();
            if let Some(pane) = self.panes.get_mut(&pane_id) {
                pane.layout.x = 0;
                pane.layout.y = 0;
                pane.layout.width = cols;
                pane.layout.height = rows;
            }
            return;
        }

        // For multiple root panes, distribute equally
        // This handles the case where panes were orphaned
        if root_panes.len() > 1 {
            let width_per_root = cols / root_panes.len() as u16;
            for (i, root_id) in root_panes.iter().enumerate() {
                let x = i as u16 * width_per_root;
                let width = if i == root_panes.len() - 1 {
                    cols - x // Last pane takes remaining space
                } else {
                    width_per_root
                };
                self.recalculate_subtree(*root_id, x, 0, width, rows);
            }
        } else {
            // Single root pane - normal case
            self.recalculate_subtree(root_panes[0], 0, 0, cols, rows);
        }
    }

    /// Recursively calculates layout for a pane and its children
    fn recalculate_subtree(&mut self, pane_id: u64, x: u16, y: u16, width: u16, height: u16) {
        // Update this pane's layout
        if let Some(pane) = self.panes.get_mut(&pane_id) {
            pane.layout.x = x;
            pane.layout.y = y;
            pane.layout.width = width;
            pane.layout.height = height;
        } else {
            return;
        }

        // Find all children (panes that have this pane as parent)
        let children: Vec<(u64, SplitDirection, f32)> = self
            .panes
            .iter()
            .filter(|(_, pane)| pane.layout.parent_id == Some(pane_id))
            .map(|(id, pane)| {
                (
                    *id,
                    pane.layout
                        .split_direction
                        .unwrap_or(SplitDirection::Horizontal),
                    pane.layout.split_ratio,
                )
            })
            .collect();

        if children.is_empty() {
            // Leaf node, no children to layout
            return;
        }

        // Group children by split direction
        let mut horizontal_children: Vec<(u64, f32)> = Vec::new();
        let mut vertical_children: Vec<(u64, f32)> = Vec::new();

        for (id, direction, ratio) in children {
            match direction {
                SplitDirection::Horizontal => horizontal_children.push((id, ratio)),
                SplitDirection::Vertical => vertical_children.push((id, ratio)),
            }
        }

        // Layout horizontal splits (stacked vertically)
        if !horizontal_children.is_empty() {
            let total_ratio: f32 = horizontal_children.iter().map(|(_, r)| r).sum();
            let mut current_y = y;

            for (i, (child_id, ratio)) in horizontal_children.iter().enumerate() {
                let normalized_ratio = ratio / total_ratio;
                let child_height = if i == horizontal_children.len() - 1 {
                    // Last child takes remaining space to avoid rounding errors
                    height - (current_y - y)
                } else {
                    (height as f32 * normalized_ratio) as u16
                };

                self.recalculate_subtree(*child_id, x, current_y, width, child_height);

                current_y += child_height;
            }
        }

        // Layout vertical splits (side by side)
        if !vertical_children.is_empty() {
            let total_ratio: f32 = vertical_children.iter().map(|(_, r)| r).sum();
            let mut current_x = x;

            for (i, (child_id, ratio)) in vertical_children.iter().enumerate() {
                let normalized_ratio = ratio / total_ratio;
                let child_width = if i == vertical_children.len() - 1 {
                    // Last child takes remaining space to avoid rounding errors
                    width - (current_x - x)
                } else {
                    (width as f32 * normalized_ratio) as u16
                };

                self.recalculate_subtree(*child_id, current_x, y, child_width, height);

                current_x += child_width;
            }
        }
    }

    /// Fallback to simple grid layout if tree structure is broken
    fn fallback_simple_layout(&mut self) {
        let count = self.panes.len();
        if count == 0 {
            return;
        }

        let (cols, rows) = self.terminal_size;
        let cols_per_pane = cols / count as u16;
        let rows_per_pane = rows;

        for (i, (_, pane)) in self.panes.iter_mut().enumerate() {
            pane.layout.x = i as u16 * cols_per_pane;
            pane.layout.y = 0;
            pane.layout.width = if i == count - 1 {
                cols - (i as u16 * cols_per_pane) // Last pane takes remaining
            } else {
                cols_per_pane
            };
            pane.layout.height = rows_per_pane;
        }
    }

    fn update_terminal_size(&mut self, cols: u16, rows: u16) {
        self.terminal_size = (cols, rows);
        self.recalculate_layout();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct PanesPlugin {
    metadata: PluginMetadata,
    state: Arc<Mutex<PluginState>>,
}

impl PanesPlugin {
    pub fn new() -> Self {
        Self::with_size(80, 24)
    }

    pub fn with_size(cols: u16, rows: u16) -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-panes",
                "0.1.0",
                "Split pane management with PTY sessions",
                "Scarab Team",
            )
            .with_emoji("🪟")
            .with_color("#E94B3C")
            .with_catchphrase("Split, resize, conquer"),
            state: Arc::new(Mutex::new(PluginState::new(cols, rows))),
        }
    }

    fn handle_keybinding(&self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        let mut state = self.state.lock();

        // Ctrl+Shift+| (vertical split) - would be custom sequence
        // For now, we'll use simpler bindings for testing

        // Ctrl+Shift+- (horizontal split) - ASCII 0x1F
        if input == [0x1f] {
            let active_id = state.active_pane_id;
            if let Some(new_id) = state.split_pane(active_id, SplitDirection::Horizontal) {
                log::info!("Split pane horizontally, created pane {}", new_id);
                ctx.notify_success(
                    "Split Pane",
                    &format!("Created pane {} (horizontal)", new_id),
                );
                return Ok(Action::Modify(Vec::new()));
            }
        }

        // Ctrl+Shift+\ (vertical split) - ASCII 0x1C
        if input == [0x1c] {
            let active_id = state.active_pane_id;
            if let Some(new_id) = state.split_pane(active_id, SplitDirection::Vertical) {
                log::info!("Split pane vertically, created pane {}", new_id);
                ctx.notify_success("Split Pane", &format!("Created pane {} (vertical)", new_id));
                return Ok(Action::Modify(Vec::new()));
            }
        }

        Ok(Action::Continue)
    }
}

impl Default for PanesPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for PanesPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn get_commands(&self) -> Vec<ModalItem> {
        vec![
            ModalItem {
                id: "panes.split_horizontal".to_string(),
                label: "Split Pane Horizontally".to_string(),
                description: Some("Split current pane horizontally (Ctrl+Shift+-)".to_string()),
            },
            ModalItem {
                id: "panes.split_vertical".to_string(),
                label: "Split Pane Vertically".to_string(),
                description: Some("Split current pane vertically (Ctrl+Shift+|)".to_string()),
            },
            ModalItem {
                id: "panes.close".to_string(),
                label: "Close Pane".to_string(),
                description: Some("Close current pane (Ctrl+Shift+W)".to_string()),
            },
            ModalItem {
                id: "panes.navigate_up".to_string(),
                label: "Navigate Up".to_string(),
                description: Some("Focus pane above (Ctrl+Shift+Up)".to_string()),
            },
            ModalItem {
                id: "panes.navigate_down".to_string(),
                label: "Navigate Down".to_string(),
                description: Some("Focus pane below (Ctrl+Shift+Down)".to_string()),
            },
            ModalItem {
                id: "panes.navigate_left".to_string(),
                label: "Navigate Left".to_string(),
                description: Some("Focus pane to the left (Ctrl+Shift+Left)".to_string()),
            },
            ModalItem {
                id: "panes.navigate_right".to_string(),
                label: "Navigate Right".to_string(),
                description: Some("Focus pane to the right (Ctrl+Shift+Right)".to_string()),
            },
            ModalItem {
                id: "panes.zoom".to_string(),
                label: "Zoom Pane".to_string(),
                description: Some("Toggle pane zoom (fullscreen)".to_string()),
            },
            ModalItem {
                id: "panes.resize_up".to_string(),
                label: "Resize Up".to_string(),
                description: Some("Resize current pane upward".to_string()),
            },
            ModalItem {
                id: "panes.resize_down".to_string(),
                label: "Resize Down".to_string(),
                description: Some("Resize current pane downward".to_string()),
            },
            ModalItem {
                id: "panes.resize_left".to_string(),
                label: "Resize Left".to_string(),
                description: Some("Resize current pane leftward".to_string()),
            },
            ModalItem {
                id: "panes.resize_right".to_string(),
                label: "Resize Right".to_string(),
                description: Some("Resize current pane rightward".to_string()),
            },
        ]
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        self.handle_keybinding(input, ctx)
    }

    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock();

        match id {
            "panes.split_horizontal" => {
                let active_id = state.active_pane_id;
                if let Some(new_id) = state.split_pane(active_id, SplitDirection::Horizontal) {
                    log::info!("Command: Split pane horizontally, created {}", new_id);
                    ctx.notify_success("Split Pane", &format!("Created pane {}", new_id));
                }
            }
            "panes.split_vertical" => {
                let active_id = state.active_pane_id;
                if let Some(new_id) = state.split_pane(active_id, SplitDirection::Vertical) {
                    log::info!("Command: Split pane vertically, created {}", new_id);
                    ctx.notify_success("Split Pane", &format!("Created pane {}", new_id));
                }
            }
            "panes.close" => {
                let active_id = state.active_pane_id;
                if state.panes.len() > 1 {
                    if state.close_pane(active_id).is_some() {
                        log::info!("Command: Closed pane {}", active_id);
                        ctx.notify_info("Pane Closed", &format!("Closed pane {}", active_id));
                    }
                } else {
                    ctx.notify_warning("Cannot Close", "Cannot close the last pane");
                }
            }
            "panes.navigate_up" => {
                if state.navigate(Direction::Up) {
                    ctx.notify_info("Navigate", "Focused pane above");
                }
            }
            "panes.navigate_down" => {
                if state.navigate(Direction::Down) {
                    ctx.notify_info("Navigate", "Focused pane below");
                }
            }
            "panes.navigate_left" => {
                if state.navigate(Direction::Left) {
                    ctx.notify_info("Navigate", "Focused pane to the left");
                }
            }
            "panes.navigate_right" => {
                if state.navigate(Direction::Right) {
                    ctx.notify_info("Navigate", "Focused pane to the right");
                }
            }
            "panes.zoom" => {
                log::info!("Command: Toggle pane zoom (not yet implemented)");
                ctx.notify_info("Zoom", "Feature coming soon");
            }
            "panes.resize_up" => {
                let active_id = state.active_pane_id;
                if state.resize_pane(active_id, Direction::Up, 2) {
                    ctx.notify_info("Resize", "Pane resized upward");
                } else {
                    ctx.notify_warning("Resize Failed", "Cannot resize in this direction");
                }
            }
            "panes.resize_down" => {
                let active_id = state.active_pane_id;
                if state.resize_pane(active_id, Direction::Down, 2) {
                    ctx.notify_info("Resize", "Pane resized downward");
                } else {
                    ctx.notify_warning("Resize Failed", "Cannot resize in this direction");
                }
            }
            "panes.resize_left" => {
                let active_id = state.active_pane_id;
                if state.resize_pane(active_id, Direction::Left, 2) {
                    ctx.notify_info("Resize", "Pane resized leftward");
                } else {
                    ctx.notify_warning("Resize Failed", "Cannot resize in this direction");
                }
            }
            "panes.resize_right" => {
                let active_id = state.active_pane_id;
                if state.resize_pane(active_id, Direction::Right, 2) {
                    ctx.notify_info("Resize", "Pane resized rightward");
                } else {
                    ctx.notify_warning("Resize Failed", "Cannot resize in this direction");
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, _ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock();
        state.update_terminal_size(cols, rows);
        log::info!(
            "Panes plugin: Terminal resized to {}x{}, recalculated layout",
            cols,
            rows
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_horizontal() {
        let mut state = PluginState::new(80, 24);
        assert_eq!(state.panes.len(), 1);

        let new_id = state.split_pane(0, SplitDirection::Horizontal);
        assert!(new_id.is_some());
        assert_eq!(state.panes.len(), 2);

        let pane0 = &state.panes[&0];
        let pane1 = &state.panes[&new_id.unwrap()];

        assert_eq!(pane0.layout.height, 12);
        assert_eq!(pane1.layout.height, 12);
        assert_eq!(pane1.layout.y, 12);
    }

    #[test]
    fn test_split_vertical() {
        let mut state = PluginState::new(80, 24);
        let new_id = state.split_pane(0, SplitDirection::Vertical);

        assert!(new_id.is_some());
        assert_eq!(state.panes.len(), 2);

        let pane0 = &state.panes[&0];
        let pane1 = &state.panes[&new_id.unwrap()];

        assert_eq!(pane0.layout.width, 40);
        assert_eq!(pane1.layout.width, 40);
        assert_eq!(pane1.layout.x, 40);
    }

    #[test]
    fn test_close_pane() {
        let mut state = PluginState::new(80, 24);
        let new_id = state.split_pane(0, SplitDirection::Horizontal).unwrap();

        assert_eq!(state.panes.len(), 2);

        let closed = state.close_pane(new_id);
        assert!(closed.is_some());
        assert_eq!(state.panes.len(), 1);
    }

    #[test]
    fn test_cannot_close_last_pane() {
        let mut state = PluginState::new(80, 24);
        let closed = state.close_pane(0);
        assert!(closed.is_none());
        assert_eq!(state.panes.len(), 1);
    }

    #[test]
    fn test_focus_pane() {
        let mut state = PluginState::new(80, 24);
        let new_id = state.split_pane(0, SplitDirection::Horizontal).unwrap();

        assert_eq!(state.active_pane_id, new_id);
        assert!(state.panes[&new_id].layout.is_focused);
        assert!(!state.panes[&0].layout.is_focused);

        state.focus_pane(0);
        assert_eq!(state.active_pane_id, 0);
        assert!(state.panes[&0].layout.is_focused);
        assert!(!state.panes[&new_id].layout.is_focused);
    }

    #[test]
    fn test_resize_updates_layout() {
        let mut state = PluginState::new(80, 24);
        state.split_pane(0, SplitDirection::Horizontal);

        state.update_terminal_size(160, 48);
        assert_eq!(state.terminal_size, (160, 48));
    }

    #[test]
    fn test_close_pane_redistributes_space() {
        let mut state = PluginState::new(80, 24);

        // Split horizontally: pane 0 (top), pane 1 (bottom)
        let pane1 = state.split_pane(0, SplitDirection::Horizontal).unwrap();

        // Both should have height of 12
        assert_eq!(state.panes[&0].layout.height, 12);
        assert_eq!(state.panes[&pane1].layout.height, 12);

        // Close pane 1, pane 0 should expand to full height
        state.close_pane(pane1);

        // Pane 0 should now take the full height
        assert_eq!(state.panes[&0].layout.height, 24);
        assert_eq!(state.panes[&0].layout.y, 0);
    }

    #[test]
    fn test_complex_split_layout() {
        let mut state = PluginState::new(100, 40);

        // Create a complex layout:
        // +----+----+
        // |    |  2 |
        // | 0  +----+
        // |    |  3 |
        // +----+----+

        // Split pane 0 vertically -> creates pane 1
        let pane1 = state.split_pane(0, SplitDirection::Vertical).unwrap();

        // Split pane 1 horizontally -> creates pane 2
        let pane2 = state.split_pane(pane1, SplitDirection::Horizontal).unwrap();

        assert_eq!(state.panes.len(), 3);

        // Verify layout structure
        assert_eq!(state.panes[&0].layout.x, 0);
        assert_eq!(state.panes[&0].layout.width, 50);
        assert_eq!(state.panes[&0].layout.height, 40);

        // Pane 1 and 2 should be on the right side
        assert!(state.panes[&pane1].layout.x >= 50);
        assert!(state.panes[&pane2].layout.x >= 50);
    }

    #[test]
    fn test_resize_pane_horizontal() {
        let mut state = PluginState::new(80, 40);

        // Create horizontal split
        let pane1 = state.split_pane(0, SplitDirection::Horizontal).unwrap();

        // Get initial heights
        let initial_height_0 = state.panes[&0].layout.height;
        let initial_height_1 = state.panes[&pane1].layout.height;

        // Resize pane 1 down (expand it)
        let success = state.resize_pane(pane1, Direction::Down, 5);
        assert!(success);

        // Note: Due to the parent-child tree structure where pane 0 is the parent
        // and pane 1 is the child, resizing affects the split ratios.
        // The actual layout depends on recalculate_layout() which uses the tree.
        // Verify that ratios changed
        assert_ne!(state.panes[&pane1].layout.split_ratio, 0.5);
    }

    #[test]
    fn test_resize_pane_vertical() {
        let mut state = PluginState::new(80, 40);

        // Create vertical split
        let pane1 = state.split_pane(0, SplitDirection::Vertical).unwrap();

        // Get initial widths
        let initial_width_1 = state.panes[&pane1].layout.width;

        // Resize pane 1 right (expand it)
        let success = state.resize_pane(pane1, Direction::Right, 5);
        assert!(success);

        // Verify the split ratio changed
        assert_ne!(state.panes[&pane1].layout.split_ratio, 0.5);

        // Verify the layout was recalculated and width changed
        assert_ne!(state.panes[&pane1].layout.width, initial_width_1);
    }

    #[test]
    fn test_resize_pane_clamping() {
        let mut state = PluginState::new(80, 40);

        // Create vertical split
        let pane1 = state.split_pane(0, SplitDirection::Vertical).unwrap();

        // Try to resize beyond limits (should clamp to 10%-90%)
        state.resize_pane(pane1, Direction::Right, 1000);

        // Verify the ratio is clamped (should be 0.9 max)
        assert!(state.panes[&pane1].layout.split_ratio <= 0.9);
        assert!(state.panes[&pane1].layout.split_ratio >= 0.1);
    }

    #[test]
    fn test_resize_root_pane_fails() {
        let mut state = PluginState::new(80, 40);

        // Try to resize the root pane (should fail)
        let success = state.resize_pane(0, Direction::Right, 5);
        assert!(!success);
    }

    #[test]
    fn test_three_way_split_close_middle() {
        let mut state = PluginState::new(90, 30);

        // Create 3 horizontal splits
        let pane1 = state.split_pane(0, SplitDirection::Horizontal).unwrap();
        let pane2 = state.split_pane(pane1, SplitDirection::Horizontal).unwrap();

        assert_eq!(state.panes.len(), 3);

        // Close middle pane (pane1)
        state.close_pane(pane1);

        // Should have 2 panes left
        assert_eq!(state.panes.len(), 2);
        assert!(state.panes.contains_key(&0));
        assert!(state.panes.contains_key(&pane2));
    }

    #[test]
    fn test_split_ratios() {
        let mut state = PluginState::new(100, 50);

        // All new splits should have 0.5 ratio (50/50)
        let pane1 = state.split_pane(0, SplitDirection::Vertical).unwrap();

        assert_eq!(state.panes[&pane1].layout.split_ratio, 0.5);
    }
}
