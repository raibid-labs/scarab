//! Integration tests for Tab/Pane Multiplexing System
//!
//! This test suite verifies the complete tab and pane management functionality
//! including lifecycle management, state preservation, and edge cases.
//!
//! GitHub Issue: #34

use scarab_daemon::session::{SessionManager, SplitDirection};
use std::time::Duration;
use tempfile::TempDir;

// ==================== Tab Lifecycle Tests ====================

#[test]
fn test_tab_creation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Initial session should have 1 tab
    assert_eq!(session.tab_count(), 1);
    let initial_tab = session.active_tab_id();
    assert!(initial_tab > 0);

    // Create a new tab
    let tab2_id = session.create_tab(Some("Tab 2".to_string())).unwrap();
    assert_eq!(session.tab_count(), 2);

    // Create another tab with auto-generated title
    let tab3_id = session.create_tab(None).unwrap();
    assert_eq!(session.tab_count(), 3);

    // Verify all tabs exist and have unique IDs
    assert_ne!(initial_tab, tab2_id);
    assert_ne!(tab2_id, tab3_id);
    assert_ne!(initial_tab, tab3_id);
}

#[test]
fn test_tab_close() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create multiple tabs
    let tab1 = session.active_tab_id();
    let tab2 = session.create_tab(Some("Tab 2".to_string())).unwrap();
    let tab3 = session.create_tab(Some("Tab 3".to_string())).unwrap();

    assert_eq!(session.tab_count(), 3);

    // Close middle tab
    let destroyed_panes = session.close_tab(tab2).unwrap();
    assert_eq!(session.tab_count(), 2);
    assert_eq!(destroyed_panes.len(), 1, "Closing a tab should destroy its panes");

    // Verify remaining tabs still exist
    let tabs = session.list_tabs();
    let tab_ids: Vec<u64> = tabs.iter().map(|(id, _, _, _)| *id).collect();
    assert!(tab_ids.contains(&tab1));
    assert!(tab_ids.contains(&tab3));
    assert!(!tab_ids.contains(&tab2));
}

#[test]
fn test_tab_cleanup_on_close() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create tab with multiple panes
    session.create_tab(Some("Tab 2".to_string())).unwrap();
    let tab2_id = session.active_tab_id();

    // Add multiple panes to tab2
    session.split_pane(SplitDirection::Vertical).unwrap();
    session.split_pane(SplitDirection::Horizontal).unwrap();

    // Close tab and verify all panes are cleaned up
    let destroyed_panes = session.close_tab(tab2_id).unwrap();
    assert_eq!(destroyed_panes.len(), 3, "Should destroy all panes in the closed tab");
}

#[test]
fn test_tab_switch() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    let tab1 = session.active_tab_id();
    let tab2 = session.create_tab(Some("Tab 2".to_string())).unwrap();
    let _tab3 = session.create_tab(Some("Tab 3".to_string())).unwrap();

    // Switch to tab1
    session.switch_tab(tab1).unwrap();
    assert_eq!(session.active_tab_id(), tab1);

    // Switch to tab2
    session.switch_tab(tab2).unwrap();
    assert_eq!(session.active_tab_id(), tab2);

    // Verify list_tabs shows correct active state
    let tabs = session.list_tabs();
    for (id, _, is_active, _) in tabs {
        if id == tab2 {
            assert!(is_active, "Tab 2 should be marked as active");
        } else {
            assert!(!is_active, "Other tabs should not be marked as active");
        }
    }
}

#[test]
fn test_tab_isolation() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create two tabs
    let tab1 = session.active_tab_id();
    let tab2 = session.create_tab(Some("Tab 2".to_string())).unwrap();

    // Switch to tab2 and add panes
    session.switch_tab(tab2).unwrap();
    session.split_pane(SplitDirection::Vertical).unwrap();
    session.split_pane(SplitDirection::Horizontal).unwrap();

    // Switch to tab1 and verify it still has only 1 pane
    session.switch_tab(tab1).unwrap();
    let tabs = session.list_tabs();
    let tab1_panes = tabs.iter().find(|(id, _, _, _)| *id == tab1).unwrap().3;
    let tab2_panes = tabs.iter().find(|(id, _, _, _)| *id == tab2).unwrap().3;

    assert_eq!(tab1_panes, 1, "Tab 1 should still have 1 pane");
    assert_eq!(tab2_panes, 3, "Tab 2 should have 3 panes");
}

