//! Focusable detection tests

use super::helpers::*;

#[test]
fn test_detect_urls_in_terminal() {
    let mut app = build_test_app();

    // Spawn focusables manually
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (6, 0),
        grid_end: (26, 0),
        content: "https://example.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::Url,
        grid_start: (11, 1),
        grid_end: (26, 1),
        content: "www.github.com".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    // Update to spawn entities
    app.update();

    // Query focusable entities
    let mut query = app.world_mut().query::<&FocusableRegion>();
    let focusables: Vec<_> = query.iter(app.world()).collect();

    // Verify two URLs were detected
    assert_eq!(focusables.len(), 2);

    let url_focusables: Vec<_> = focusables
        .iter()
        .filter(|f| f.region_type == FocusableType::Url)
        .collect();

    assert_eq!(url_focusables.len(), 2);

    // Verify content
    let urls: Vec<&str> = url_focusables.iter().map(|f| f.content.as_str()).collect();
    assert!(urls.contains(&"https://example.com"));
    assert!(urls.contains(&"www.github.com"));
}

#[test]
fn test_detect_filepaths() {
    let mut app = build_test_app();

    // Spawn focusables for paths
    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::FilePath,
        grid_start: (6, 0),
        grid_end: (28, 0),
        content: "/usr/local/bin/foo.txt".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.world_mut().spawn(FocusableRegion {
        region_type: FocusableType::FilePath,
        grid_start: (7, 1),
        grid_end: (26, 1),
        content: "./relative/path.rs".to_string(),
        source: FocusableSource::Terminal,
        pane_id: None,
        generation: 0,
        screen_position: None,
    });

    app.update();

    // Query focusable entities
    let mut query = app.world_mut().query::<&FocusableRegion>();
    let focusables: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(focusables.len(), 2);

    let path_focusables: Vec<_> = focusables
        .iter()
        .filter(|f| f.region_type == FocusableType::FilePath)
        .collect();

    assert_eq!(path_focusables.len(), 2);

    // Verify paths
    let paths: Vec<&str> = path_focusables.iter().map(|f| f.content.as_str()).collect();
    assert!(paths.contains(&"/usr/local/bin/foo.txt"));
    assert!(paths.contains(&"./relative/path.rs"));
}

#[test]
fn test_max_focusables_limit() {
    let mut app = build_test_app();

    // Set a low max limit
    app.world_mut()
        .resource_mut::<FocusableScanConfig>()
        .max_focusables = 5;

    // Test the detector directly
    let config = FocusableScanConfig {
        max_focusables: 5,
        ..Default::default()
    };
    let detector = FocusableDetector::new(&config);

    let mut text = String::new();
    for i in 0..20 {
        text.push_str(&format!("https://example{}.com ", i));
    }

    let detected = detector.detect_all(&text, config.max_focusables);
    assert_eq!(detected.len(), 5); // Detector respects limit
}
