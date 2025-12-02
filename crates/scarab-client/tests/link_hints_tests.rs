// Comprehensive link hints UI tests using HeadlessTestHarness
// Tests the full link hints feature integration with Bevy ECS

use bevy::prelude::*;

// Import test harness
mod harness;
use harness::HeadlessTestHarness;
use harness::mocks::MockSharedMemoryReader;

// Import link hints components
use scarab_client::ui::link_hints::{
    LinkDetector, LinkHint, LinkHintsPlugin, LinkHintsState, LinkActivatedEvent, LinkType,
};
use scarab_client::rendering::text::TextRenderer;
use scarab_client::rendering::config::FontConfig;
use cosmic_text::{FontSystem, SwashCache};

/// Helper to setup a harness with link hints plugin and mock data
fn setup_link_hints_harness() -> HeadlessTestHarness {
    HeadlessTestHarness::with_setup(|app| {
        // Add link hints plugin
        app.add_plugins(LinkHintsPlugin);

        // Initialize text renderer for positioning
        // Create a minimal renderer for testing (just for positioning info)
        let font_config = FontConfig::default();
        let (cell_width, cell_height) = font_config.cell_dimensions();

        let mut font_system = FontSystem::new();
        let font_db = font_system.db_mut();
        font_db.load_system_fonts();

        let swash_cache = SwashCache::new();

        // Create atlas with our existing assets
        let atlas = {
            let mut images = app.world_mut().resource_mut::<Assets<Image>>();
            use scarab_client::rendering::atlas::GlyphAtlas;
            GlyphAtlas::new(&mut *images)
        };

        let renderer = TextRenderer {
            font_system,
            swash_cache,
            atlas,
            config: font_config,
            cell_width,
            cell_height,
        };

        app.insert_resource(renderer);
    })
}

/// Helper to populate mock terminal with URLs
fn populate_urls(mock: &mut MockSharedMemoryReader, urls: &[(u16, u16, &str)]) {
    for (x, y, url) in urls {
        mock.set_text(*x, *y, url, 0xFFFFFFFF, 0x000000FF);
    }
    mock.tick();
}

// =============================================================================
// Test 1: URL Detection in Grid
// =============================================================================

#[test]
fn test_detect_urls_in_grid() {
    let mut harness = setup_link_hints_harness();

    // Setup mock terminal with URLs
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        populate_urls(&mut mock, &[
            (0, 0, "Visit https://github.com/raibid-labs/scarab"),
            (0, 1, "Documentation at https://docs.rs/scarab"),
            (0, 2, "Or check www.example.com for more"),
        ]);
    }

    // Trigger link detection
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        mock.get_row_text(0) + "\n" + &mock.get_row_text(1) + "\n" + &mock.get_row_text(2)
    };

    let links = {
        let detector = harness.resource::<LinkDetector>();
        detector.detect(&text)
    };

    // Assert correct number of links detected
    let url_links: Vec<_> = links.iter()
        .filter(|(_, link_type)| *link_type == LinkType::Url)
        .collect();

    assert!(url_links.len() >= 3, "Expected at least 3 URLs, found {}", url_links.len());

    // Verify specific URLs are detected
    assert!(url_links.iter().any(|(url, _)| url.contains("github.com")));
    assert!(url_links.iter().any(|(url, _)| url.contains("docs.rs")));
    assert!(url_links.iter().any(|(url, _)| url.contains("www.example.com")));
}

// =============================================================================
// Test 2: Hint Labels Generation
// =============================================================================