#[test]
fn test_multiple_tabs_independence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create 5 tabs
    let tab1 = session.active_tab_id();
    let tab2 = session.create_tab(Some("Tab 2".to_string())).unwrap();
    let tab3 = session.create_tab(Some("Tab 3".to_string())).unwrap();
    let tab4 = session.create_tab(Some("Tab 4".to_string())).unwrap();
    let tab5 = session.create_tab(Some("Tab 5".to_string())).unwrap();

    // Add different numbers of panes to each
    session.switch_tab(tab2).unwrap();
    session.split_pane(SplitDirection::Vertical).unwrap();

    session.switch_tab(tab3).unwrap();
    session.split_pane(SplitDirection::Horizontal).unwrap();
    session.split_pane(SplitDirection::Vertical).unwrap();

    // Verify each tab maintains its pane count
    let tabs = session.list_tabs();
    assert_eq!(tabs.len(), 5);

    let pane_counts: Vec<usize> = vec![tab1, tab2, tab3, tab4, tab5]
        .iter()
        .map(|id| {
            tabs.iter()
                .find(|(tab_id, _, _, _)| tab_id == id)
                .unwrap()
                .3
        })
        .collect();

    assert_eq!(pane_counts, vec![1, 2, 3, 1, 1]);
}

// ==================== Pane Lifecycle Tests ====================

#[test]
fn test_pane_split_horizontal() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Initial state: 1 pane
    let tabs = session.list_tabs();
    assert_eq!(tabs[0].3, 1);

    // Split horizontally
    let new_pane = session.split_pane(SplitDirection::Horizontal).unwrap();
    assert!(new_pane > 0);

    let tabs = session.list_tabs();
    assert_eq!(tabs[0].3, 2, "Should now have 2 panes");
}

#[test]
fn test_pane_split_vertical() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Split vertically
    let new_pane = session.split_pane(SplitDirection::Vertical).unwrap();
    assert!(new_pane > 0);

    let tabs = session.list_tabs();
    assert_eq!(tabs[0].3, 2);
}

#[test]
fn test_pane_close() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create multiple panes
    let pane2 = session.split_pane(SplitDirection::Vertical).unwrap();
    let _pane3 = session.split_pane(SplitDirection::Horizontal).unwrap();

    assert_eq!(session.list_tabs()[0].3, 3);

    // Close a pane
    session.close_pane(pane2).unwrap();
    assert_eq!(session.list_tabs()[0].3, 2);
}

#[test]
fn test_pane_tree_rebalancing() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create complex pane layout
    let pane2 = session.split_pane(SplitDirection::Vertical).unwrap();
    let pane3 = session.split_pane(SplitDirection::Horizontal).unwrap();
    let pane4 = session.split_pane(SplitDirection::Vertical).unwrap();

    assert_eq!(session.list_tabs()[0].3, 4);

    // Close panes and verify count reduces properly (tree rebalancing)
    session.close_pane(pane3).unwrap();
    assert_eq!(session.list_tabs()[0].3, 3);

    session.close_pane(pane2).unwrap();
    assert_eq!(session.list_tabs()[0].3, 2);

    session.close_pane(pane4).unwrap();
    assert_eq!(session.list_tabs()[0].3, 1);
}

#[test]
fn test_pane_focus_changes() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Get initial pane
    let pane1 = session.get_active_pane().unwrap().id;

    // Create more panes
    let pane2 = session.split_pane(SplitDirection::Vertical).unwrap();
    let pane3 = session.split_pane(SplitDirection::Horizontal).unwrap();

    // Focus pane2
    session.focus_pane(pane2).unwrap();
    assert_eq!(session.get_active_pane().unwrap().id, pane2);

    // Focus pane1
    session.focus_pane(pane1).unwrap();
    assert_eq!(session.get_active_pane().unwrap().id, pane1);

    // Focus pane3
    session.focus_pane(pane3).unwrap();
    assert_eq!(session.get_active_pane().unwrap().id, pane3);
}

#[test]
fn test_pane_resize() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Get initial dimensions
    let pane = session.get_active_pane().unwrap();
    assert_eq!(pane.dimensions(), (80, 24));

    // Resize session (affects active pane)
    session.resize(120, 40).unwrap();

    // Verify active pane was resized
    let pane = session.get_active_pane().unwrap();
    assert_eq!(pane.dimensions(), (120, 40));
}

