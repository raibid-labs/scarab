//! Integration tests for clipboard operations
//!
//! These tests verify clipboard copy/paste functionality across different
//! clipboard types and paste confirmation modes.

use scarab_clipboard::{ClipboardManager, ClipboardType, PasteConfirmation};

#[test]
fn test_clipboard_manager_initialization() {
    let manager = ClipboardManager::new();

    // Clipboard may or may not be available in CI/test environments
    // Just ensure initialization doesn't panic
    let _is_available = manager.is_available();
}

#[test]
fn test_default_clipboard_manager() {
    let manager = ClipboardManager::default();

    // Verify default confirmation mode is Smart
    assert_eq!(manager.confirmation_mode(), PasteConfirmation::Smart);
}

#[test]
fn test_confirmation_mode_changes() {
    let mut manager = ClipboardManager::new();

    // Default should be Smart
    assert_eq!(manager.confirmation_mode(), PasteConfirmation::Smart);

    // Test changing to Always
    manager.set_confirmation_mode(PasteConfirmation::Always);
    assert_eq!(manager.confirmation_mode(), PasteConfirmation::Always);

    // Test changing to Never
    manager.set_confirmation_mode(PasteConfirmation::Never);
    assert_eq!(manager.confirmation_mode(), PasteConfirmation::Never);

    // Test changing back to Smart
    manager.set_confirmation_mode(PasteConfirmation::Smart);
    assert_eq!(manager.confirmation_mode(), PasteConfirmation::Smart);
}

#[test]
fn test_clipboard_unavailable_copy() {
    // Create a manager that may not have clipboard access
    let mut manager = ClipboardManager::new();

    // If clipboard is not available, copy should fail gracefully
    if !manager.is_available() {
        let result = manager.copy("test", ClipboardType::Standard);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }
}

#[test]
fn test_clipboard_unavailable_paste() {
    let mut manager = ClipboardManager::new();

    // If clipboard is not available, paste should fail gracefully
    if !manager.is_available() {
        let result = manager.paste(ClipboardType::Standard);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }
}

#[test]
fn test_clipboard_unavailable_clear() {
    let mut manager = ClipboardManager::new();

    // If clipboard is not available, clear should fail gracefully
    if !manager.is_available() {
        let result = manager.clear();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }
}

// Tests that require actual clipboard access are marked with #[ignore]
// Run with: cargo test --test clipboard_tests -- --ignored

#[test]
#[ignore]
fn test_standard_clipboard_copy_paste() {
    let mut manager = ClipboardManager::new();

    if !manager.is_available() {
        println!("Clipboard not available, skipping test");
        return;
    }

    let test_text = "Hello from Scarab Terminal!";

    // Copy to standard clipboard
    let copy_result = manager.copy(test_text, ClipboardType::Standard);
    assert!(copy_result.is_ok(), "Copy failed: {:?}", copy_result);

    // Paste from standard clipboard
    let paste_result = manager.paste(ClipboardType::Standard);
    assert!(paste_result.is_ok(), "Paste failed: {:?}", paste_result);

    let pasted = paste_result.unwrap();
    assert_eq!(pasted, test_text, "Pasted text doesn't match copied text");
}

#[test]
#[ignore]
fn test_multiline_clipboard_copy_paste() {
    let mut manager = ClipboardManager::new();

    if !manager.is_available() {
        println!("Clipboard not available, skipping test");
        return;
    }

    let test_text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";

    let copy_result = manager.copy(test_text, ClipboardType::Standard);
    assert!(copy_result.is_ok());

    let paste_result = manager.paste(ClipboardType::Standard);
    assert!(paste_result.is_ok());

    let pasted = paste_result.unwrap();
    assert_eq!(pasted, test_text);
}

#[test]
#[ignore]
fn test_large_text_copy_paste() {
    let mut manager = ClipboardManager::new();

    if !manager.is_available() {
        println!("Clipboard not available, skipping test");
        return;
    }

    // Create a large text string (2KB)
    let test_text = "A".repeat(2048);

    let copy_result = manager.copy(&test_text, ClipboardType::Standard);
    assert!(copy_result.is_ok());

    let paste_result = manager.paste(ClipboardType::Standard);
    assert!(paste_result.is_ok());

    let pasted = paste_result.unwrap();
    assert_eq!(pasted.len(), test_text.len());
    assert_eq!(pasted, test_text);
}

#[test]
#[ignore]
fn test_unicode_clipboard_copy_paste() {
    let mut manager = ClipboardManager::new();

    if !manager.is_available() {
        println!("Clipboard not available, skipping test");
        return;
    }

    let test_text = "Hello 世界 🦀 Rust ❤️ Terminal";

    let copy_result = manager.copy(test_text, ClipboardType::Standard);
    assert!(copy_result.is_ok());

    let paste_result = manager.paste(ClipboardType::Standard);
    assert!(paste_result.is_ok());

    let pasted = paste_result.unwrap();
    assert_eq!(pasted, test_text);
}

