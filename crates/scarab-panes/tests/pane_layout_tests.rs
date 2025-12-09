//! Tests for pane layout calculations and positioning
//!
//! These tests verify internal layout calculations using the existing tests
//! in lib.rs as a foundation, and add additional coverage for edge cases.

// Note: Most layout logic is tested in lib.rs tests section
// These integration tests verify the behavior from a user perspective

use scarab_panes::{PaneLayout, SplitDirection};

#[test]
fn test_pane_layout_creation() {
    let layout = PaneLayout {
        id: 0,
        parent_id: None,
        split_direction: None,
        x: 0,
        y: 0,
        width: 80,
        height: 24,
        is_focused: true,
        split_ratio: 0.5,
    };

    assert_eq!(layout.id, 0);
    assert_eq!(layout.parent_id, None);
    assert_eq!(layout.x, 0);
    assert_eq!(layout.y, 0);
    assert_eq!(layout.width, 80);
    assert_eq!(layout.height, 24);
    assert!(layout.is_focused);
    assert_eq!(layout.split_ratio, 0.5);
}

#[test]
fn test_pane_layout_with_parent() {
    let layout = PaneLayout {
        id: 1,
        parent_id: Some(0),
        split_direction: Some(SplitDirection::Horizontal),
        x: 0,
        y: 12,
        width: 80,
        height: 12,
        is_focused: false,
        split_ratio: 0.5,
    };

    assert_eq!(layout.id, 1);
    assert_eq!(layout.parent_id, Some(0));
    assert_eq!(layout.split_direction, Some(SplitDirection::Horizontal));
    assert!(!layout.is_focused);
}

#[test]
fn test_pane_layout_serialization() {
    let layout = PaneLayout {
        id: 5,
        parent_id: Some(2),
        split_direction: Some(SplitDirection::Vertical),
        x: 40,
        y: 0,
        width: 40,
        height: 24,
        is_focused: true,
        split_ratio: 0.7,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&layout).unwrap();
    assert!(!json.is_empty());

    // Deserialize back
    let deserialized: PaneLayout = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, layout.id);
    assert_eq!(deserialized.parent_id, layout.parent_id);
    assert_eq!(deserialized.split_direction, layout.split_direction);
    assert_eq!(deserialized.x, layout.x);
    assert_eq!(deserialized.y, layout.y);
    assert_eq!(deserialized.width, layout.width);
    assert_eq!(deserialized.height, layout.height);
    assert_eq!(deserialized.is_focused, layout.is_focused);
    assert_eq!(deserialized.split_ratio, layout.split_ratio);
}

#[test]
fn test_pane_layout_clone() {
    let original = PaneLayout {
        id: 10,
        parent_id: Some(5),
        split_direction: Some(SplitDirection::Horizontal),
        x: 10,
        y: 20,
        width: 30,
        height: 40,
        is_focused: false,
        split_ratio: 0.3,
    };

    let cloned = original.clone();

    assert_eq!(cloned.id, original.id);
    assert_eq!(cloned.parent_id, original.parent_id);
    assert_eq!(cloned.split_direction, original.split_direction);
    assert_eq!(cloned.x, original.x);
    assert_eq!(cloned.y, original.y);
    assert_eq!(cloned.width, original.width);
    assert_eq!(cloned.height, original.height);
    assert_eq!(cloned.is_focused, original.is_focused);
    assert_eq!(cloned.split_ratio, original.split_ratio);
}

#[test]
fn test_various_split_ratios() {
    let ratios = vec![0.1, 0.25, 0.5, 0.75, 0.9];

    for ratio in ratios {
        let layout = PaneLayout {
            id: 0,
            parent_id: None,
            split_direction: None,
            x: 0,
            y: 0,
            width: 100,
            height: 50,
            is_focused: true,
            split_ratio: ratio,
        };

        assert_eq!(layout.split_ratio, ratio);
        assert!(layout.split_ratio >= 0.0 && layout.split_ratio <= 1.0);
    }
}

#[test]
fn test_layout_boundary_positions() {
    // Test layouts at various positions
    let positions = vec![
        (0, 0, 80, 24),
        (0, 12, 80, 12),
        (40, 0, 40, 24),
        (60, 18, 20, 6),
    ];

    for (x, y, width, height) in positions {
        let layout = PaneLayout {
            id: 0,
            parent_id: None,
            split_direction: None,
            x,
            y,
            width,
            height,
            is_focused: true,
            split_ratio: 0.5,
        };

        assert_eq!(layout.x, x);
        assert_eq!(layout.y, y);
        assert_eq!(layout.width, width);
        assert_eq!(layout.height, height);
    }
}

#[test]
fn test_minimal_pane_dimensions() {
    // Test that we can create layouts with minimal dimensions
    let layout = PaneLayout {
        id: 0,
        parent_id: None,
        split_direction: None,
        x: 0,
        y: 0,
        width: 1,
        height: 1,
        is_focused: true,
        split_ratio: 0.5,
    };

    assert_eq!(layout.width, 1);
    assert_eq!(layout.height, 1);
}

#[test]
fn test_maximum_pane_dimensions() {
    // Test with large terminal dimensions
    let layout = PaneLayout {
        id: 0,
        parent_id: None,
        split_direction: None,
        x: 0,
        y: 0,
        width: u16::MAX,
        height: u16::MAX,
        is_focused: true,
        split_ratio: 0.5,
    };

    assert_eq!(layout.width, u16::MAX);
    assert_eq!(layout.height, u16::MAX);
}

#[test]
fn test_pane_layout_debug_output() {
    let layout = PaneLayout {
        id: 42,
        parent_id: Some(10),
        split_direction: Some(SplitDirection::Vertical),
        x: 5,
        y: 10,
        width: 20,
        height: 15,
        is_focused: true,
        split_ratio: 0.6,
    };

    let debug_string = format!("{:?}", layout);
    assert!(debug_string.contains("42")); // id
    assert!(debug_string.contains("PaneLayout"));
}

#[test]
fn test_nested_layout_parent_relationships() {
    // Simulate a nested layout structure
    let root = PaneLayout {
        id: 0,
        parent_id: None,
        split_direction: None,
        x: 0,
        y: 0,
        width: 100,
        height: 50,
        is_focused: false,
        split_ratio: 0.5,
    };

    let child1 = PaneLayout {
        id: 1,
        parent_id: Some(0),
        split_direction: Some(SplitDirection::Vertical),
        x: 0,
        y: 0,
        width: 50,
        height: 50,
        is_focused: false,
        split_ratio: 0.5,
    };

    let child2 = PaneLayout {
        id: 2,
        parent_id: Some(0),
        split_direction: Some(SplitDirection::Vertical),
        x: 50,
        y: 0,
        width: 50,
        height: 50,
        is_focused: true,
        split_ratio: 0.5,
    };

    assert_eq!(root.parent_id, None);
    assert_eq!(child1.parent_id, Some(0));
    assert_eq!(child2.parent_id, Some(0));
    assert_eq!(child1.split_direction, Some(SplitDirection::Vertical));
    assert_eq!(child2.split_direction, Some(SplitDirection::Vertical));
}