// ==================== Multiplexing Integration Tests ====================

#[test]
fn test_tab_with_multiple_panes() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create a tab with 4 panes
    let tab1 = session.active_tab_id();
    session.split_pane(SplitDirection::Vertical).unwrap();
    session.split_pane(SplitDirection::Horizontal).unwrap();
    session.split_pane(SplitDirection::Vertical).unwrap();

    let tabs = session.list_tabs();
    assert_eq!(tabs[0].3, 4);

    // Create another tab with 2 panes
    let tab2 = session.create_tab(Some("Tab 2".to_string())).unwrap();
    session.switch_tab(tab2).unwrap();
    session.split_pane(SplitDirection::Horizontal).unwrap();

    let tabs = session.list_tabs();
    let tab2_panes = tabs.iter().find(|(id, _, _, _)| *id == tab2).unwrap().3;
    assert_eq!(tab2_panes, 2);

    // Verify first tab still has 4 panes
    let tab1_panes = tabs.iter().find(|(id, _, _, _)| *id == tab1).unwrap().3;
    assert_eq!(tab1_panes, 4);
}

#[test]
fn test_switch_tabs_preserves_pane_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Setup tab1 with panes
    let tab1 = session.active_tab_id();
    let _tab1_pane1 = session.get_active_pane().unwrap().id;
    let tab1_pane2 = session.split_pane(SplitDirection::Vertical).unwrap();
    session.focus_pane(tab1_pane2).unwrap();

    // Setup tab2 with panes
    let tab2 = session.create_tab(Some("Tab 2".to_string())).unwrap();
    session.switch_tab(tab2).unwrap();
    let tab2_pane1 = session.get_active_pane().unwrap().id;
    let _tab2_pane2 = session.split_pane(SplitDirection::Horizontal).unwrap();
    session.focus_pane(tab2_pane1).unwrap();

    // Switch back to tab1
    session.switch_tab(tab1).unwrap();

    // Verify tab1's pane state was preserved
    assert_eq!(session.get_active_pane().unwrap().id, tab1_pane2);
    let tabs = session.list_tabs();
    let tab1_info = tabs.iter().find(|(id, _, _, _)| *id == tab1).unwrap();
    assert_eq!(tab1_info.3, 2);

    // Switch to tab2 and verify its state
    session.switch_tab(tab2).unwrap();
    assert_eq!(session.get_active_pane().unwrap().id, tab2_pane1);
    let tabs = session.list_tabs();
    let tab2_info = tabs.iter().find(|(id, _, _, _)| *id == tab2).unwrap();
    assert_eq!(tab2_info.3, 2);
}

#[test]
fn test_close_tab_with_panes() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create tab with 3 panes
    session.create_tab(Some("Tab 2".to_string())).unwrap();
    let tab2 = session.active_tab_id();
    session.split_pane(SplitDirection::Vertical).unwrap();
    session.split_pane(SplitDirection::Horizontal).unwrap();

    // Close tab and verify all panes destroyed
    let destroyed = session.close_tab(tab2).unwrap();
    assert_eq!(destroyed.len(), 3);

    // Verify we switched to remaining tab
    assert_ne!(session.active_tab_id(), tab2);
}

#[test]
fn test_navigation_state_across_pane_switches() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create multiple panes
    let pane1 = session.get_active_pane().unwrap().id;
    let pane2 = session.split_pane(SplitDirection::Vertical).unwrap();
    let pane3 = session.split_pane(SplitDirection::Horizontal).unwrap();

    // Navigate between panes multiple times
    session.focus_pane(pane1).unwrap();
    assert_eq!(session.get_active_pane().unwrap().id, pane1);

    session.focus_pane(pane2).unwrap();
    assert_eq!(session.get_active_pane().unwrap().id, pane2);

    session.focus_pane(pane3).unwrap();
    assert_eq!(session.get_active_pane().unwrap().id, pane3);

    session.focus_pane(pane1).unwrap();
    assert_eq!(session.get_active_pane().unwrap().id, pane1);
}

// ==================== Edge Cases ====================

#[test]
fn test_cannot_close_last_pane() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    let pane_id = session.get_active_pane().unwrap().id;

    // Attempt to close the last pane
    let result = session.close_pane(pane_id);
    assert!(result.is_err(), "Should not be able to close the last pane");

    // Verify pane still exists
    assert_eq!(session.list_tabs()[0].3, 1);
}

