use crate::images::{
    parse_iterm2_image, parse_sixel_dcs, ImageFormat, ImagePlacementState, ImageSize,
};
use scarab_protocol::{Cell, SharedState, ZoneTracker, GRID_HEIGHT, GRID_WIDTH};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// VTE (Virtual Terminal Emulator) Parser Integration
///
/// This module implements the VTE parser to handle ANSI escape sequences
/// and update terminal state with proper terminal emulation.
///
/// Features:
/// - Parse ANSI/VT100 escape sequences
/// - Handle cursor positioning and scrolling
/// - Support colors and text attributes
/// - UTF-8 multibyte character handling
/// - Scrollback buffer (10k lines)
/// - Image protocol support (iTerm2)
/// - Instance-based grid storage (for multiplexing)
/// - OSC 133 shell integration markers
use vte::{Parser, Perform};

/// Maximum scrollback buffer size (10,000 lines)
const SCROLLBACK_SIZE: usize = 10_000;

/// Maximum images per pane (matches SharedImageBuffer MAX_IMAGES)
const MAX_IMAGES_PER_PANE: usize = 64;

/// Default colors
const DEFAULT_FG: u32 = 0xFFCCCCCC; // Light gray
const DEFAULT_BG: u32 = 0xFF000000; // Black

/// Text attribute flags
pub const FLAG_BOLD: u8 = 1 << 0;
pub const FLAG_ITALIC: u8 = 1 << 1;
pub const FLAG_UNDERLINE: u8 = 1 << 2;
pub const FLAG_INVERSE: u8 = 1 << 3;
pub const FLAG_DIM: u8 = 1 << 4;

/// Shell prompt marker types (OSC 133)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptMarkerType {
    /// Prompt start (133;A)
    PromptStart,
    /// Command start / input begins (133;B)
    CommandStart,
    /// Command executed / output begins (133;C)
    CommandExecuted,
    /// Command finished with exit code (133;D)
    CommandFinished { exit_code: i32 },
}

/// A shell prompt marker at a specific line
#[derive(Debug, Clone)]
pub struct PromptMarker {
    pub marker_type: PromptMarkerType,
    /// Absolute line number in scrollback (scrollback lines + current line)
    pub line: usize,
    /// Timestamp when marker was recorded
    pub timestamp: Instant,
}

/// Current text attributes for rendering
#[derive(Clone, Copy, Debug)]
struct TextAttributes {
    fg: u32,
    bg: u32,
    flags: u8,
}

impl Default for TextAttributes {
    fn default() -> Self {
        Self {
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            flags: 0,
        }
    }
}

/// Local grid buffer for off-screen rendering
///
/// Each TerminalState owns its own Grid, enabling multiple panes
/// to have independent terminal content. The active pane's Grid
/// is "blitted" (copied) to SharedState for client rendering.
#[derive(Clone)]
pub struct Grid {
    /// Cell data for the grid
    pub cells: Vec<Cell>,
    /// Number of columns
    pub cols: u16,
    /// Number of rows
    pub rows: u16,
}

impl Grid {
    /// Create a new grid with the specified dimensions
    pub fn new(cols: u16, rows: u16) -> Self {
        let size = cols as usize * rows as usize;
        let cells = vec![
            Cell {
                char_codepoint: 0,
                fg: DEFAULT_FG,
                bg: DEFAULT_BG,
                flags: 0,
                _padding: [0; 3],
            };
            size
        ];
        Self { cells, cols, rows }
    }

    /// Resize the grid, preserving content where possible
    pub fn resize(&mut self, new_cols: u16, new_rows: u16) {
        let new_size = new_cols as usize * new_rows as usize;
        let mut new_cells = vec![
            Cell {
                char_codepoint: 0,
                fg: DEFAULT_FG,
                bg: DEFAULT_BG,
                flags: 0,
                _padding: [0; 3],
            };
            new_size
        ];

        // Copy existing content
        let copy_cols = self.cols.min(new_cols) as usize;
        let copy_rows = self.rows.min(new_rows) as usize;

        for y in 0..copy_rows {
            for x in 0..copy_cols {
                let old_idx = y * self.cols as usize + x;
                let new_idx = y * new_cols as usize + x;
                if old_idx < self.cells.len() && new_idx < new_cells.len() {
                    new_cells[new_idx] = self.cells[old_idx];
                }
            }
        }

        self.cells = new_cells;
        self.cols = new_cols;
        self.rows = new_rows;
    }

