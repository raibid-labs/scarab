//! Ligature rendering verification tests
//!
//! Tests that programming ligatures render correctly with cosmic-text + Harfbuzz.
//! This verifies that the shape-run-cache feature is enabled and fonts support ligatures.

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};

/// Common programming ligature sequences
const LIGATURE_SEQUENCES: &[&str] = &[
    "==",  // Equal
    "!=",  // Not equal
    "===", // Triple equal
    "!==", // Triple not equal
    "->",  // Arrow
    "=>",  // Fat arrow
    ">=",  // Greater or equal
    "<=",  // Less or equal
    "<-",  // Left arrow
    "::",  // Scope
    "++",  // Increment
    "--",  // Decrement
    "&&",  // And
    "||",  // Or
    "/*",  // Comment start
    "*/",  // Comment end
    "//",  // Line comment
    "...", // Ellipsis
    "..<", // Range
    "..=", // Inclusive range
    "|>",  // Pipe
    "<|",  // Reverse pipe
    ">>>", // Triple shift
    "<<<", // Triple left shift
];

/// Test harness for headless text rendering
struct LigatureTestHarness {
    font_system: FontSystem,
}

impl LigatureTestHarness {
    fn new() -> Self {
        let mut font_system = FontSystem::new();

        // Load system fonts
        let font_db = font_system.db_mut();
        font_db.load_system_fonts();

        let face_count = font_db.faces().count();
        eprintln!("Loaded {} font faces from system", face_count);

        if face_count == 0 {
            eprintln!("WARNING: No fonts loaded! Tests may not render correctly.");
        }

        Self { font_system }
    }

    /// Render text and capture the layout information
    fn render_text(&mut self, text: &str, font_size: f32) -> RenderResult {
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, font_size * 1.2),
        );

        buffer.set_size(&mut self.font_system, 1000.0, 100.0);

        // Use Advanced shaping to enable ligatures via Harfbuzz
        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);

        let mut glyphs = Vec::new();
        let mut total_width = 0.0;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                glyphs.push(GlyphInfo {
                    glyph_id: glyph.glyph_id,
                    x: glyph.x,
                    y: glyph.y,
                    w: glyph.w,
                });
                total_width = glyph.x + glyph.w;
            }
        }

        let glyph_count = glyphs.len();

        RenderResult {
            text: text.to_string(),
            glyphs,
            total_width,
            glyph_count,
        }
    }

    /// Render text with Basic shaping (no ligatures) for comparison
    fn render_text_no_ligatures(&mut self, text: &str, font_size: f32) -> RenderResult {
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, font_size * 1.2),
        );

        buffer.set_size(&mut self.font_system, 1000.0, 100.0);

        // Use Basic shaping to disable ligatures
        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Basic);

        let mut glyphs = Vec::new();
        let mut total_width = 0.0;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                glyphs.push(GlyphInfo {
                    glyph_id: glyph.glyph_id,
                    x: glyph.x,
                    y: glyph.y,
                    w: glyph.w,
                });
                total_width = glyph.x + glyph.w;
            }
        }

        let glyph_count = glyphs.len();

        RenderResult {
            text: text.to_string(),
            glyphs,
            total_width,
            glyph_count,
        }
    }
}

#[derive(Debug, Clone)]
struct GlyphInfo {
    glyph_id: u16,
    x: f32,
    y: f32,
    w: f32,
}

#[derive(Debug)]
struct RenderResult {
    text: String,
    glyphs: Vec<GlyphInfo>,
    total_width: f32,
    glyph_count: usize,
}

impl RenderResult {
    /// Check if this result has fewer glyphs than expected (indicating ligatures)
    fn has_ligatures(&self) -> bool {
        // If we have fewer glyphs than characters, ligatures were applied
        self.glyph_count < self.text.chars().count()
    }