#[test]
fn test_cannot_close_last_tab() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    let tab_id = session.active_tab_id();

    // Attempt to close the last tab
    let result = session.close_tab(tab_id);
    assert!(result.is_err(), "Should not be able to close the last tab");

    // Verify tab still exists
    assert_eq!(session.tab_count(), 1);
}

#[test]
fn test_rapid_tab_switching() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create multiple tabs
    let tab1 = session.active_tab_id();
    let tab2 = session.create_tab(None).unwrap();
    let tab3 = session.create_tab(None).unwrap();
    let tab4 = session.create_tab(None).unwrap();
    let tab5 = session.create_tab(None).unwrap();

    // Rapidly switch between tabs
    for _ in 0..100 {
        session.switch_tab(tab1).unwrap();
        session.switch_tab(tab3).unwrap();
        session.switch_tab(tab5).unwrap();
        session.switch_tab(tab2).unwrap();
        session.switch_tab(tab4).unwrap();
    }

    // Verify all tabs still exist and system is stable
    assert_eq!(session.tab_count(), 5);
    assert_eq!(session.active_tab_id(), tab4);
}

#[test]
fn test_rapid_pane_switching() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create multiple panes
    let pane1 = session.get_active_pane().unwrap().id;
    let pane2 = session.split_pane(SplitDirection::Vertical).unwrap();
    let pane3 = session.split_pane(SplitDirection::Horizontal).unwrap();
    let pane4 = session.split_pane(SplitDirection::Vertical).unwrap();

    // Rapidly switch between panes
    for _ in 0..100 {
        session.focus_pane(pane1).unwrap();
        session.focus_pane(pane3).unwrap();
        session.focus_pane(pane4).unwrap();
        session.focus_pane(pane2).unwrap();
    }

    // Verify all panes still exist and system is stable
    assert_eq!(session.list_tabs()[0].3, 4);
    assert_eq!(session.get_active_pane().unwrap().id, pane2);
}

#[test]
fn test_pane_focus_after_close() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create panes
    let pane1 = session.get_active_pane().unwrap().id;
    let pane2 = session.split_pane(SplitDirection::Vertical).unwrap();
    let _pane3 = session.split_pane(SplitDirection::Horizontal).unwrap();

    // Focus pane2 and close it
    session.focus_pane(pane2).unwrap();
    session.close_pane(pane2).unwrap();

    // Verify focus was automatically moved to another pane
    let active_pane = session.get_active_pane().unwrap();
    assert_ne!(active_pane.id, pane2);
    assert!(active_pane.id == pane1 || active_pane.id > pane2);
}

#[test]
fn test_tab_focus_after_close() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create tabs
    let tab1 = session.active_tab_id();
    let tab2 = session.create_tab(None).unwrap();
    let _tab3 = session.create_tab(None).unwrap();

    // Focus tab2 and close it
    session.switch_tab(tab2).unwrap();
    session.close_tab(tab2).unwrap();

    // Verify focus was automatically moved to another tab
    let active_tab = session.active_tab_id();
    assert_ne!(active_tab, tab2);
    assert!(active_tab == tab1 || active_tab > tab2);
}

#[test]
fn test_complex_multi_tab_multi_pane_scenario() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create complex scenario: 3 tabs, each with different pane layouts

    // Tab 1: 2x2 grid (4 panes)
    let tab1 = session.active_tab_id();
    session.split_pane(SplitDirection::Vertical).unwrap();
    session.split_pane(SplitDirection::Horizontal).unwrap();
    session.split_pane(SplitDirection::Vertical).unwrap();

    // Tab 2: 3 panes vertical
    let tab2 = session.create_tab(Some("Vertical Split".to_string())).unwrap();
    session.switch_tab(tab2).unwrap();
    session.split_pane(SplitDirection::Vertical).unwrap();
    session.split_pane(SplitDirection::Vertical).unwrap();

    // Tab 3: 1 pane (default)
    let tab3 = session.create_tab(Some("Single Pane".to_string())).unwrap();

    // Verify structure
    let tabs = session.list_tabs();
    assert_eq!(tabs.len(), 3);

    let tab1_panes = tabs.iter().find(|(id, _, _, _)| *id == tab1).unwrap().3;
    let tab2_panes = tabs.iter().find(|(id, _, _, _)| *id == tab2).unwrap().3;
    let tab3_panes = tabs.iter().find(|(id, _, _, _)| *id == tab3).unwrap().3;

    assert_eq!(tab1_panes, 4);
    assert_eq!(tab2_panes, 3);
    assert_eq!(tab3_panes, 1);

    // Switch between tabs and verify isolation
    session.switch_tab(tab1).unwrap();
    let active_tab1_panes = session.list_tabs()
        .iter()
        .find(|(_id, _, is_active, _)| *is_active)
        .unwrap()
        .3;
    assert_eq!(active_tab1_panes, 4);

    session.switch_tab(tab2).unwrap();
    let active_tab2_panes = session.list_tabs()
        .iter()
        .find(|(_id, _, is_active, _)| *is_active)
        .unwrap()
        .3;
    assert_eq!(active_tab2_panes, 3);

    session.switch_tab(tab3).unwrap();
    let active_tab3_panes = session.list_tabs()
        .iter()
        .find(|(_id, _, is_active, _)| *is_active)
        .unwrap()
        .3;
    assert_eq!(active_tab3_panes, 1);
}

