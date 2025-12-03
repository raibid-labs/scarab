use scarab_protocol::{Cell, SharedState, GRID_HEIGHT, GRID_WIDTH};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use super::settings::ExportFormat;

/// Export terminal grid to various formats for accessibility
pub struct TerminalExporter;

impl TerminalExporter {
    /// Export grid to specified format
    pub fn export(
        grid: &[Cell],
        width: usize,
        height: usize,
        format: ExportFormat,
        path: &Path,
    ) -> io::Result<()> {
        let content = match format {
            ExportFormat::PlainText => Self::export_to_text(grid, width, height),
            ExportFormat::Html => Self::export_to_html(grid, width, height),
            ExportFormat::Markdown => Self::export_to_markdown(grid, width, height),
        };

        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Export from SharedState directly
    pub fn export_from_shared_state(
        state: &SharedState,
        format: ExportFormat,
        path: &Path,
    ) -> io::Result<()> {
        Self::export(&state.cells, GRID_WIDTH, GRID_HEIGHT, format, path)
    }

    /// Export to plain text format (ANSI stripped)
    pub fn export_to_text(grid: &[Cell], width: usize, height: usize) -> String {
        let mut output = String::new();

        for row in 0..height {
            let mut line = String::new();
            let mut has_content = false;

            for col in 0..width {
                let idx = row * width + col;
                if idx >= grid.len() {
                    break;
                }

                let cell = &grid[idx];
                let ch = char::from_u32(cell.char_codepoint).unwrap_or(' ');

                // Track if we have any non-space content
                if ch != ' ' {
                    has_content = true;
                }

                line.push(ch);
            }

            // Trim trailing spaces from each line
            let line = line.trim_end();

            // Skip completely empty lines at the end
            if has_content || !output.is_empty() {
                output.push_str(line);
                output.push('\n');
            }
        }

        // Remove trailing empty lines
        output.trim_end().to_string() + "\n"
    }

    /// Export to HTML with CSS color preservation
    pub fn export_to_html(grid: &[Cell], width: usize, height: usize) -> String {
        let mut output = String::new();

        // HTML header with styling
        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html>\n<head>\n");
        output.push_str("  <meta charset=\"UTF-8\">\n");
        output.push_str("  <title>Terminal Export</title>\n");
        output.push_str("  <style>\n");
        output.push_str("    body {\n");
        output.push_str("      background-color: #000;\n");
        output.push_str("      color: #fff;\n");
        output.push_str("      font-family: 'Courier New', monospace;\n");
        output.push_str("      font-size: 14px;\n");
        output.push_str("      line-height: 1.2;\n");
        output.push_str("      padding: 20px;\n");
        output.push_str("    }\n");
        output.push_str("    .terminal {\n");
        output.push_str("      white-space: pre;\n");
        output.push_str("      display: inline-block;\n");
        output.push_str("    }\n");
        output.push_str("    .terminal-line {\n");
        output.push_str("      display: block;\n");
        output.push_str("    }\n");
        output.push_str("    .cell {\n");
        output.push_str("      display: inline;\n");
        output.push_str("    }\n");
        output.push_str("    .bold { font-weight: bold; }\n");
        output.push_str("    .italic { font-style: italic; }\n");
        output.push_str("  </style>\n");
        output.push_str("</head>\n<body>\n");
        output.push_str("<div class=\"terminal\">\n");

        // Export grid with color information
        for row in 0..height {
            output.push_str("<span class=\"terminal-line\">");

            for col in 0..width {
                let idx = row * width + col;
                if idx >= grid.len() {
                    break;
                }

                let cell = &grid[idx];
                let ch = char::from_u32(cell.char_codepoint).unwrap_or(' ');

                // Extract RGBA components
                let fg = cell.fg;
                let bg = cell.bg;

                let fg_r = ((fg >> 24) & 0xFF) as u8;
                let fg_g = ((fg >> 16) & 0xFF) as u8;
                let fg_b = ((fg >> 8) & 0xFF) as u8;

                let bg_r = ((bg >> 24) & 0xFF) as u8;
                let bg_g = ((bg >> 16) & 0xFF) as u8;
                let bg_b = ((bg >> 8) & 0xFF) as u8;

                // Build style classes
                let mut classes = vec!["cell"];
                if cell.flags & 0x01 != 0 {
                    classes.push("bold");
                }
                if cell.flags & 0x02 != 0 {
                    classes.push("italic");
                }

                // Only add span if we have non-default colors or styles
                let has_custom_fg = fg != 0xFFFFFFFF;
                let has_custom_bg = bg != 0x000000FF;
                let has_styles = classes.len() > 1;

                if has_custom_fg || has_custom_bg || has_styles {
                    output.push_str(&format!("<span class=\"{}\" style=\"", classes.join(" ")));

                    if has_custom_fg {
                        output.push_str(&format!("color: rgb({}, {}, {});", fg_r, fg_g, fg_b));
                    }
                    if has_custom_bg {
                        output.push_str(&format!(
                            "background-color: rgb({}, {}, {});",
                            bg_r, bg_g, bg_b
                        ));
                    }

                    output.push_str("\">");
                }

                // Escape HTML entities
                match ch {
                    '<' => output.push_str("&lt;"),
                    '>' => output.push_str("&gt;"),
                    '&' => output.push_str("&amp;"),
                    ' ' => output.push_str("&nbsp;"),
                    _ => output.push(ch),
                }

                if has_custom_fg || has_custom_bg || has_styles {
                    output.push_str("</span>");
                }
            }

            output.push_str("</span>\n");
        }

        output.push_str("</div>\n");
        output.push_str("</body>\n</html>\n");

        output
    }

    /// Export to Markdown code block format
    pub fn export_to_markdown(grid: &[Cell], width: usize, height: usize) -> String {
        let mut output = String::new();

        // Markdown header
        output.push_str("# Terminal Export\n\n");
        output.push_str("```\n");

        // Export as plain text within code block
        let text = Self::export_to_text(grid, width, height);
        output.push_str(&text);

        output.push_str("```\n");

        output
    }

    /// Extract visible text from grid (for screen reader announcements)
    pub fn extract_visible_text(grid: &[Cell], width: usize, height: usize) -> String {
        Self::export_to_text(grid, width, height)
    }

    /// Extract text from a specific region
    pub fn extract_region(
        grid: &[Cell],
        width: usize,
        x: usize,
        y: usize,
        region_width: usize,
        region_height: usize,
    ) -> String {
        let mut output = String::new();

        for row in y..(y + region_height) {
            let mut line = String::new();

            for col in x..(x + region_width) {
                if col >= width {
                    break;
                }

                let idx = row * width + col;
                if idx >= grid.len() {
                    break;
                }

                let cell = &grid[idx];
                let ch = char::from_u32(cell.char_codepoint).unwrap_or(' ');
                line.push(ch);
            }

            output.push_str(line.trim_end());
            output.push('\n');
        }

        output.trim_end().to_string()
    }

    /// Extract text from current line (for cursor position announcements)
    pub fn extract_line(grid: &[Cell], width: usize, line: usize) -> String {
        let mut output = String::new();
        let start_idx = line * width;

        for col in 0..width {
            let idx = start_idx + col;
            if idx >= grid.len() {
                break;
            }

            let cell = &grid[idx];
            let ch = char::from_u32(cell.char_codepoint).unwrap_or(' ');
            output.push(ch);
        }

        output.trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scarab_protocol::Cell;

    fn create_test_grid() -> Vec<Cell> {
        let mut grid = vec![Cell::default(); 80 * 24];

        // Add some test content
        let test_text = b"Hello, World!";
        for (i, &byte) in test_text.iter().enumerate() {
            grid[i].char_codepoint = byte as u32;
        }

        grid
    }

    #[test]
    fn test_export_to_text() {
        let grid = create_test_grid();
        let text = TerminalExporter::export_to_text(&grid, 80, 24);
        assert!(text.contains("Hello, World!"));
    }

    #[test]
    fn test_export_to_html() {
        let grid = create_test_grid();
        let html = TerminalExporter::export_to_html(&grid, 80, 24);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Hello, World!"));
    }

    #[test]
    fn test_export_to_markdown() {
        let grid = create_test_grid();
        let md = TerminalExporter::export_to_markdown(&grid, 80, 24);
        assert!(md.contains("```"));
        assert!(md.contains("Hello, World!"));
    }

    #[test]
    fn test_extract_region() {
        let grid = create_test_grid();
        let region = TerminalExporter::extract_region(&grid, 80, 0, 0, 13, 1);
        assert_eq!(region.trim(), "Hello, World!");
    }

    #[test]
    fn test_extract_line() {
        let grid = create_test_grid();
        let line = TerminalExporter::extract_line(&grid, 80, 0);
        assert_eq!(line.trim(), "Hello, World!");
    }
}