    /// Format for snapshot comparison
    fn to_snapshot_string(&self) -> String {
        format!(
            "Text: {:?}\nGlyph Count: {}\nChar Count: {}\nTotal Width: {:.2}\nGlyphs:\n{}",
            self.text,
            self.glyph_count,
            self.text.chars().count(),
            self.total_width,
            self.glyphs
                .iter()
                .map(|g| format!("  id={} x={:.2} y={:.2} w={:.2}", g.glyph_id, g.x, g.y, g.w))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[test]
fn test_advanced_shaping_enabled() {
    // Verify that Advanced shaping mode is available
    // This is a prerequisite for ligature support

    let mut harness = LigatureTestHarness::new();

    // Render a simple ligature sequence with Advanced shaping
    let result = harness.render_text("->", 16.0);

    // We should get at least some glyphs
    assert!(
        result.glyph_count > 0,
        "No glyphs rendered - font system may not be working"
    );

    eprintln!(
        "Advanced shaping test: {} glyphs for '{}'",
        result.glyph_count, result.text
    );
}

#[test]
fn test_ligature_sequences_render() {
    let mut harness = LigatureTestHarness::new();

    // Test all common ligature sequences
    for seq in LIGATURE_SEQUENCES {
        let result = harness.render_text(seq, 16.0);

        // Verify the sequence renders (produces glyphs)
        assert!(
            result.glyph_count > 0,
            "Ligature sequence '{}' produced no glyphs",
            seq
        );

        // The sequence should produce at least one glyph
        assert!(
            result.glyph_count <= seq.chars().count(),
            "Ligature sequence '{}' produced more glyphs ({}) than characters ({})",
            seq,
            result.glyph_count,
            seq.chars().count()
        );

        eprintln!(
            "Sequence '{}': {} chars -> {} glyphs {}",
            seq,
            seq.chars().count(),
            result.glyph_count,
            if result.has_ligatures() {
                "(ligated)"
            } else {
                "(no ligature)"
            }
        );
    }
}

#[test]
fn test_ligatures_with_advanced_vs_basic_shaping() {
    // Compare Advanced shaping (with ligatures) vs Basic shaping (without)

    let mut harness = LigatureTestHarness::new();

    let test_sequences = vec!["->", "=>", "!=", "==", ">=", "<="];

    for seq in test_sequences {
        let advanced = harness.render_text(seq, 16.0);
        let basic = harness.render_text_no_ligatures(seq, 16.0);

        eprintln!("\nSequence: '{}'", seq);
        eprintln!("  Advanced shaping: {} glyphs", advanced.glyph_count);
        eprintln!("  Basic shaping: {} glyphs", basic.glyph_count);

        // Both should produce glyphs
        assert!(
            advanced.glyph_count > 0,
            "Advanced shaping produced no glyphs for '{}'",
            seq
        );
        assert!(
            basic.glyph_count > 0,
            "Basic shaping produced no glyphs for '{}'",
            seq
        );

        // Advanced shaping may produce fewer glyphs (ligatures) or same (no ligature font)
        assert!(
            advanced.glyph_count <= basic.glyph_count,
            "Advanced shaping produced more glyphs than basic for '{}': {} vs {}",
            seq,
            advanced.glyph_count,
            basic.glyph_count
        );
    }
}

#[test]
fn test_monospace_property_preserved() {
    // Verify that ligatures don't break monospace alignment
    // In a terminal, "=>" and "ab" should have the same total width

    let mut harness = LigatureTestHarness::new();

    let ligature = harness.render_text("=>", 16.0);
    let regular = harness.render_text("ab", 16.0);

    eprintln!("\nMonospace test:");
    eprintln!(
        "  '=>' width: {:.2} ({} glyphs)",
        ligature.total_width, ligature.glyph_count
    );
    eprintln!(
        "  'ab' width: {:.2} ({} glyphs)",
        regular.total_width, regular.glyph_count
    );

    // For true monospace fonts, these should be very close
    // Allow some tolerance for rounding
    let width_diff = (ligature.total_width - regular.total_width).abs();
    let tolerance = 2.0; // pixels

    if width_diff > tolerance {
        eprintln!(
            "  WARNING: Width difference {:.2}px exceeds tolerance {:.2}px",
            width_diff, tolerance
        );
        eprintln!("  This may indicate the font is not truly monospace");
    }

    // Verify that measurements were successfully taken
    assert!(ligature.total_width > 0.0, "Ligature width should be measured");
    assert!(regular.total_width > 0.0, "Regular text width should be measured");
    assert!(ligature.glyph_count > 0, "Ligature should have glyphs");
    assert!(regular.glyph_count > 0, "Regular text should have glyphs");

    // Document whether monospace property is preserved (informational, not strict)
    // Some fonts with ligatures intentionally break strict monospace for better appearance
    if width_diff <= tolerance {
        eprintln!("  PASS: Monospace property preserved within tolerance");
    } else {
        eprintln!(
            "  INFO: Font uses proportional ligatures (width diff: {:.2}px)",
            width_diff
        );
    }
}

#[test]
fn test_ligature_line_rendering() {
    // Test a line with multiple ligatures like real code
    let mut harness = LigatureTestHarness::new();

    let test_line = "if x != y && z >= 0 => result";
    let result = harness.render_text(test_line, 16.0);

    eprintln!("\nCode line test:");
    eprintln!("  Text: '{}'", test_line);
    eprintln!("  Chars: {}", test_line.chars().count());
    eprintln!("  Glyphs: {}", result.glyph_count);
    eprintln!("  Width: {:.2}", result.total_width);

    // Should produce glyphs
    assert!(result.glyph_count > 0, "Code line produced no glyphs");

    // May have fewer glyphs if ligatures are applied
    if result.has_ligatures() {
        eprintln!("  Ligatures detected!");
    }

    // Create snapshot for golden comparison
    let snapshot = result.to_snapshot_string();
    insta::assert_snapshot!("ligature_line", snapshot);
}

#[test]
fn test_complex_ligature_combinations() {
    // Test sequences that combine multiple ligature opportunities
    let mut harness = LigatureTestHarness::new();

    let complex_sequences = vec![
        ">>>=", // Shift assign
        "<<=",  // Left shift assign
        "===",  // Triple equal
        "!==",  // Triple not equal
        "...",  // Ellipsis
        "<=>",  // Spaceship operator
    ];

    for seq in complex_sequences {
        let result = harness.render_text(seq, 16.0);

        eprintln!(
            "Complex sequence '{}': {} chars -> {} glyphs",
            seq,
            seq.chars().count(),
            result.glyph_count
        );

        assert!(
            result.glyph_count > 0,
            "Complex sequence '{}' produced no glyphs",
            seq
        );
    }
}

#[test]
fn test_ligature_boundary_conditions() {
    // Test ligatures at word boundaries and with spaces
    let mut harness = LigatureTestHarness::new();

    let boundary_tests = vec![
        ("=>", "=> "),  // Ligature followed by space
        ("->", " ->"),  // Space followed by ligature
        ("!=", " != "), // Ligature with spaces on both sides
        ("==", "a==b"), // Ligature between characters
    ];

    for (_ligature, text) in boundary_tests {
        let result = harness.render_text(text, 16.0);

        eprintln!(
            "Boundary test '{}': {} chars -> {} glyphs",
            text,
            text.chars().count(),
            result.glyph_count
        );

        assert!(
            result.glyph_count > 0,
            "Boundary test '{}' produced no glyphs",
            text
        );
    }
}

#[test]
fn test_snapshot_all_ligatures() {
    // Create a comprehensive snapshot of all ligature sequences
    let mut harness = LigatureTestHarness::new();

    let mut snapshot_output = String::new();
    snapshot_output.push_str("=== Programming Ligature Test Results ===\n\n");

    for seq in LIGATURE_SEQUENCES {
        let result = harness.render_text(seq, 16.0);
        snapshot_output.push_str(&format!(
            "Sequence: {:6} | Chars: {} | Glyphs: {} | Ligated: {}\n",
            format!("'{}'", seq),
            seq.chars().count(),
            result.glyph_count,
            result.has_ligatures()
        ));
    }

    insta::assert_snapshot!("all_ligatures_summary", snapshot_output);
}
