use super::pane::{Pane, PaneId, Rect};
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

/// Unique identifier for a tab
pub type TabId = u64;

/// Split direction for pane layout
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// A tab containing one or more panes in a layout
///
/// Each tab manages its own set of panes and tracks which pane is currently
/// focused (active). For the MVP, we use a simple flat list of panes.
/// Future versions can implement a tree structure for complex splits.
pub struct Tab {
    pub id: TabId,
    pub title: String,
    /// All panes owned by this tab (flattened ownership)
    panes: HashMap<PaneId, Arc<Pane>>,
    /// The currently focused pane within this tab
    active_pane_id: PaneId,
    /// Next pane ID to assign
    next_pane_id: PaneId,
    /// Tab creation timestamp
    pub created_at: SystemTime,
}

impl Tab {
    /// Create a new tab with a single initial pane
    pub fn new(id: TabId, title: String, shell: &str, cols: u16, rows: u16) -> Result<Self> {
        let pane_id: PaneId = 1;
        let pane = Pane::new(pane_id, shell, cols, rows, None)?;

        let mut panes = HashMap::new();
        panes.insert(pane_id, Arc::new(pane));

        Ok(Self {
            id,
            title,
            panes,
            active_pane_id: pane_id,
            next_pane_id: 2,
            created_at: SystemTime::now(),
        })
    }

    /// Create an empty tab (for restoration)
    pub fn empty(id: TabId, title: String) -> Self {
        Self {
            id,
            title,
            panes: HashMap::new(),
            active_pane_id: 0,
            next_pane_id: 1,
            created_at: SystemTime::now(),
        }
    }

    /// Add a pane to this tab
    pub fn add_pane(&mut self, pane: Pane) -> PaneId {
        let pane_id = pane.id;
        self.panes.insert(pane_id, Arc::new(pane));

        // If this is the first pane, make it active
        if self.active_pane_id == 0 {
            self.active_pane_id = pane_id;
        }

        pane_id
    }

    /// Split the active pane, creating a new pane
    pub fn split_pane(&mut self, direction: SplitDirection, shell: &str) -> Result<PaneId> {
        let active_pane = self
            .get_active_pane()
            .ok_or_else(|| anyhow::anyhow!("No active pane to split"))?;

        let (cols, rows) = active_pane.dimensions();

        // Calculate new dimensions based on split direction
        let (new_cols, new_rows) = match direction {
            SplitDirection::Horizontal => (cols, rows / 2),
            SplitDirection::Vertical => (cols / 2, rows),
        };

        // Create new pane
        let new_pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let new_pane = Pane::new(new_pane_id, shell, new_cols, new_rows, None)?;
        self.panes.insert(new_pane_id, Arc::new(new_pane));

        // Update viewports (simplified - just splits in half)
        self.recalculate_layout()?;

        Ok(new_pane_id)
    }

    /// Close a pane by ID
    pub fn close_pane(&mut self, pane_id: PaneId) -> Result<()> {
        if self.panes.len() <= 1 {
            bail!("Cannot close the last pane in a tab");
        }

        self.panes.remove(&pane_id);

        // If we closed the active pane, switch to another
        if self.active_pane_id == pane_id {
            self.active_pane_id = *self.panes.keys().next().unwrap_or(&0);
        }

        self.recalculate_layout()?;
        Ok(())
    }

    /// Get the active pane
    pub fn get_active_pane(&self) -> Option<Arc<Pane>> {
        self.panes.get(&self.active_pane_id).cloned()
    }

    /// Get the active pane ID
    pub fn active_pane_id(&self) -> PaneId {
        self.active_pane_id
    }

    /// Set the active pane
    pub fn set_active_pane(&mut self, pane_id: PaneId) -> Result<()> {
        if !self.panes.contains_key(&pane_id) {
            bail!("Pane {} not found in tab {}", pane_id, self.id);
        }
        self.active_pane_id = pane_id;
        Ok(())
    }

    /// Get a pane by ID
    pub fn get_pane(&self, pane_id: PaneId) -> Option<Arc<Pane>> {
        self.panes.get(&pane_id).cloned()
    }

    /// Get all panes in this tab
    pub fn panes(&self) -> impl Iterator<Item = &Arc<Pane>> {
        self.panes.values()
    }

    /// Get pane count
    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }

    /// Get all pane IDs in this tab
    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.panes.keys().copied().collect()
    }

    /// Recalculate pane layouts (simplified tiling algorithm)
    fn recalculate_layout(&mut self) -> Result<()> {
        // For MVP: simple horizontal tiling
        // Each pane gets an equal portion of the width
        let pane_count = self.panes.len();
        if pane_count == 0 {
            return Ok(());
        }

        // Get dimensions from first pane (assumes all have same container size)
        let first_pane = self.panes.values().next().unwrap();
        let (total_cols, total_rows) = first_pane.dimensions();

        let pane_width = total_cols / pane_count as u16;
        let mut x_offset = 0u16;

        // Note: We can't mutate Pane directly since it's in Arc.
        // For proper layout updates, we'd need Arc<RwLock<Pane>> or interior mutability.
        // For now, we track layout in a separate structure or accept this limitation.
        // The viewport will be recalculated when compositing to the shared grid.

        for _pane in self.panes.values() {
            // Layout info would be stored separately or pane would use interior mutability
            // For MVP, we'll handle this at composition time
            let _viewport = Rect::new(x_offset, 0, pane_width, total_rows);
            x_offset += pane_width;
        }

        Ok(())
    }

    /// Resize all panes in this tab
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        // For single-pane MVP, just resize the active pane
        if let Some(pane) = self.get_active_pane() {
            pane.resize(cols, rows)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new(1, "Test Tab".to_string(), "bash", 80, 24).unwrap();
        assert_eq!(tab.id, 1);
        assert_eq!(tab.title, "Test Tab");
        assert_eq!(tab.pane_count(), 1);
        assert!(tab.get_active_pane().is_some());
    }

    #[test]
    fn test_tab_active_pane() {
        let tab = Tab::new(1, "Test".to_string(), "bash", 80, 24).unwrap();
        let active = tab.get_active_pane().unwrap();
        assert!(active.has_pty());
    }

    #[test]
    fn test_cannot_close_last_pane() {
        let mut tab = Tab::new(1, "Test".to_string(), "bash", 80, 24).unwrap();
        let pane_id = tab.active_pane_id();
        assert!(tab.close_pane(pane_id).is_err());
    }
}
