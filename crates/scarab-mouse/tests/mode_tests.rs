//! Integration tests for mouse mode detection and management

use scarab_mouse::mode::{AppHeuristics, ModeDetector};
use scarab_mouse::types::MouseMode;

#[test]
fn test_mode_detector_creation() {
    let detector = ModeDetector::new();
    assert_eq!(detector.mode(), MouseMode::Normal);
}

#[test]
fn test_mode_detector_default() {
    let detector = ModeDetector::default();
    assert_eq!(detector.mode(), MouseMode::Normal);
}

#[test]
fn test_detect_x10_mouse_enable() {
    let mut detector = ModeDetector::new();

    let result = detector.scan_sequence(b"\x1b[?1000h");
    assert_eq!(result, Some(MouseMode::Application));
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_detect_x10_mouse_disable() {
    let mut detector = ModeDetector::new();

    // Enable first
    detector.scan_sequence(b"\x1b[?1000h");
    assert_eq!(detector.mode(), MouseMode::Application);

    // Then disable
    let result = detector.scan_sequence(b"\x1b[?1000l");
    assert_eq!(result, Some(MouseMode::Normal));
    assert_eq!(detector.mode(), MouseMode::Normal);
}

#[test]
fn test_detect_button_event_tracking() {
    let mut detector = ModeDetector::new();

    let result = detector.scan_sequence(b"\x1b[?1002h");
    assert_eq!(result, Some(MouseMode::Application));
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_detect_any_event_tracking() {
    let mut detector = ModeDetector::new();

    let result = detector.scan_sequence(b"\x1b[?1003h");
    assert_eq!(result, Some(MouseMode::Application));
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_detect_sgr_mouse_mode() {
    let mut detector = ModeDetector::new();

    let result = detector.scan_sequence(b"\x1b[?1006h");
    assert_eq!(result, Some(MouseMode::Application));
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_detect_urxvt_mouse_mode() {
    let mut detector = ModeDetector::new();

    let result = detector.scan_sequence(b"\x1b[?1015h");
    assert_eq!(result, Some(MouseMode::Application));
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_detect_multiple_modes_in_sequence() {
    let mut detector = ModeDetector::new();

    // Multiple enable sequences
    detector.scan_sequence(b"\x1b[?1000h\x1b[?1006h");
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_scan_sequence_no_match() {
    let mut detector = ModeDetector::new();

    let result = detector.scan_sequence(b"Hello World");
    assert_eq!(result, None);
    assert_eq!(detector.mode(), MouseMode::Normal);
}

#[test]
fn test_scan_sequence_invalid_utf8() {
    let mut detector = ModeDetector::new();

    let invalid = vec![0xFF, 0xFE, 0xFD];
    let result = detector.scan_sequence(&invalid);
    assert_eq!(result, None);
}

#[test]
fn test_scan_sequence_partial_match() {
    let mut detector = ModeDetector::new();

    // Incomplete sequence
    let result = detector.scan_sequence(b"\x1b[?1000");
    assert_eq!(result, None);
    assert_eq!(detector.mode(), MouseMode::Normal);
}

#[test]
fn test_scan_sequence_with_surrounding_data() {
    let mut detector = ModeDetector::new();

    // Mode change embedded in other data
    let result = detector.scan_sequence(b"Some text \x1b[?1000h more text");
    assert_eq!(result, Some(MouseMode::Application));
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_set_mode_manual_override() {
    let mut detector = ModeDetector::new();

    detector.set_mode(MouseMode::Application);
    assert_eq!(detector.mode(), MouseMode::Application);

    detector.set_mode(MouseMode::Normal);
    assert_eq!(detector.mode(), MouseMode::Normal);
}

#[test]
fn test_mode_toggle_sequence() {
    let mut detector = ModeDetector::new();

    // Enable
    detector.scan_sequence(b"\x1b[?1000h");
    assert_eq!(detector.mode(), MouseMode::Application);

    // Disable
    detector.scan_sequence(b"\x1b[?1000l");
    assert_eq!(detector.mode(), MouseMode::Normal);

    // Enable again
    detector.scan_sequence(b"\x1b[?1000h");
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_disable_sequences() {
    let mut detector = ModeDetector::new();

    // Set to application mode first
    detector.set_mode(MouseMode::Application);

    // Test all disable sequences
    let disable_sequences = vec![
        b"\x1b[?1000l" as &[u8],
        b"\x1b[?1002l",
        b"\x1b[?1003l",
        b"\x1b[?1006l",
        b"\x1b[?1015l",
    ];

    for seq in disable_sequences {
        detector.set_mode(MouseMode::Application);
        let result = detector.scan_sequence(seq);
        assert_eq!(result, Some(MouseMode::Normal));
        assert_eq!(detector.mode(), MouseMode::Normal);
    }
}

#[test]
fn test_app_heuristics_vim() {
    assert!(AppHeuristics::detect_mouse_app("vim"));
    assert!(AppHeuristics::detect_mouse_app("nvim"));
    assert!(AppHeuristics::detect_mouse_app("VIM")); // case insensitive
}

#[test]
fn test_app_heuristics_emacs() {
    assert!(AppHeuristics::detect_mouse_app("emacs"));
    assert!(AppHeuristics::detect_mouse_app("EMACS"));
}

#[test]
fn test_app_heuristics_nano() {
    assert!(AppHeuristics::detect_mouse_app("nano"));
    assert!(AppHeuristics::detect_mouse_app("NANO"));
}

#[test]
fn test_app_heuristics_tmux() {
    assert!(AppHeuristics::detect_mouse_app("tmux"));
    assert!(AppHeuristics::detect_mouse_app("screen"));
}

#[test]
fn test_app_heuristics_system_monitors() {
    assert!(AppHeuristics::detect_mouse_app("htop"));
}

#[test]
fn test_app_heuristics_pagers() {
    assert!(AppHeuristics::detect_mouse_app("less"));
    assert!(AppHeuristics::detect_mouse_app("more"));
}

#[test]
fn test_app_heuristics_file_managers() {
    assert!(AppHeuristics::detect_mouse_app("ranger"));
    assert!(AppHeuristics::detect_mouse_app("mc"));
}

#[test]
fn test_app_heuristics_fuzzy_finder() {
    assert!(AppHeuristics::detect_mouse_app("fzf"));
}

#[test]
fn test_app_heuristics_shells() {
    assert!(!AppHeuristics::detect_mouse_app("bash"));
    assert!(!AppHeuristics::detect_mouse_app("zsh"));
    assert!(!AppHeuristics::detect_mouse_app("fish"));
    assert!(!AppHeuristics::detect_mouse_app("sh"));
}

#[test]
fn test_app_heuristics_unknown_app() {
    assert!(!AppHeuristics::detect_mouse_app("unknown_app"));
    assert!(!AppHeuristics::detect_mouse_app(""));
}

#[test]
fn test_app_description_vim() {
    let desc = AppHeuristics::get_app_description("vim");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Vim"));
}

#[test]
fn test_app_description_nvim() {
    let desc = AppHeuristics::get_app_description("nvim");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Vim"));
}

#[test]
fn test_app_description_emacs() {
    let desc = AppHeuristics::get_app_description("emacs");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Emacs"));
}

#[test]
fn test_app_description_tmux() {
    let desc = AppHeuristics::get_app_description("tmux");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Tmux"));
}

#[test]
fn test_app_description_screen() {
    let desc = AppHeuristics::get_app_description("screen");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Screen"));
}

#[test]
fn test_app_description_htop() {
    let desc = AppHeuristics::get_app_description("htop");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Htop"));
}

#[test]
fn test_app_description_pagers() {
    let desc = AppHeuristics::get_app_description("less");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Pager"));

    let desc = AppHeuristics::get_app_description("more");
    assert!(desc.is_some());
}

#[test]
fn test_app_description_file_managers() {
    let desc = AppHeuristics::get_app_description("ranger");
    assert!(desc.is_some());

    let desc = AppHeuristics::get_app_description("mc");
    assert!(desc.is_some());
}

#[test]
fn test_app_description_fzf() {
    let desc = AppHeuristics::get_app_description("fzf");
    assert!(desc.is_some());
    assert!(desc.unwrap().contains("Fuzzy"));
}

#[test]
fn test_app_description_unknown() {
    let desc = AppHeuristics::get_app_description("bash");
    assert!(desc.is_none());

    let desc = AppHeuristics::get_app_description("unknown");
    assert!(desc.is_none());
}

#[test]
fn test_app_description_case_insensitive() {
    let desc1 = AppHeuristics::get_app_description("vim");
    let desc2 = AppHeuristics::get_app_description("VIM");
    assert_eq!(desc1, desc2);
}

#[test]
fn test_mouse_mode_is_application() {
    let mode = MouseMode::Application;
    assert!(mode.is_application());
    assert!(!mode.is_normal());
}

#[test]
fn test_mouse_mode_is_normal() {
    let mode = MouseMode::Normal;
    assert!(mode.is_normal());
    assert!(!mode.is_application());
}

#[test]
fn test_mode_persistence() {
    let mut detector = ModeDetector::new();

    detector.scan_sequence(b"\x1b[?1000h");
    assert_eq!(detector.mode(), MouseMode::Application);

    // Mode should persist across non-matching sequences
    detector.scan_sequence(b"random data");
    assert_eq!(detector.mode(), MouseMode::Application);

    detector.scan_sequence(b"more random data");
    assert_eq!(detector.mode(), MouseMode::Application);
}

#[test]
fn test_empty_sequence() {
    let mut detector = ModeDetector::new();
    let result = detector.scan_sequence(b"");
    assert_eq!(result, None);
}

#[test]
fn test_multiple_enable_sequences() {
    let mut detector = ModeDetector::new();

    // Enable multiple times should stay in Application mode
    detector.scan_sequence(b"\x1b[?1000h");
    detector.scan_sequence(b"\x1b[?1002h");
    detector.scan_sequence(b"\x1b[?1003h");

    assert_eq!(detector.mode(), MouseMode::Application);
}
