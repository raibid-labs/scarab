# Proposal: Scarab Dock & Plugin Menu System

**Goal:** Create a unified "Dock" UI in Scarab to display loaded plugins (like `| phage | tolaria |`) and provide a standardized, keyboard-navigable menu system for each.

## 1. The Scarab Dock
A dedicated UI pane at the bottom of the Scarab window (rendered via Bevy).
*   **Appearance:** A horizontal bar of cells. Each cell represents a plugin.
*   **Content:** Uses `PluginInfo.emoji` and `PluginInfo.name` (e.g., `🦠 Phage`, `🏭 Tolaria`).
*   **Interaction:**
    *   The Dock is registered as an `InteractiveElement` in the `scarab-nav-protocol`.
    *   This makes it "hintable" (e.g., user presses `Leader` -> hints appear over dock items).
    *   Activating a dock item opens its **Plugin Menu**.

## 2. The Plugin Menu System
We extend `Plugin` trait in `scarab-plugin-api` to enforce a "Menu Contract".

### API Extension (`scarab-plugin-api`)
```rust
pub struct MenuItem {
    pub label: String,
    pub icon: Option<String>, // Emoji
    pub action: MenuAction,
    pub shortcut: Option<String>, // e.g. "Ctrl+P"
}

pub enum MenuAction {
    Command(String),    // Execute a Scarab command
    Remote(String),     // Send a remote ID to the plugin (on_remote_command)
    SubMenu(Vec<MenuItem>),
}

#[async_trait]
pub trait Plugin: Send + Sync {
    // ... existing methods ...

    /// Define the top-level menu for this plugin
    fn get_menu(&self) -> Vec<MenuItem> {
        Vec::new() // Default: No menu
    }
}
```

### Navigation Optimization
This solves the "Search Space" problem:
1.  When a user activates a plugin from the Dock, Scarab requests `plugin.get_menu()`.
2.  Scarab renders the menu (as a pop-up or overlay).
3.  **Crucially**, Scarab *knows* exactly where these items are and *immediately* generates `scarab-nav` hints for them (e.g., `a`, `s`, `d`).
4.  The user presses the hint key -> Scarab triggers `MenuAction`.
5.  If `MenuAction::Remote(id)`, Scarab calls `plugin.on_remote_command(id)`.

## 3. Phage Plugin Implementation
The `PhagePlugin` becomes the reference implementation.

*   **Dock Item:** `🦠 Phage`
*   **Menu:**
    *   `💬 Chat` -> Opens Phage Chat Pane.
    *   `📝 Summarize Buffer` -> Pipes current terminal buffer to Phage.
    *   `🐞 Analyze Error` -> Pipes last command output to Phage.
    *   `⚙️ Config` -> Opens configuration.

## 4. Required Changes

### A. `scarab-plugin-api`
*   Add `MenuItem` and `MenuAction` types.
*   Add `get_menu()` method to `Plugin` trait.

### B. `scarab` (Core)
*   Implement `DockPlugin`: A built-in system that queries all loaded plugins and renders the bottom bar.
*   Implement `MenuRenderer`: A system to draw the menus.
*   Integration with `scarab-nav-client`:
    *   Report Dock items as `InteractiveElement`.
    *   When a menu is open, report Menu items as `InteractiveElement`.

### C. `phage-plugin-scarab`
*   Implement the new `get_menu()` method.
*   Handle the resulting `on_remote_command` calls.