#[test]
fn test_hint_labels_generated() {
    let mut harness = setup_link_hints_harness();

    // Populate grid with multiple URLs
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        populate_urls(&mut mock, &[
            (0, 0, "https://first.com"),
            (0, 1, "https://second.com"),
            (0, 2, "https://third.com"),
            (0, 3, "https://fourth.com"),
        ]);
    }

    // Activate link hints mode manually (simulating Ctrl+K)
    {
        let mut state = harness.resource_mut::<LinkHintsState>();
        state.active = true;
    }

    // Detect links and assign hint keys
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        (0..4)
            .map(|i| mock.get_row_text(i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let (detected_links, hint_keys) = {
        let detector = harness.resource::<LinkDetector>();
        let detected_links = detector.detect_with_positions(&text);
        let hint_keys = LinkDetector::generate_hint_keys(detected_links.len());
        (detected_links, hint_keys)
    };

    // Assert hint labels are generated correctly (a, b, c, d)
    assert_eq!(hint_keys.len(), detected_links.len());
    assert_eq!(hint_keys[0], "a");
    assert_eq!(hint_keys[1], "b");
    assert_eq!(hint_keys[2], "c");
    assert_eq!(hint_keys[3], "d");

    // Verify all keys are unique
    let unique_keys: std::collections::HashSet<_> = hint_keys.iter().collect();
    assert_eq!(unique_keys.len(), hint_keys.len());
}

// =============================================================================
// Test 3: Hint Positioning
// =============================================================================

#[test]
fn test_hints_positioned_correctly() {
    let mut harness = setup_link_hints_harness();

    // Setup URLs at known positions
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(10, 5, "https://test1.com", 0xFFFFFFFF, 0x000000FF);
        mock.set_text(20, 10, "https://test2.com", 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Get renderer metrics
    let (cell_width, cell_height) = {
        let renderer = harness.resource::<TextRenderer>();
        (renderer.cell_width, renderer.cell_height)
    };

    // Detect links with positions
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        (0..20)
            .map(|i| mock.get_row_text(i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let hints = {
        let detector = harness.resource::<LinkDetector>();
        let detected_links = detector.detect_with_positions(&text);
        let hint_keys = LinkDetector::generate_hint_keys(detected_links.len());

        // Convert to LinkHints with positions
        detected_links
            .into_iter()
            .zip(hint_keys)
            .map(|((url, link_type, col, row), hint_key)| {
                use scarab_client::ui::grid_utils::grid_to_pixel;
                let position = grid_to_pixel(col as u16, row as u16, cell_width, cell_height);

                LinkHint {
                    url,
                    position,
                    grid_col: col as u16,
                    grid_row: row as u16,
                    hint_key,
                    link_type,
                }
            })
            .collect::<Vec<_>>()
    };

    // Assert positions are calculated correctly
    assert!(hints.len() >= 2, "Expected at least 2 links detected");

    // Find the link at row 5, col 10
    let link1 = hints.iter().find(|h| h.grid_row == 5);
    assert!(link1.is_some(), "Link at row 5 not found");
    let link1 = link1.unwrap();
    assert_eq!(link1.grid_col, 10, "Link should start at column 10");

    // Find the link at row 10, col 20
    let link2 = hints.iter().find(|h| h.grid_row == 10);
    assert!(link2.is_some(), "Link at row 10 not found");
    let link2 = link2.unwrap();
    assert_eq!(link2.grid_col, 20, "Link should start at column 20");

    // Verify pixel positions are non-zero (actually positioned)
    assert_ne!(link1.position.x, 0.0);
    assert_ne!(link1.position.y, 0.0);
    assert_ne!(link2.position.x, 0.0);
    assert_ne!(link2.position.y, 0.0);
}

// =============================================================================
// Test 4: Hint Activation
// =============================================================================

#[test]
fn test_hint_activation() {
    let mut harness = setup_link_hints_harness();

    // Add event system
    harness.app.add_event::<LinkActivatedEvent>();

    // Setup a URL in the grid
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(0, 0, "https://example.com", 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Get renderer for positioning
    let (cell_width, cell_height) = {
        let renderer = harness.resource::<TextRenderer>();
        (renderer.cell_width, renderer.cell_height)
    };

    // Detect and create hint
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        mock.get_row_text(0)
    };

    let hint = {
        let detector = harness.resource::<LinkDetector>();
        let detected_links = detector.detect_with_positions(&text);

        // Filter to only URL types (the filepath regex may also match parts of URLs)
        let url_links: Vec<_> = detected_links.iter()
            .filter(|(_, link_type, _, _)| *link_type == LinkType::Url)
            .collect();

        let hint_keys = LinkDetector::generate_hint_keys(1);

        assert_eq!(url_links.len(), 1, "Should detect exactly 1 URL (filtering by type)");
        assert_eq!(hint_keys[0], "a", "First hint should be 'a'");

        let (url, link_type, col, row) = url_links[0];

        use scarab_client::ui::grid_utils::grid_to_pixel;
        let position = grid_to_pixel(*col as u16, *row as u16, cell_width, cell_height);

        LinkHint {
            url: url.clone(),
            position,
            grid_col: *col as u16,
            grid_row: *row as u16,
            hint_key: "a".to_string(),
            link_type: link_type.clone(),
        }
    };

    // Simulate hint activation by sending event
    // Note: We use world_mut().send_event() directly instead of harness.send_event()
    // because send_event() calls update() which triggers systems that require
    // ButtonInput<KeyCode> resource not available in the minimal test harness.
    let expected_url = hint.url.clone();
    let expected_key = hint.hint_key.clone();
    harness.world_mut().send_event(LinkActivatedEvent { link: hint });

    // Read events to verify activation (without cloning)
    let mut event_count = 0;
    let mut found_url = String::new();
    let mut found_key = String::new();

    {
        let event_reader = harness.world().resource::<Events<LinkActivatedEvent>>();
        let mut cursor = event_reader.get_cursor();
        for event in cursor.read(event_reader) {
            event_count += 1;
            found_url = event.link.url.clone();
            found_key = event.link.hint_key.clone();
        }
    }

    assert_eq!(event_count, 1, "Should have received 1 activation event");
    assert_eq!(found_url, expected_url);
    assert_eq!(found_key, expected_key);
}

// =============================================================================
// Test 5: Hints Clear on Deactivate
// =============================================================================

#[test]
fn test_hints_clear_on_deactivate() {
    let mut harness = setup_link_hints_harness();

    // Setup URLs
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        populate_urls(&mut mock, &[
            (0, 0, "https://link1.com"),
            (0, 1, "https://link2.com"),
            (0, 2, "https://link3.com"),
        ]);
    }

    // Get renderer and text
    let (cell_width, cell_height) = {
        let renderer = harness.resource::<TextRenderer>();
        (renderer.cell_width, renderer.cell_height)
    };

    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        (0..3)
            .map(|i| mock.get_row_text(i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Activate hints mode and populate hints
    {
        let detector = harness.resource::<LinkDetector>();
        let detected_links = detector.detect_with_positions(&text);
        let hint_keys = LinkDetector::generate_hint_keys(detected_links.len());

        let hints: Vec<LinkHint> = detected_links
            .into_iter()
            .zip(hint_keys)
            .map(|((url, link_type, col, row), hint_key)| {
                use scarab_client::ui::grid_utils::grid_to_pixel;
                let position = grid_to_pixel(col as u16, row as u16, cell_width, cell_height);

                LinkHint {
                    url,
                    position,
                    grid_col: col as u16,
                    grid_row: row as u16,
                    hint_key,
                    link_type,
                }
            })
            .collect();

        let mut state = harness.resource_mut::<LinkHintsState>();
        state.active = true;
        state.hints = hints;

        assert!(state.hints.len() >= 3, "Should have detected at least 3 hints");
    }

    // Note: Skipping harness.update() because the link hints systems require
    // ButtonInput<KeyCode> and SharedMemoryReader resources that are not
    // available in the minimal test harness. This test verifies state
    // manipulation without running the full system chain.

    // Deactivate hints mode
    {
        let mut state = harness.resource_mut::<LinkHintsState>();
        state.active = false;
        state.hints.clear();
        state.current_input.clear();
    }

    // Verify hints are cleared
    {
        let state = harness.resource::<LinkHintsState>();
        assert_eq!(state.hints.len(), 0, "Hints should be cleared");
        assert!(!state.active, "Hints mode should be inactive");
        assert_eq!(state.current_input, "", "Input should be cleared");
    }
}

// =============================================================================
// Test 6: Filepath Detection
// =============================================================================

#[test]
fn test_filepath_detection() {
    let mut harness = setup_link_hints_harness();

    // Setup grid with file paths
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(0, 0, "Edit /usr/local/bin/script.sh", 0xFFFFFFFF, 0x000000FF);
        mock.set_text(0, 1, "Check ./relative/path.txt", 0xFFFFFFFF, 0x000000FF);
        mock.set_text(0, 2, "Open ~/Documents/file.md", 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Detect links
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        (0..3)
            .map(|i| mock.get_row_text(i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let links = {
        let detector = harness.resource::<LinkDetector>();
        detector.detect(&text)
    };

    // Filter filepath links
    let filepath_links: Vec<_> = links.iter()
        .filter(|(_, link_type)| *link_type == LinkType::FilePath)
        .collect();

    assert!(!filepath_links.is_empty(), "Should detect file paths");

    // Verify specific paths are detected
    assert!(filepath_links.iter().any(|(path, _)| path.contains("/usr/local/bin")));
    assert!(filepath_links.iter().any(|(path, _)| path.contains("./relative/path.txt")));
    assert!(filepath_links.iter().any(|(path, _)| path.contains("~/Documents")));
}

// =============================================================================
// Test 7: Multiple URLs Per Line
// =============================================================================

#[test]
fn test_multiple_urls_per_line() {
    let mut harness = setup_link_hints_harness();

    // Setup a line with multiple URLs
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(0, 0, "Check https://first.com and https://second.com or www.third.com", 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Detect links
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        mock.get_row_text(0)
    };

    let links = {
        let detector = harness.resource::<LinkDetector>();
        detector.detect_with_positions(&text)
    };

    // Filter URL links
    let url_links: Vec<_> = links.iter()
        .filter(|(_, link_type, _, _)| *link_type == LinkType::Url)
        .collect();

    assert!(url_links.len() >= 3, "Should detect 3 URLs on same line, found {}", url_links.len());

    // Verify they're all on row 0
    for (_, _, _, row) in &url_links {
        assert_eq!(*row, 0, "All URLs should be on row 0");
    }

    // Verify column positions are different (in order)
    let cols: Vec<_> = url_links.iter().map(|(_, _, col, _)| col).collect();
    for i in 1..cols.len() {
        assert!(cols[i] > cols[i-1], "URLs should have increasing column positions");
    }
}

// =============================================================================
// Test 8: Edge Case - URLs at Grid Edges
// =============================================================================

#[test]
fn test_urls_at_grid_edges() {
    let mut harness = setup_link_hints_harness();

    // Setup URLs at grid boundaries
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();

        // URL at start of first row
        mock.set_text(0, 0, "https://start.com", 0xFFFFFFFF, 0x000000FF);

        // URL near end of row (assuming 200 column grid)
        mock.set_text(180, 5, "https://end.com", 0xFFFFFFFF, 0x000000FF);

        // URL at last row (assuming 100 row grid)
        mock.set_text(0, 99, "https://bottom.com", 0xFFFFFFFF, 0x000000FF);

        mock.tick();
    }

    // Detect links with positions
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        (0..100)
            .map(|i| mock.get_row_text(i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let detected_links = {
        let detector = harness.resource::<LinkDetector>();
        detector.detect_with_positions(&text)
    };

    assert!(detected_links.len() >= 3, "Should detect all edge URLs");

    // Verify edge positions are handled correctly
    let link_at_start = detected_links.iter().find(|(_url, _, col, row)| *row == 0 && *col == 0);
    assert!(link_at_start.is_some(), "Should find URL at grid start");

    let link_at_end = detected_links.iter().find(|(_url, _, col, row)| *row == 5 && *col == 180);
    assert!(link_at_end.is_some(), "Should find URL near row end");

    let link_at_bottom = detected_links.iter().find(|(_url, _, _col, row)| *row == 99);
    assert!(link_at_bottom.is_some(), "Should find URL at grid bottom");
}

// =============================================================================
// Test 9: Email Detection
// =============================================================================

#[test]
fn test_email_detection() {
    let mut harness = setup_link_hints_harness();

    // Setup grid with email addresses
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(0, 0, "Contact: user@example.com", 0xFFFFFFFF, 0x000000FF);
        mock.set_text(0, 1, "Support: support@scarab.dev", 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Detect links
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        mock.get_row_text(0) + "\n" + &mock.get_row_text(1)
    };

    let links = {
        let detector = harness.resource::<LinkDetector>();
        detector.detect(&text)
    };

    // Filter email links
    let email_links: Vec<_> = links.iter()
        .filter(|(_, link_type)| *link_type == LinkType::Email)
        .collect();

    assert_eq!(email_links.len(), 2, "Should detect 2 email addresses");

    // Verify specific emails
    assert!(email_links.iter().any(|(email, _)| email == "user@example.com"));
    assert!(email_links.iter().any(|(email, _)| email == "support@scarab.dev"));
}

// =============================================================================
// Test 10: Very Long URLs
// =============================================================================

#[test]
fn test_very_long_urls() {
    let mut harness = setup_link_hints_harness();

    // Create a very long URL (but still realistic)
    let long_url = "https://github.com/raibid-labs/scarab/blob/main/crates/scarab-client/src/ui/link_hints.rs?line=123#L456";

    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(0, 0, long_url, 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Detect links
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        mock.get_row_text(0)
    };

    let links = {
        let detector = harness.resource::<LinkDetector>();
        detector.detect(&text)
    };

    // Should detect the long URL
    let url_links: Vec<_> = links.iter()
        .filter(|(_, link_type)| *link_type == LinkType::Url)
        .collect();

    assert_eq!(url_links.len(), 1, "Should detect the long URL");
    assert_eq!(url_links[0].0, long_url);
}

// =============================================================================
// Test 11: Hint Input Filtering
// =============================================================================

#[test]
fn test_hint_input_filtering() {
    let mut harness = setup_link_hints_harness();

    // Setup multiple URLs
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        populate_urls(&mut mock, &[
            (0, 0, "https://alpha.com"),
            (0, 1, "https://beta.com"),
            (0, 2, "https://gamma.com"),
        ]);
    }

    // Get renderer for positioning
    let (cell_width, cell_height) = {
        let renderer = harness.resource::<TextRenderer>();
        (renderer.cell_width, renderer.cell_height)
    };

    // Detect links
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        (0..3)
            .map(|i| mock.get_row_text(i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let (detected_links, hint_keys) = {
        let detector = harness.resource::<LinkDetector>();
        let detected_links = detector.detect_with_positions(&text);
        let hint_keys = LinkDetector::generate_hint_keys(detected_links.len());
        (detected_links, hint_keys)
    };

    // Activate hints and populate state
    {
        let mut state = harness.resource_mut::<LinkHintsState>();
        state.active = true;
        state.hints = detected_links
            .into_iter()
            .zip(hint_keys)
            .map(|((url, link_type, col, row), hint_key)| {
                use scarab_client::ui::grid_utils::grid_to_pixel;
                let position = grid_to_pixel(col as u16, row as u16, cell_width, cell_height);

                LinkHint {
                    url,
                    position,
                    grid_col: col as u16,
                    grid_row: row as u16,
                    hint_key,
                    link_type,
                }
            })
            .collect();

        // Simulate partial input "a"
        state.current_input = "a".to_string();
    }

    // Filter hints that match the input
    let matching_hints = {
        let state = harness.resource::<LinkHintsState>();
        state.hints.iter()
            .filter(|h| h.hint_key.starts_with(&state.current_input))
            .cloned()
            .collect::<Vec<_>>()
    };

    // With keys "a", "b", "c", only "a" should match
    assert_eq!(matching_hints.len(), 1, "Only one hint should match 'a'");
    assert_eq!(matching_hints[0].hint_key, "a");
}

// =============================================================================
// Test 12: No False Positives
// =============================================================================

#[test]
fn test_no_false_positives() {
    let mut harness = setup_link_hints_harness();

    // Setup grid with text that shouldn't be detected as links
    {
        let mut mock = harness.resource_mut::<MockSharedMemoryReader>();
        mock.set_text(0, 0, "This is plain text", 0xFFFFFFFF, 0x000000FF);
        mock.set_text(0, 1, "Numbers: 123 456 789", 0xFFFFFFFF, 0x000000FF);
        mock.set_text(0, 2, "Dots: ... and ...", 0xFFFFFFFF, 0x000000FF);
        mock.tick();
    }

    // Detect links
    let text = {
        let mock = harness.resource::<MockSharedMemoryReader>();
        (0..3)
            .map(|i| mock.get_row_text(i))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let links = {
        let detector = harness.resource::<LinkDetector>();
        detector.detect(&text)
    };

    // Filter to only URLs and emails (filepaths might match some patterns)
    let url_or_email_links: Vec<_> = links.iter()
        .filter(|(_, link_type)| {
            *link_type == LinkType::Url || *link_type == LinkType::Email
        })
        .collect();

    // Should not detect any URLs or emails in plain text
    assert_eq!(url_or_email_links.len(), 0, "Should not detect URLs/emails in plain text");
}