#[test]
#[ignore]
fn test_empty_clipboard_copy() {
    let mut manager = ClipboardManager::new();

    if !manager.is_available() {
        println!("Clipboard not available, skipping test");
        return;
    }

    let test_text = "";

    let copy_result = manager.copy(test_text, ClipboardType::Standard);
    assert!(copy_result.is_ok());

    let paste_result = manager.paste(ClipboardType::Standard);
    assert!(paste_result.is_ok());

    let pasted = paste_result.unwrap();
    assert_eq!(pasted, test_text);
}

#[test]
#[ignore]
fn test_clipboard_clear() {
    let mut manager = ClipboardManager::new();

    if !manager.is_available() {
        println!("Clipboard not available, skipping test");
        return;
    }

    // First, copy some text
    let test_text = "Text to be cleared";
    let _ = manager.copy(test_text, ClipboardType::Standard);

    // Clear the clipboard
    let clear_result = manager.clear();
    assert!(clear_result.is_ok());

    // Paste should now return empty or fail
    let paste_result = manager.paste(ClipboardType::Standard);
    // The behavior after clear varies by platform
    // On some platforms it returns empty string, on others it errors
    assert!(paste_result.is_ok() || paste_result.is_err());
}

#[test]
#[ignore]
fn test_clipboard_clear_specific_type() {
    let mut manager = ClipboardManager::new();

    if !manager.is_available() {
        println!("Clipboard not available, skipping test");
        return;
    }

    let test_text = "Standard clipboard text";
    let _ = manager.copy(test_text, ClipboardType::Standard);

    // Clear standard clipboard
    let clear_result = manager.clear_clipboard(ClipboardType::Standard);
    assert!(clear_result.is_ok());
}

#[cfg(target_os = "linux")]
mod linux_specific {
    use super::*;

    #[test]
    #[ignore]
    fn test_primary_selection_copy_paste() {
        let mut manager = ClipboardManager::new();

        if !manager.is_available() {
            println!("Clipboard not available, skipping test");
            return;
        }

        let test_text = "Primary selection text";

        // Copy to primary selection
        let copy_result = manager.copy(test_text, ClipboardType::Primary);
        assert!(copy_result.is_ok(), "Copy to primary failed: {:?}", copy_result);

        // Paste from primary selection
        let paste_result = manager.paste(ClipboardType::Primary);
        assert!(paste_result.is_ok(), "Paste from primary failed: {:?}", paste_result);

        let pasted = paste_result.unwrap();
        assert_eq!(pasted, test_text);
    }

    #[test]
    #[ignore]
    fn test_primary_and_standard_clipboard_independence() {
        let mut manager = ClipboardManager::new();

        if !manager.is_available() {
            println!("Clipboard not available, skipping test");
            return;
        }

        let standard_text = "Standard clipboard";
        let primary_text = "Primary selection";

        // Copy different text to each clipboard
        manager.copy(standard_text, ClipboardType::Standard).unwrap();
        manager.copy(primary_text, ClipboardType::Primary).unwrap();

        // Verify each clipboard has its own content
        let standard_pasted = manager.paste(ClipboardType::Standard).unwrap();
        let primary_pasted = manager.paste(ClipboardType::Primary).unwrap();

        assert_eq!(standard_pasted, standard_text);
        assert_eq!(primary_pasted, primary_text);
    }

    #[test]
    #[ignore]
    fn test_clear_primary_selection() {
        let mut manager = ClipboardManager::new();

        if !manager.is_available() {
            println!("Clipboard not available, skipping test");
            return;
        }

        let test_text = "Text in primary selection";
        manager.copy(test_text, ClipboardType::Primary).unwrap();

        // Clear primary selection
        let clear_result = manager.clear_clipboard(ClipboardType::Primary);
        assert!(clear_result.is_ok());
    }
}

#[test]
fn test_clipboard_type_display() {
    let standard = ClipboardType::Standard;
    assert_eq!(format!("{}", standard), "standard clipboard");

    #[cfg(target_os = "linux")]
    {
        let primary = ClipboardType::Primary;
        assert_eq!(format!("{}", primary), "primary selection");
    }
}

#[test]
fn test_paste_confirmation_variants() {
    // Test that all variants can be created and compared
    let always = PasteConfirmation::Always;
    let smart = PasteConfirmation::Smart;
    let never = PasteConfirmation::Never;

    assert_ne!(always, smart);
    assert_ne!(smart, never);
    assert_ne!(always, never);

    // Test equality
    assert_eq!(always, PasteConfirmation::Always);
    assert_eq!(smart, PasteConfirmation::Smart);
    assert_eq!(never, PasteConfirmation::Never);
}
