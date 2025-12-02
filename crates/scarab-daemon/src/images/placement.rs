//! Image Placement Tracking
//!
//! Manages the positions and state of images displayed in the terminal.
//! Handles scrolling, clearing, and coordinate tracking for images.

use super::ImageData;
use log::debug;

/// A single image placement in the terminal grid
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    /// Unique identifier for this placement
    pub id: u64,
    /// Column position in terminal grid
    pub x: u16,
    /// Row position in terminal grid
    pub y: u16,
    /// Width in terminal cells
    pub width_cells: u16,
    /// Height in terminal cells
    pub height_cells: u16,
    /// Raw image data (PNG, JPEG, GIF, etc.)
    pub image_data: Vec<u8>,
}

/// Manages all active image placements
#[derive(Debug, Default)]
pub struct ImagePlacementState {
    /// All active image placements
    pub placements: Vec<ImagePlacement>,
    /// Counter for generating unique IDs
    next_id: u64,
}

impl ImagePlacementState {
    /// Create a new empty placement state
    pub fn new() -> Self {
        Self {
            placements: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a new image placement at the specified position
    ///
    /// # Arguments
    /// * `x` - Column position in terminal grid
    /// * `y` - Row position in terminal grid
    /// * `data` - Parsed image data from protocol
    ///
    /// # Returns
    /// The unique ID assigned to this placement
    pub fn add_placement(&mut self, x: u16, y: u16, data: ImageData) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        // Calculate cell dimensions based on ImageSize specifications
        // For now, use simple defaults - client will need to do actual sizing
        let width_cells = match data.width {
            super::ImageSize::Auto => 10, // Default to 10 cells wide
            super::ImageSize::Cells(n) => n,
            super::ImageSize::Pixels(_) => 10, // TODO: Convert pixels to cells
            super::ImageSize::Percent(_) => 10, // TODO: Calculate from terminal width
        };

        let height_cells = match data.height {
            super::ImageSize::Auto => 5, // Default to 5 cells high
            super::ImageSize::Cells(n) => n,
            super::ImageSize::Pixels(_) => 5, // TODO: Convert pixels to cells
            super::ImageSize::Percent(_) => 5, // TODO: Calculate from terminal height
        };

        let placement = ImagePlacement {
            id,
            x,
            y,
            width_cells,
            height_cells,
            image_data: data.data,
        };

        debug!(
            "Adding image placement: id={}, pos=({},{}), size=({}x{})",
            id, x, y, width_cells, height_cells
        );

        self.placements.push(placement);
        id
    }

    /// Remove an image placement by ID
    ///
    /// # Arguments
    /// * `id` - The unique ID of the placement to remove
    pub fn remove_placement(&mut self, id: u64) {
        self.placements.retain(|p| p.id != id);
        debug!("Removed image placement: id={}", id);
    }

    /// Adjust image positions when the terminal scrolls
    ///
    /// # Arguments
    /// * `lines` - Number of lines scrolled (positive = scroll up, negative = scroll down)
    pub fn scroll(&mut self, lines: i32) {
        if lines == 0 {
            return;
        }

        // Update y positions and remove images that scrolled off-screen
        self.placements.retain_mut(|placement| {
            let new_y = placement.y as i32 - lines;

            // Remove if scrolled off the top
            if new_y < 0 {
                debug!("Image {} scrolled off top", placement.id);
                return false;
            }

            // TODO: Remove if scrolled off the bottom (need terminal height)

            placement.y = new_y as u16;
            true
        });

        if lines > 0 {
            debug!("Scrolled up {} lines, {} placements remain", lines, self.placements.len());
        } else {
            debug!("Scrolled down {} lines, {} placements remain", -lines, self.placements.len());
        }
    }

    /// Clear all image placements
    pub fn clear(&mut self) {
        let count = self.placements.len();
        self.placements.clear();
        if count > 0 {
            debug!("Cleared {} image placements", count);
        }
    }

    /// Get the number of active placements
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// Check if there are no active placements
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    /// Get placements in a specific row range
    ///
    /// Useful for partial screen updates
    pub fn get_in_range(&self, start_y: u16, end_y: u16) -> Vec<&ImagePlacement> {
        self.placements
            .iter()
            .filter(|p| p.y >= start_y && p.y < end_y)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::images::ImageSize;

    fn make_test_image_data() -> ImageData {
        ImageData {
            data: vec![0xFF; 100], // Dummy image data
            width: ImageSize::Cells(10),
            height: ImageSize::Cells(5),
            preserve_aspect_ratio: true,
            inline: true,
            do_not_move_cursor: false,
            filename: None,
        }
    }

    #[test]
    fn test_add_placement() {
        let mut state = ImagePlacementState::new();
        let data = make_test_image_data();

        let id = state.add_placement(5, 10, data);
        assert_eq!(id, 1);
        assert_eq!(state.len(), 1);

        let placement = &state.placements[0];
        assert_eq!(placement.x, 5);
        assert_eq!(placement.y, 10);
        assert_eq!(placement.width_cells, 10);
        assert_eq!(placement.height_cells, 5);
    }

    #[test]
    fn test_add_multiple_placements() {
        let mut state = ImagePlacementState::new();

        let id1 = state.add_placement(0, 0, make_test_image_data());
        let id2 = state.add_placement(10, 10, make_test_image_data());
        let id3 = state.add_placement(20, 20, make_test_image_data());

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_eq!(state.len(), 3);
    }

    #[test]
    fn test_remove_placement() {
        let mut state = ImagePlacementState::new();

        let id1 = state.add_placement(0, 0, make_test_image_data());
        let id2 = state.add_placement(10, 10, make_test_image_data());

        state.remove_placement(id1);
        assert_eq!(state.len(), 1);
        assert_eq!(state.placements[0].id, id2);

        state.remove_placement(id2);
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_scroll_up() {
        let mut state = ImagePlacementState::new();

        state.add_placement(0, 10, make_test_image_data());
        state.add_placement(0, 20, make_test_image_data());
        state.add_placement(0, 5, make_test_image_data());

        // Scroll up 3 lines
        state.scroll(3);

        assert_eq!(state.len(), 3);
        assert_eq!(state.placements[0].y, 7);  // 10 - 3
        assert_eq!(state.placements[1].y, 17); // 20 - 3
        assert_eq!(state.placements[2].y, 2);  // 5 - 3
    }

    #[test]
    fn test_scroll_removes_off_screen() {
        let mut state = ImagePlacementState::new();

        state.add_placement(0, 2, make_test_image_data());
        state.add_placement(0, 10, make_test_image_data());

        // Scroll up 5 lines - first image should be removed
        state.scroll(5);

        assert_eq!(state.len(), 1);
        assert_eq!(state.placements[0].y, 5); // 10 - 5
    }

    #[test]
    fn test_scroll_down() {
        let mut state = ImagePlacementState::new();

        state.add_placement(0, 10, make_test_image_data());
        state.add_placement(0, 20, make_test_image_data());

        // Scroll down 3 lines
        state.scroll(-3);

        assert_eq!(state.len(), 2);
        assert_eq!(state.placements[0].y, 13); // 10 + 3
        assert_eq!(state.placements[1].y, 23); // 20 + 3
    }

    #[test]
    fn test_clear() {
        let mut state = ImagePlacementState::new();

        state.add_placement(0, 0, make_test_image_data());
        state.add_placement(10, 10, make_test_image_data());
        state.add_placement(20, 20, make_test_image_data());

        assert_eq!(state.len(), 3);
        state.clear();
        assert_eq!(state.len(), 0);
        assert!(state.is_empty());
    }

    #[test]
    fn test_get_in_range() {
        let mut state = ImagePlacementState::new();

        state.add_placement(0, 5, make_test_image_data());
        state.add_placement(0, 15, make_test_image_data());
        state.add_placement(0, 25, make_test_image_data());

        let in_range = state.get_in_range(10, 20);
        assert_eq!(in_range.len(), 1);
        assert_eq!(in_range[0].y, 15);
    }

    #[test]
    fn test_auto_sizing() {
        let mut state = ImagePlacementState::new();

        let mut data = make_test_image_data();
        data.width = ImageSize::Auto;
        data.height = ImageSize::Auto;

        state.add_placement(0, 0, data);

        // Should use default sizes
        assert_eq!(state.placements[0].width_cells, 10);
        assert_eq!(state.placements[0].height_cells, 5);
    }
}