    /// Clear the entire grid
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell {
                char_codepoint: 0,
                fg: DEFAULT_FG,
                bg: DEFAULT_BG,
                flags: 0,
                _padding: [0; 3],
            };
        }
    }

    /// Get a cell at the given position (returns None if out of bounds)
    #[inline]
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        if x < self.cols && y < self.rows {
            let idx = y as usize * self.cols as usize + x as usize;
            self.cells.get(idx)
        } else {
            None
        }
    }

    /// Get a mutable cell at the given position (returns None if out of bounds)
    #[inline]
    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        if x < self.cols && y < self.rows {
            let idx = y as usize * self.cols as usize + x as usize;
            self.cells.get_mut(idx)
        } else {
            None
        }
    }
}

/// Terminal state manager that implements the VTE Perform trait
///
/// Each TerminalState owns its own Grid for off-screen rendering,
/// enabling multiplexing support where multiple panes can have
/// independent terminal state.
pub struct TerminalState {
    /// Local grid buffer (owned, not shared memory)
    pub grid: Grid,
    /// VTE parser instance
    parser: Parser,
    /// Current cursor position (0-indexed)
    pub cursor_x: u16,
    pub cursor_y: u16,
    /// Current terminal dimensions
    cols: u16,
    rows: u16,
    /// Current text attributes
    attrs: TextAttributes,
    /// Scrollback buffer (stores lines that scrolled off the top)
    scrollback: VecDeque<Vec<Cell>>,
    /// Saved cursor position (for DECSC/DECRC)
    saved_cursor: (u16, u16),
    saved_attrs: TextAttributes,
    /// Image placement state for inline images
    pub image_state: ImagePlacementState,
    /// Shell integration markers (OSC 133)
    pub prompt_markers: Vec<PromptMarker>,
    /// Maximum markers to retain
    pub max_markers: usize,
    /// Maximum images per pane (for eviction)
    pub max_images: usize,
    /// DCS sequence buffer for Sixel graphics
    dcs_buffer: Vec<u8>,
    /// Whether we're currently in a DCS sequence
    in_dcs: bool,
    /// Semantic zone tracker for deep shell integration
    pub zone_tracker: ZoneTracker,
}