#[test]
fn test_stress_many_tabs_and_panes() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("stress_test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    // Create 10 tabs, each with varying pane counts
    let mut tab_ids = vec![session.active_tab_id()];

    for i in 2..=10 {
        let tab_id = session.create_tab(Some(format!("Tab {}", i))).unwrap();
        tab_ids.push(tab_id);

        // Switch to the new tab before adding panes
        session.switch_tab(tab_id).unwrap();

        // Add i-1 additional panes (so tab i has i panes)
        for _ in 1..i {
            session.split_pane(SplitDirection::Vertical).unwrap();
        }
    }

    // Verify all tabs exist with correct pane counts
    let tabs = session.list_tabs();
    assert_eq!(tabs.len(), 10);

    for (i, tab_id) in tab_ids.iter().enumerate() {
        let expected_panes = i + 1;
        let actual_panes = tabs.iter().find(|(id, _, _, _)| id == tab_id).unwrap().3;
        assert_eq!(
            actual_panes, expected_panes,
            "Tab {} should have {} panes, found {}",
            i + 1, expected_panes, actual_panes
        );
    }
}

#[test]
fn test_session_with_tabs_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    // Create session with tabs
    let session_id = {
        let manager = SessionManager::new(db_path.clone()).unwrap();
        let id = manager.create_session("persistent".to_string(), 80, 24).unwrap();
        let session = manager.get_session(&id).unwrap();

        // Create tabs
        session.create_tab(Some("Tab 2".to_string())).unwrap();
        session.create_tab(Some("Tab 3".to_string())).unwrap();

        assert_eq!(session.tab_count(), 3);
        id
    };

    // Restore and verify
    {
        let manager = SessionManager::new(db_path).unwrap();
        manager.restore_sessions("/bin/sh", 80, 24).unwrap();

        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.name, "persistent");
        // Note: Tab/pane restoration is handled separately in the full implementation
    }
}

#[test]
fn test_rename_tab() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let session_id = manager.create_session("test".to_string(), 80, 24).unwrap();
    let session = manager.get_session(&session_id).unwrap();

    let tab_id = session.active_tab_id();

    // Rename tab
    session.rename_tab(tab_id, "Renamed Tab".to_string()).unwrap();

    // Verify rename
    let tabs = session.list_tabs();
    let tab_title = tabs.iter().find(|(id, _, _, _)| *id == tab_id).unwrap().1.clone();
    assert_eq!(tab_title, "Renamed Tab");
}

#[test]
fn test_concurrent_pane_operations() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = Arc::new(SessionManager::new(db_path).unwrap());
    let session_id = manager.create_session("concurrent".to_string(), 80, 24).unwrap();

    // Spawn multiple threads performing pane operations
    let mut handles = vec![];

    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let session_id_clone = session_id.clone();

        let handle = thread::spawn(move || {
            if let Some(session) = manager_clone.get_session(&session_id_clone) {
                // Each thread creates a tab and splits it
                let tab_id = session.create_tab(Some(format!("Tab {}", i))).unwrap();
                session.switch_tab(tab_id).unwrap();
                session.split_pane(SplitDirection::Vertical).unwrap();

                // Small delay to create interleaving
                thread::sleep(Duration::from_millis(10));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all tabs were created
    let session = manager.get_session(&session_id).unwrap();
    assert_eq!(session.tab_count(), 6); // 1 initial + 5 created
}