impl TerminalState {
    /// Create a new terminal state manager with local grid
    ///
    /// The grid is stored locally and can be blitted to shared memory
    /// when this pane becomes active.
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            grid: Grid::new(cols, rows),
            parser: Parser::new(),
            cursor_x: 0,
            cursor_y: 0,
            cols,
            rows,
            attrs: TextAttributes::default(),
            scrollback: VecDeque::with_capacity(SCROLLBACK_SIZE),
            saved_cursor: (0, 0),
            saved_attrs: TextAttributes::default(),
            image_state: ImagePlacementState::new(),
            prompt_markers: Vec::new(),
            max_markers: 1000, // Keep last 1000 markers
            max_images: MAX_IMAGES_PER_PANE,
            dcs_buffer: Vec::new(),
            in_dcs: false,
            zone_tracker: ZoneTracker::new(500), // Keep last 500 command blocks
        }
    }

    /// Create with legacy SharedState pointer (for backwards compatibility during migration)
    #[deprecated(note = "Use new(cols, rows) instead - this is for migration only")]
    pub fn new_legacy(shared_ptr: *mut SharedState, sequence_counter: Arc<AtomicU64>) -> Self {
        let state = Self::new(GRID_WIDTH as u16, GRID_HEIGHT as u16);
        // Initialize by blitting to shared memory immediately
        state.blit_to_shm(shared_ptr, &sequence_counter);
        state
    }

    /// Update terminal dimensions
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols.min(GRID_WIDTH as u16);
        self.rows = rows.min(GRID_HEIGHT as u16);
        self.cursor_x = self.cursor_x.min(self.cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(self.rows.saturating_sub(1));
        self.grid.resize(self.cols, self.rows);
    }

    /// Blit (copy) the local grid to shared memory
    ///
    /// This is called when this pane is active to update the client's view.
    /// The sequence counter is incremented to signal to the client that
    /// new data is available.
    pub fn blit_to_shm(&self, shm: *mut SharedState, sequence_counter: &Arc<AtomicU64>) {
        unsafe {
            let state = &mut *shm;

            // Copy cells from local grid to shared memory
            // We need to map from local grid layout to SharedState's fixed GRID_WIDTH layout
            for y in 0..self.rows.min(GRID_HEIGHT as u16) {
                for x in 0..self.cols.min(GRID_WIDTH as u16) {
                    let local_idx = y as usize * self.cols as usize + x as usize;
                    let shm_idx = y as usize * GRID_WIDTH + x as usize;

                    if local_idx < self.grid.cells.len() && shm_idx < state.cells.len() {
                        state.cells[shm_idx] = self.grid.cells[local_idx];
                    }
                }
            }

            // Update cursor position
            state.cursor_x = self.cursor_x;
            state.cursor_y = self.cursor_y;

            // Mark dirty and increment sequence number
            state.dirty_flag = 1;
            let new_seq = sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;
            state.sequence_number = new_seq;
        }
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Calculate the absolute line number in scrollback
    ///
    /// This is used for prompt markers to track their position across scrolling.
    /// Returns: scrollback_lines + current_y
    fn absolute_line(&self) -> usize {
        self.scrollback.len() + self.cursor_y as usize
    }

    /// Get current timestamp in microseconds since UNIX epoch
    fn current_timestamp_micros() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }

    /// Add a shell integration marker at the current cursor position
    pub fn add_prompt_marker(&mut self, marker_type: PromptMarkerType) {
        let marker = PromptMarker {
            marker_type,
            line: self.absolute_line(),
            timestamp: Instant::now(),
        };
        self.prompt_markers.push(marker);

        // Trim old markers if needed
        if self.prompt_markers.len() > self.max_markers {
            self.prompt_markers.remove(0);
        }

        log::debug!(
            "Added prompt marker {:?} at line {} (scrollback: {}, cursor_y: {})",
            marker_type,
            self.absolute_line(),
            self.scrollback.len(),
            self.cursor_y
        );
    }

    /// Find the previous prompt from current position
    pub fn previous_prompt(&self, from_line: usize) -> Option<&PromptMarker> {
        self.prompt_markers
            .iter()
            .rev()
            .find(|m| m.line < from_line && matches!(m.marker_type, PromptMarkerType::PromptStart))
    }

    /// Find the next prompt from current position
    pub fn next_prompt(&self, from_line: usize) -> Option<&PromptMarker> {
        self.prompt_markers
            .iter()
            .find(|m| m.line > from_line && matches!(m.marker_type, PromptMarkerType::PromptStart))
    }

    /// Get all prompt markers
    pub fn prompt_markers(&self) -> &[PromptMarker] {
        &self.prompt_markers
    }

    /// Add an image placement from iTerm2 parser
    ///
    /// Automatically evicts oldest image if at max_images limit.
    pub fn add_image(&mut self, image_data: crate::images::ImageData) {
        // Evict oldest if at limit
        if self.image_state.len() >= self.max_images {
            if let Some(oldest) = self.image_state.placements.first() {
                let oldest_id = oldest.id;
                self.image_state.remove_placement(oldest_id);
                log::debug!(
                    "Evicted oldest image {} (at limit {})",
                    oldest_id,
                    self.max_images
                );
            }
        }

        let id = self
            .image_state
            .add_placement(self.cursor_x, self.cursor_y, image_data);

        log::debug!(
            "Added image placement {} at ({}, {})",
            id,
            self.cursor_x,
            self.cursor_y
        );
    }

    /// Clear all images (called on RIS/full reset)
    pub fn clear_images(&mut self) {
        self.image_state.clear();
    }

    /// Process PTY output through the VTE parser
    ///
    /// Updates the local grid - call blit_to_shm() after processing
    /// to copy the grid to shared memory for the client.
    pub fn process_output(&mut self, data: &[u8]) {
        // Take ownership of the parser temporarily to satisfy borrow checker
        let mut parser = std::mem::replace(&mut self.parser, vte::Parser::new());

        for byte in data {
            parser.advance(self, *byte);
        }

        // Restore the parser
        self.parser = parser;
    }

    /// Write a character at the current cursor position
    fn write_char(&mut self, c: char) {
        if self.cursor_x >= self.cols {
            // Handle line wrapping
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        if self.cursor_y >= self.rows {
            // Scroll up
            self.scroll_up(1);
        }

        // Write to local grid
        if let Some(cell) = self.grid.get_mut(self.cursor_x, self.cursor_y) {
            *cell = Cell {
                char_codepoint: c as u32,
                fg: self.attrs.fg,
                bg: self.attrs.bg,
                flags: self.attrs.flags,
                _padding: [0; 3],
            };
        }

        self.cursor_x += 1;
    }

    /// Scroll the screen up by n lines
    fn scroll_up(&mut self, lines: usize) {
        let cols = self.cols as usize;
        let rows = self.rows as usize;

        // Save scrolled lines to scrollback buffer
        for i in 0..lines {
            if i >= rows {
                break;
            }
            let mut line = Vec::with_capacity(cols);
            for x in 0..cols {
                let idx = i * cols + x;
                if idx < self.grid.cells.len() {
                    line.push(self.grid.cells[idx]);
                }
            }
            self.scrollback.push_back(line);

            // Limit scrollback buffer size
            if self.scrollback.len() > SCROLLBACK_SIZE {
                self.scrollback.pop_front();
            }
        }

        // Shift grid content up
        for y in 0..(rows.saturating_sub(lines)) {
            for x in 0..cols {
                let src_idx = (y + lines) * cols + x;
                let dst_idx = y * cols + x;
                if src_idx < self.grid.cells.len() && dst_idx < self.grid.cells.len() {
                    self.grid.cells[dst_idx] = self.grid.cells[src_idx];
                }
            }
        }

        // Clear the bottom lines
        for y in (rows.saturating_sub(lines))..rows {
            for x in 0..cols {
                let idx = y * cols + x;
                if idx < self.grid.cells.len() {
                    self.grid.cells[idx] = Cell {
                        char_codepoint: 0,
                        fg: DEFAULT_FG,
                        bg: DEFAULT_BG,
                        flags: 0,
                        _padding: [0; 3],
                    };
                }
            }
        }

        // Adjust cursor position
        if self.cursor_y >= lines as u16 {
            self.cursor_y -= lines as u16;
        } else {
            self.cursor_y = 0;
        }

        // Update image positions when scrolling
        self.image_state.scroll(lines as i32);

        // Update zone line numbers when scrolling
        // Lines move into scrollback, so absolute line numbers increase
        self.zone_tracker.adjust_for_scroll(lines as i32);
    }

    /// Clear the screen
    fn clear_screen(&mut self) {
        self.grid.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;

        // Clear image placements when clearing screen
        self.clear_images();
    }

    /// Clear from cursor to end of line
    fn clear_to_eol(&mut self) {
        let cols = self.cols as usize;
        for x in self.cursor_x as usize..cols {
            let idx = self.cursor_y as usize * cols + x;
            if idx < self.grid.cells.len() {
                self.grid.cells[idx] = Cell {
                    char_codepoint: 0,
                    fg: DEFAULT_FG,
                    bg: DEFAULT_BG,
                    flags: 0,
                    _padding: [0; 3],
                };
            }
        }
    }

    /// Set SGR (Select Graphic Rendition) attributes
    fn set_sgr(&mut self, params: &[i64]) {
        if params.is_empty() {
            // Reset all attributes
            self.attrs = TextAttributes::default();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.attrs = TextAttributes::default(),
                1 => self.attrs.flags |= FLAG_BOLD,
                2 => self.attrs.flags |= FLAG_DIM,
                3 => self.attrs.flags |= FLAG_ITALIC,
                4 => self.attrs.flags |= FLAG_UNDERLINE,
                7 => self.attrs.flags |= FLAG_INVERSE,
                22 => self.attrs.flags &= !(FLAG_BOLD | FLAG_DIM),
                23 => self.attrs.flags &= !FLAG_ITALIC,
                24 => self.attrs.flags &= !FLAG_UNDERLINE,
                27 => self.attrs.flags &= !FLAG_INVERSE,

                // Foreground colors (30-37, 90-97)
                30..=37 => self.attrs.fg = ansi_color_to_rgba(params[i] as u8 - 30),
                90..=97 => self.attrs.fg = ansi_bright_color_to_rgba(params[i] as u8 - 90),

                // Background colors (40-47, 100-107)
                40..=47 => self.attrs.bg = ansi_color_to_rgba(params[i] as u8 - 40),
                100..=107 => self.attrs.bg = ansi_bright_color_to_rgba(params[i] as u8 - 100),

                // 256-color mode (38;5;n for fg, 48;5;n for bg)
                38 | 48 => {
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        let color = color_256_to_rgba(params[i + 2] as u8);
                        if params[i] == 38 {
                            self.attrs.fg = color;
                        } else {
                            self.attrs.bg = color;
                        }
                        i += 2;
                    }
                }

                // Default colors
                39 => self.attrs.fg = DEFAULT_FG,
                49 => self.attrs.bg = DEFAULT_BG,

                _ => {} // Ignore unknown codes
            }
            i += 1;
        }
    }

    /// Get current image placements for rendering
    pub fn image_placements(&self) -> &[crate::images::ImagePlacement] {
        &self.image_state.placements
    }
}

impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        self.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            0x09 => {
                // Tab (move to next tab stop, typically 8 spaces)
                self.cursor_x = ((self.cursor_x / 8) + 1) * 8;
                if self.cursor_x >= self.cols {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                    if self.cursor_y >= self.rows {
                        self.scroll_up(1);
                    }
                }
            }
            0x0A => {
                // Line Feed (LF)
                // In most terminal emulators, LF also implies CR (newline mode / LNM)
                // This matches expected shell behavior where \n starts a new line at column 0
                self.cursor_x = 0;
                self.cursor_y += 1;
                if self.cursor_y >= self.rows {
                    self.scroll_up(1);
                }
            }
            0x0D => {
                // Carriage Return
                self.cursor_x = 0;
            }
            _ => {}
        }
    }

    fn hook(&mut self, params: &vte::Params, _intermediates: &[u8], _ignore: bool, action: char) {
        // DCS (Device Control String) hook - starts a DCS sequence
        // For Sixel: ESC P <params> q <sixel_data> ST
        // action will be 'q' for Sixel graphics

        if action == 'q' {
            // This is a Sixel graphics sequence
            self.in_dcs = true;
            self.dcs_buffer.clear();

            // Store the parameters (e.g., "1;1" for aspect ratio)
            for param in params.iter() {
                for &value in param {
                    self.dcs_buffer
                        .extend_from_slice(value.to_string().as_bytes());
                    self.dcs_buffer.push(b';');
                }
            }
            // Remove trailing semicolon if present
            if self.dcs_buffer.last() == Some(&b';') {
                self.dcs_buffer.pop();
            }
            // Add the 'q' marker
            self.dcs_buffer.push(b'q');

            log::debug!(
                "Sixel DCS sequence started with params: {:?}",
                std::str::from_utf8(&self.dcs_buffer).unwrap_or("<invalid>")
            );
        }
    }

    fn put(&mut self, byte: u8) {
        // Accumulate DCS data bytes
        if self.in_dcs {
            self.dcs_buffer.push(byte);
        }
    }

    fn unhook(&mut self) {
        // DCS sequence complete - process the data
        if self.in_dcs {
            self.in_dcs = false;

            log::debug!(
                "Sixel DCS sequence complete, buffer size: {} bytes",
                self.dcs_buffer.len()
            );

            // Try to parse as Sixel
            if let Some(sixel_data) = parse_sixel_dcs(&self.dcs_buffer) {
                // Convert Sixel to ImageData format for placement
                if sixel_data.width > 0 && sixel_data.height > 0 {
                    let image_data = crate::images::ImageData {
                        data: sixel_data.pixels,
                        width: ImageSize::Pixels(sixel_data.width),
                        height: ImageSize::Pixels(sixel_data.height),
                        preserve_aspect_ratio: true,
                        inline: true,
                        do_not_move_cursor: false,
                        filename: None,
                    };

                    log::debug!(
                        "Parsed Sixel image: {}x{} pixels ({} bytes)",
                        sixel_data.width,
                        sixel_data.height,
                        image_data.data.len()
                    );

                    // Add the image using the existing infrastructure
                    self.add_image(image_data);

                    // Move cursor down by the image height (in cells)
                    // Assume ~20 pixels per cell height
                    let height_cells = ((sixel_data.height + 19) / 20) as u16;
                    self.cursor_y = (self.cursor_y + height_cells).min(self.rows - 1);
                    if self.cursor_y >= self.rows - 1 {
                        // If image is tall, may need to scroll
                        let scroll_amount =
                            height_cells.saturating_sub(self.rows - 1 - self.cursor_y);
                        if scroll_amount > 0 {
                            self.scroll_up(scroll_amount as usize);
                        }
                    }
                } else {
                    log::warn!("Sixel parsed but resulted in empty image");
                }
            } else {
                log::warn!("Failed to parse Sixel DCS sequence");
            }

            self.dcs_buffer.clear();
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // Handle OSC sequences
        if params.is_empty() {
            return;
        }

        let first = params[0];

        // Handle OSC 133 - Shell Integration (FinalTerm/VS Code)
        if first == b"133" {
            if let Some(code) = params.get(1) {
                let line = self.absolute_line() as u32;
                let timestamp = Self::current_timestamp_micros();

                match *code {
                    b"A" => {
                        // Prompt start
                        self.add_prompt_marker(PromptMarkerType::PromptStart);
                        self.zone_tracker.mark_prompt_start(line, timestamp);
                    }
                    b"B" => {
                        // Command start / input begins
                        self.add_prompt_marker(PromptMarkerType::CommandStart);
                        self.zone_tracker.mark_command_start(line, timestamp);
                    }
                    b"C" => {
                        // Command executed / output begins
                        self.add_prompt_marker(PromptMarkerType::CommandExecuted);
                        self.zone_tracker.mark_command_executed(line, timestamp);
                    }
                    b"D" => {
                        // Command finished with exit code
                        // Exit code may be in params[2] in format "D;exit_code"
                        let exit_code = if params.len() > 2 {
                            std::str::from_utf8(params[2])
                                .ok()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0)
                        } else {
                            0
                        };
                        self.add_prompt_marker(PromptMarkerType::CommandFinished { exit_code });
                        self.zone_tracker
                            .mark_command_finished(line, exit_code, timestamp);
                    }
                    _ => {
                        log::debug!("Unknown OSC 133 code: {:?}", code);
                    }
                }
            }
            return;
        }

        // Handle OSC 1337 - iTerm2 image protocol
        if first == b"1337" {
            if params.len() < 2 {
                log::debug!("OSC 1337 received but no payload");
                return;
            }

            // Combine remaining params (they were split by ';')
            // We need to reconstruct the full payload
            let mut payload = Vec::new();
            for (i, param) in params[1..].iter().enumerate() {
                if i > 0 {
                    payload.push(b';');
                }
                payload.extend_from_slice(param);
            }

            if let Some(image_data) = parse_iterm2_image(&payload) {
                // Use the new add_image method which handles eviction
                let do_not_move_cursor = image_data.do_not_move_cursor;
                self.add_image(image_data);

                // Move cursor if needed (unless doNotMoveCursor is set)
                if !do_not_move_cursor {
                    // For now, just move to next line
                    // TODO: Calculate actual cursor movement based on image size
                    self.cursor_y += 1;
                    if self.cursor_y >= self.rows {
                        self.scroll_up(1);
                    }
                }
            } else {
                log::warn!("Failed to parse iTerm2 image from OSC 1337");
            }
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let params: Vec<i64> = params.iter().map(|p| p[0] as i64).collect();

        match action {
            'A' => {
                // Cursor Up
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                self.cursor_y = self.cursor_y.saturating_sub(n);
            }
            'B' => {
                // Cursor Down
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                self.cursor_y = (self.cursor_y + n).min(self.rows - 1);
            }
            'C' => {
                // Cursor Forward
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                self.cursor_x = (self.cursor_x + n).min(self.cols - 1);
            }
            'D' => {
                // Cursor Back
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                self.cursor_x = self.cursor_x.saturating_sub(n);
            }
            'H' | 'f' => {
                // Cursor Position
                let row = params.get(0).copied().unwrap_or(1).max(1) as u16 - 1;
                let col = params.get(1).copied().unwrap_or(1).max(1) as u16 - 1;
                self.cursor_y = row.min(self.rows - 1);
                self.cursor_x = col.min(self.cols - 1);
            }
            'J' => {
                // Erase in Display
                let n = params.get(0).copied().unwrap_or(0);
                let cols = self.cols as usize;
                match n {
                    0 => {
                        // Clear from cursor to end of screen
                        self.clear_to_eol();
                        for y in (self.cursor_y as usize + 1)..self.rows as usize {
                            for x in 0..cols {
                                let idx = y * cols + x;
                                if idx < self.grid.cells.len() {
                                    self.grid.cells[idx] = Cell::default();
                                }
                            }
                        }
                    }
                    1 => {
                        // Clear from cursor to beginning of screen
                        for y in 0..self.cursor_y as usize {
                            for x in 0..cols {
                                let idx = y * cols + x;
                                if idx < self.grid.cells.len() {
                                    self.grid.cells[idx] = Cell::default();
                                }
                            }
                        }
                    }
                    2 => {
                        // Clear entire screen (RIS - Reset Initial State)
                        self.clear_screen();
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line
                let n = params.get(0).copied().unwrap_or(0);
                let cols = self.cols as usize;
                match n {
                    0 => self.clear_to_eol(),
                    1 => {
                        // Clear from beginning of line to cursor
                        for x in 0..=self.cursor_x as usize {
                            let idx = self.cursor_y as usize * cols + x;
                            if idx < self.grid.cells.len() {
                                self.grid.cells[idx] = Cell::default();
                            }
                        }
                    }
                    2 => {
                        // Clear entire line
                        for x in 0..cols {
                            let idx = self.cursor_y as usize * cols + x;
                            if idx < self.grid.cells.len() {
                                self.grid.cells[idx] = Cell::default();
                            }
                        }
                    }
                    _ => {}
                }
            }
            'm' => {
                // SGR (Select Graphic Rendition)
                self.set_sgr(&params);
            }
            's' => {
                // Save cursor position (DECSC)
                self.saved_cursor = (self.cursor_x, self.cursor_y);
                self.saved_attrs = self.attrs;
            }
            'u' => {
                // Restore cursor position (DECRC)
                self.cursor_x = self.saved_cursor.0;
                self.cursor_y = self.saved_cursor.1;
                self.attrs = self.saved_attrs;
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

/// Convert ANSI color index (0-7) to RGBA
fn ansi_color_to_rgba(index: u8) -> u32 {
    match index {
        0 => 0xFF000000, // Black
        1 => 0xFFCD0000, // Red
        2 => 0xFF00CD00, // Green
        3 => 0xFFCDCD00, // Yellow
        4 => 0xFF0000EE, // Blue
        5 => 0xFFCD00CD, // Magenta
        6 => 0xFF00CDCD, // Cyan
        7 => 0xFFE5E5E5, // White
        _ => DEFAULT_FG,
    }
}

/// Convert ANSI bright color index (0-7) to RGBA
fn ansi_bright_color_to_rgba(index: u8) -> u32 {
    match index {
        0 => 0xFF7F7F7F, // Bright Black (Gray)
        1 => 0xFFFF0000, // Bright Red
        2 => 0xFF00FF00, // Bright Green
        3 => 0xFFFFFF00, // Bright Yellow
        4 => 0xFF5C5CFF, // Bright Blue
        5 => 0xFFFF00FF, // Bright Magenta
        6 => 0xFF00FFFF, // Bright Cyan
        7 => 0xFFFFFFFF, // Bright White
        _ => DEFAULT_FG,
    }
}

/// Convert 256-color palette index to RGBA
fn color_256_to_rgba(index: u8) -> u32 {
    match index {
        // 0-15: Standard ANSI colors
        0..=7 => ansi_color_to_rgba(index),
        8..=15 => ansi_bright_color_to_rgba(index - 8),

        // 16-231: 6x6x6 color cube
        16..=231 => {
            let idx = index - 16;
            let r = (idx / 36) * 51;
            let g = ((idx % 36) / 6) * 51;
            let b = (idx % 6) * 51;
            0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }

        // 232-255: Grayscale
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            0xFF000000 | ((gray as u32) << 16) | ((gray as u32) << 8) | (gray as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_color_conversion() {
        assert_eq!(ansi_color_to_rgba(0), 0xFF000000); // Black
        assert_eq!(ansi_color_to_rgba(1), 0xFFCD0000); // Red
        assert_eq!(ansi_color_to_rgba(7), 0xFFE5E5E5); // White
    }

    #[test]
    fn test_256_color_conversion() {
        // Test standard colors
        assert_eq!(color_256_to_rgba(0), ansi_color_to_rgba(0));

        // Test grayscale
        let gray = color_256_to_rgba(232);
        assert!(gray != 0);
    }

    #[test]
    fn test_osc_133_parsing() {
        let mut state = TerminalState::new(80, 24);

        // Simulate OSC 133;A (Prompt Start)
        state.osc_dispatch(&[b"133", b"A"], true);
        assert_eq!(state.prompt_markers.len(), 1);
        assert!(matches!(
            state.prompt_markers[0].marker_type,
            PromptMarkerType::PromptStart
        ));

        // Simulate OSC 133;B (Command Start)
        state.osc_dispatch(&[b"133", b"B"], true);
        assert_eq!(state.prompt_markers.len(), 2);
        assert!(matches!(
            state.prompt_markers[1].marker_type,
            PromptMarkerType::CommandStart
        ));

        // Simulate OSC 133;C (Command Executed)
        state.osc_dispatch(&[b"133", b"C"], true);
        assert_eq!(state.prompt_markers.len(), 3);
        assert!(matches!(
            state.prompt_markers[2].marker_type,
            PromptMarkerType::CommandExecuted
        ));

        // Simulate OSC 133;D;0 (Command Finished with exit code 0)
        state.osc_dispatch(&[b"133", b"D", b"0"], true);
        assert_eq!(state.prompt_markers.len(), 4);
        if let PromptMarkerType::CommandFinished { exit_code } = state.prompt_markers[3].marker_type
        {
            assert_eq!(exit_code, 0);
        } else {
            panic!("Expected CommandFinished marker");
        }

        // Simulate OSC 133;D;127 (Command Finished with exit code 127)
        state.osc_dispatch(&[b"133", b"D", b"127"], true);
        assert_eq!(state.prompt_markers.len(), 5);
        if let PromptMarkerType::CommandFinished { exit_code } = state.prompt_markers[4].marker_type
        {
            assert_eq!(exit_code, 127);
        } else {
            panic!("Expected CommandFinished marker");
        }
    }

    #[test]
    fn test_prompt_navigation() {
        let mut state = TerminalState::new(80, 24);

        // Add several prompt markers at different lines
        state.cursor_y = 0;
        state.add_prompt_marker(PromptMarkerType::PromptStart);

        state.cursor_y = 5;
        state.add_prompt_marker(PromptMarkerType::PromptStart);

        state.cursor_y = 10;
        state.add_prompt_marker(PromptMarkerType::PromptStart);

        // Test previous_prompt
        let prev = state.previous_prompt(7);
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().line, 5);

        // Test next_prompt
        let next = state.next_prompt(7);
        assert!(next.is_some());
        assert_eq!(next.unwrap().line, 10);
    }

    #[test]
    fn test_marker_limit() {
        let mut state = TerminalState::new(80, 24);
        state.max_markers = 10; // Set a low limit for testing

        // Add more markers than the limit
        for i in 0..15 {
            state.cursor_y = i as u16;
            state.add_prompt_marker(PromptMarkerType::PromptStart);
        }

        // Should have exactly max_markers
        assert_eq!(state.prompt_markers.len(), 10);

        // First marker should be the 6th one added (0-indexed 5th)
        assert_eq!(state.prompt_markers[0].line, 5);
    }

    #[test]
    fn test_image_eviction() {
        let mut state = TerminalState::new(80, 24);
        state.max_images = 3; // Set low limit for testing

        // Create minimal test image data
        let make_image = || crate::images::ImageData {
            data: vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
                0x00, 0x00, 0x00, 0x0D, // IHDR length (13)
                0x49, 0x48, 0x44, 0x52, // "IHDR"
                0x00, 0x00, 0x00, 0x01, // Width = 1
                0x00, 0x00, 0x00, 0x01, // Height = 1
            ],
            width: crate::images::ImageSize::Auto,
            height: crate::images::ImageSize::Auto,
            preserve_aspect_ratio: true,
            inline: true,
            do_not_move_cursor: true,
            filename: None,
        };

        // Add 4 images - should evict first one
        state.add_image(make_image());
        state.add_image(make_image());
        state.add_image(make_image());
        assert_eq!(state.image_state.len(), 3);

        state.add_image(make_image());
        assert_eq!(state.image_state.len(), 3); // Still at limit

        // First image should have been evicted (ID 1)
        // Remaining should be IDs 2, 3, 4
        assert_eq!(state.image_state.placements[0].id, 2);
        assert_eq!(state.image_state.placements[1].id, 3);
        assert_eq!(state.image_state.placements[2].id, 4);
    }

    #[test]
    fn test_clear_images() {
        let mut state = TerminalState::new(80, 24);

        let make_image = || crate::images::ImageData {
            data: vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
                0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            ],
            width: crate::images::ImageSize::Auto,
            height: crate::images::ImageSize::Auto,
            preserve_aspect_ratio: true,
            inline: true,
            do_not_move_cursor: true,
            filename: None,
        };

        state.add_image(make_image());
        state.add_image(make_image());
        assert_eq!(state.image_state.len(), 2);

        state.clear_images();
        assert_eq!(state.image_state.len(), 0);
    }
}
