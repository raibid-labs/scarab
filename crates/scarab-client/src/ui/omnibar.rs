//! Unified omnibar for command palette, file search, history, and more
//!
//! This module provides a provider-based search interface that combines:
//! - Command execution (">")
//! - File search (no prefix)
//! - History search ("#")
//! - Session switching ("@")

use crate::ipc::IpcChannel;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use scarab_protocol::ControlMessage;
use std::sync::Arc;

/// Plugin for unified omnibar functionality
pub struct OmnibarPlugin;

impl Plugin for OmnibarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OmnibarState>()
            .init_resource::<ProviderRegistry>()
            .add_event::<OmnibarExecuteEvent>()
            .add_systems(
                Update,
                (
                    toggle_omnibar_system,
                    handle_omnibar_input_system,
                    render_omnibar_system,
                    execute_action_system,
                    update_results_system,
                )
                    .chain(),
            )
            .add_systems(Startup, register_default_providers_system);
    }
}

/// Context passed to provider execute methods
pub struct OmnibarContext<'a> {
    pub ipc: &'a IpcChannel,
}

/// Trait implemented by all omnibar providers
pub trait OmnibarProvider: Send + Sync {
    /// Provider identifier (e.g., "files", "history", "commands")
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Icon/emoji for this provider
    fn icon(&self) -> &str;

    /// Prefix that activates this provider exclusively (e.g., ">" for commands)
    fn prefix(&self) -> Option<&str>;

    /// Query this provider for results
    fn query(&self, query: &str, limit: usize) -> Vec<OmnibarResult>;

    /// Execute a selected result
    fn execute(&self, result: &OmnibarResult, ctx: &mut OmnibarContext);

    /// Priority for result ranking (higher = shown first when no prefix)
    fn priority(&self) -> i32 {
        0
    }
}

/// Result from an omnibar provider
#[derive(Clone)]
pub struct OmnibarResult {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: String,
    pub provider_id: String,
    pub score: i64,
    pub data: serde_json::Value,
}

/// Registry of all omnibar providers
#[derive(Resource, Default)]
pub struct ProviderRegistry {
    providers: Vec<Arc<dyn OmnibarProvider>>,
}

impl ProviderRegistry {
    pub fn register(&mut self, provider: Arc<dyn OmnibarProvider>) {
        self.providers.push(provider);
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn OmnibarProvider>> {
        self.providers.iter().find(|p| p.id() == id).cloned()
    }

    pub fn all(&self) -> &[Arc<dyn OmnibarProvider>] {
        &self.providers
    }

    /// Find provider by prefix
    pub fn find_by_prefix(&self, prefix: &str) -> Option<Arc<dyn OmnibarProvider>> {
        self.providers
            .iter()
            .find(|p| p.prefix() == Some(prefix))
            .cloned()
    }

    /// Query all providers or a specific one
    pub fn query(&self, query: &str, limit: usize, prefix: Option<&str>) -> Vec<OmnibarResult> {
        if let Some(prefix) = prefix {
            // Query only the provider with this prefix
            if let Some(provider) = self.find_by_prefix(prefix) {
                return provider.query(query, limit);
            }
            return Vec::new();
        }

        // Query all providers and merge results
        let mut all_results: Vec<OmnibarResult> = self
            .providers
            .iter()
            .flat_map(|p| p.query(query, limit))
            .collect();

        // Sort by score (highest first), then by provider priority
        all_results.sort_by(|a, b| {
            b.score.cmp(&a.score).then_with(|| {
                let a_prio = self
                    .get(&a.provider_id)
                    .map(|p| p.priority())
                    .unwrap_or(0);
                let b_prio = self
                    .get(&b.provider_id)
                    .map(|p| p.priority())
                    .unwrap_or(0);
                b_prio.cmp(&a_prio)
            })
        });

        all_results.truncate(limit);
        all_results
    }
}

/// State of the omnibar
#[derive(Resource)]
pub struct OmnibarState {
    pub active: bool,
    pub query: String,
    pub active_prefix: Option<String>,
    pub results: Vec<OmnibarResult>,
    pub selected_index: usize,
    pub hint_mode: bool,
    pub hint_input: String,
}

impl Default for OmnibarState {
    fn default() -> Self {
        Self {
            active: false,
            query: String::new(),
            active_prefix: None,
            results: Vec::new(),
            selected_index: 0,
            hint_mode: false,
            hint_input: String::new(),
        }
    }
}

/// Event fired when an omnibar result is executed
#[derive(Event)]
pub struct OmnibarExecuteEvent {
    pub result: OmnibarResult,
}

/// Component for omnibar UI elements
#[derive(Component)]
pub struct OmnibarUI;

/// Component for result item
#[derive(Component)]
#[allow(dead_code)]
struct ResultItem {
    index: usize,
}

/// Toggle omnibar visibility
fn toggle_omnibar_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<OmnibarState>,
    registry: Res<ProviderRegistry>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Ctrl+P - Open omnibar (all providers)
    if ctrl && !shift && keyboard.just_pressed(KeyCode::KeyP) {
        state.active = !state.active;

        if state.active {
            state.query.clear();
            state.active_prefix = None;
            state.selected_index = 0;
            state.results = registry.query("", 10, None);
        }
    }

    // Ctrl+Shift+P - Open omnibar with ">" prefix (commands only)
    if ctrl && shift && keyboard.just_pressed(KeyCode::KeyP) {
        state.active = !state.active;

        if state.active {
            state.query.clear();
            state.active_prefix = Some(">".to_string());
            state.selected_index = 0;
            state.results = registry.query("", 10, Some(">"));
        }
    }

    // Ctrl+R - Open omnibar with "#" prefix (history)
    if ctrl && keyboard.just_pressed(KeyCode::KeyR) {
        state.active = !state.active;

        if state.active {
            state.query.clear();
            state.active_prefix = Some("#".to_string());
            state.selected_index = 0;
            state.results = registry.query("", 10, Some("#"));
        }
    }

    // Close with Escape
    if state.active && keyboard.just_pressed(KeyCode::Escape) {
        if state.hint_mode {
            // Cancel hint mode
            state.hint_mode = false;
            state.hint_input.clear();
        } else if state.active_prefix.is_some() {
            // Clear prefix
            state.active_prefix = None;
            state.query.clear();
            state.results = registry.query("", 10, None);
        } else {
            // Close omnibar
            state.active = false;
        }
    }
}

/// Handle input in omnibar
fn handle_omnibar_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut char_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut state: ResMut<OmnibarState>,
    _registry: Res<ProviderRegistry>,
    mut execute_events: EventWriter<OmnibarExecuteEvent>,
) {
    if !state.active {
        return;
    }

    // Handle hint mode
    if state.hint_mode {
        for event in char_events.read() {
            if !event.state.is_pressed() {
                continue;
            }

            if let bevy::input::keyboard::Key::Character(ref s) = event.logical_key {
                let ch = s.chars().next().unwrap_or('\0');
                if ch.is_ascii_lowercase() && ch >= 'a' && ch <= 'l' {
                    state.hint_input.push(ch);

                    // If we have a complete hint, execute it
                    let hint_index = (ch as u8 - b'a') as usize;
                    if hint_index < state.results.len() {
                        if let Some(result) = state.results.get(hint_index) {
                            execute_events.send(OmnibarExecuteEvent {
                                result: result.clone(),
                            });
                            state.active = false;
                            state.hint_mode = false;
                            state.hint_input.clear();
                        }
                    }
                }
            }
        }
        return;
    }

    // Handle character input for query
    for event in char_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        if let bevy::input::keyboard::Key::Character(ref s) = event.logical_key {
            // Check for prefix activation
            if state.query.is_empty() && state.active_prefix.is_none() {
                match s.as_str() {
                    ">" | "#" | "@" | ":" | "~" => {
                        state.active_prefix = Some(s.to_string());
                        continue;
                    }
                    _ => {}
                }
            }

            // Skip control characters
            if s.bytes().any(|b| b < 0x20 || b == 0x7F) {
                continue;
            }

            state.query.push_str(s);
        }
    }

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        if state.query.is_empty() && state.active_prefix.is_some() {
            // Remove prefix if query is empty
            state.active_prefix = None;
        } else {
            state.query.pop();
        }
        state.selected_index = 0;
    }

    // Handle navigation
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if state.selected_index < state.results.len().saturating_sub(1) {
            state.selected_index += 1;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        state.selected_index = state.selected_index.saturating_sub(1);
    }

    // Handle selection with Enter
    if keyboard.just_pressed(KeyCode::Enter) {
        if let Some(result) = state.results.get(state.selected_index) {
            execute_events.send(OmnibarExecuteEvent {
                result: result.clone(),
            });
            state.active = false;
        }
    }

    // Toggle hint mode with Tab (when many results)
    if keyboard.just_pressed(KeyCode::Tab) && state.results.len() > 3 {
        state.hint_mode = !state.hint_mode;
        state.hint_input.clear();
    }
}

/// Update results when query changes
fn update_results_system(mut state: ResMut<OmnibarState>, registry: Res<ProviderRegistry>) {
    if !state.active {
        return;
    }

    if state.is_changed() {
        state.results = registry.query(
            &state.query,
            10,
            state.active_prefix.as_ref().map(|s| s.as_str()),
        );
        state.selected_index = state.selected_index.min(state.results.len().saturating_sub(1));
    }
}

/// Render omnibar UI
fn render_omnibar_system(
    mut commands: Commands,
    state: Res<OmnibarState>,
    existing_ui: Query<Entity, With<OmnibarUI>>,
) {
    // Remove existing UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if !state.active {
        return;
    }

    // Create omnibar container
    commands
        .spawn((
            OmnibarUI,
            Node {
                width: Val::Px(700.0),
                height: Val::Px(450.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                margin: UiRect {
                    left: Val::Px(-350.0), // Center with width/2
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            // Search input display
            let prefix_str = state
                .active_prefix
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("");
            let hint_suffix = if state.hint_mode {
                format!(" [hint: {}]", state.hint_input)
            } else {
                String::new()
            };

            parent.spawn((
                Text::new(format!("{}{}{}", prefix_str, state.query, hint_suffix)),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
                BorderRadius::all(Val::Px(4.0)),
            ));

            // Results list
            for (index, result) in state.results.iter().take(10).enumerate() {
                let is_selected = index == state.selected_index;
                let bg_color = if is_selected {
                    Color::srgba(0.3, 0.4, 0.6, 0.9)
                } else {
                    Color::srgba(0.2, 0.2, 0.2, 0.5)
                };

                let hint_key = if state.hint_mode && index < 12 {
                    format!("[{}] ", (b'a' + index as u8) as char)
                } else {
                    String::new()
                };

                parent
                    .spawn((
                        ResultItem { index },
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            margin: UiRect::bottom(Val::Px(3.0)),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(bg_color),
                        BorderRadius::all(Val::Px(4.0)),
                    ))
                    .with_children(|item| {
                        // First line: hint + icon + label
                        item.spawn((
                            Text::new(format!("{}{} {}", hint_key, result.icon, result.label)),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // Second line: description + provider
                        if let Some(desc) = &result.description {
                            let provider_name = result.provider_id.clone();
                            item.spawn((
                                Text::new(format!("{} - {}", desc, provider_name)),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                                Node {
                                    margin: UiRect::top(Val::Px(4.0)),
                                    ..default()
                                },
                            ));
                        }
                    });
            }

            // Help text at bottom
            parent.spawn((
                Text::new("Up/Down: Navigate  Enter: Select  Tab: Hints  Esc: Close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));
        });
}

/// Execute selected result
fn execute_action_system(
    mut events: EventReader<OmnibarExecuteEvent>,
    registry: Res<ProviderRegistry>,
    ipc: Res<IpcChannel>,
) {
    for event in events.read() {
        if let Some(provider) = registry.get(&event.result.provider_id) {
            info!(
                "Executing: {} from {}",
                event.result.label, event.result.provider_id
            );
            let mut ctx = OmnibarContext { ipc: &ipc };
            provider.execute(&event.result, &mut ctx);
        }
    }
}

/// Register default providers at startup
fn register_default_providers_system(mut registry: ResMut<ProviderRegistry>) {
    // Register commands provider
    registry.register(Arc::new(CommandsProvider::new()));

    // Register files provider
    registry.register(Arc::new(FilesProvider::new()));

    // Register history provider
    registry.register(Arc::new(HistoryProvider::new()));
}

// ============================================================================
// Built-in Providers
// ============================================================================

/// Commands provider (prefix: ">")
struct CommandsProvider {
    commands: Vec<CommandDef>,
    matcher: SkimMatcherV2,
}

#[derive(Clone)]
struct CommandDef {
    id: String,
    name: String,
    description: String,
    #[allow(dead_code)]
    category: String,
    action: Arc<dyn Fn(&IpcChannel) + Send + Sync>,
}

impl CommandsProvider {
    fn new() -> Self {
        let mut commands = Vec::new();

        // Clear terminal
        commands.push(CommandDef {
            id: "clear".to_string(),
            name: "Clear Terminal".to_string(),
            description: "Clear all terminal output".to_string(),
            category: "Terminal".to_string(),
            action: Arc::new(|ipc| {
                ipc.send(ControlMessage::Input { data: vec![0x0C] });
            }),
        });

        // Reset terminal
        commands.push(CommandDef {
            id: "reset".to_string(),
            name: "Reset Terminal".to_string(),
            description: "Reset terminal to initial state".to_string(),
            category: "Terminal".to_string(),
            action: Arc::new(|ipc| {
                ipc.send(ControlMessage::Input {
                    data: b"reset\n".to_vec(),
                });
            }),
        });

        // Interrupt
        commands.push(CommandDef {
            id: "interrupt".to_string(),
            name: "Interrupt Process".to_string(),
            description: "Send SIGINT to current process".to_string(),
            category: "Terminal".to_string(),
            action: Arc::new(|ipc| {
                ipc.send(ControlMessage::Input { data: vec![0x03] });
            }),
        });

        // Paste
        commands.push(CommandDef {
            id: "paste".to_string(),
            name: "Paste from Clipboard".to_string(),
            description: "Paste clipboard contents".to_string(),
            category: "Edit".to_string(),
            action: Arc::new(|ipc| {
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        ipc.send(ControlMessage::Input {
                            data: text.into_bytes(),
                        });
                    }
                }
            }),
        });

        Self {
            commands,
            matcher: SkimMatcherV2::default(),
        }
    }
}

impl OmnibarProvider for CommandsProvider {
    fn id(&self) -> &str {
        "commands"
    }

    fn name(&self) -> &str {
        "Commands"
    }

    fn icon(&self) -> &str {
        ">"
    }

    fn prefix(&self) -> Option<&str> {
        Some(">")
    }

    fn query(&self, query: &str, limit: usize) -> Vec<OmnibarResult> {
        let mut results: Vec<(CommandDef, i64)> = if query.is_empty() {
            self.commands.iter().map(|c| (c.clone(), 0)).collect()
        } else {
            self.commands
                .iter()
                .filter_map(|cmd| {
                    let name_score = self.matcher.fuzzy_match(&cmd.name, query).unwrap_or(0);
                    let desc_score = self
                        .matcher
                        .fuzzy_match(&cmd.description, query)
                        .unwrap_or(0);
                    let score = name_score.max(desc_score);

                    if score > 0 {
                        Some((cmd.clone(), score))
                    } else {
                        None
                    }
                })
                .collect()
        };

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(limit);

        results
            .into_iter()
            .map(|(cmd, score)| OmnibarResult {
                id: cmd.id.clone(),
                label: cmd.name.clone(),
                description: Some(cmd.description.clone()),
                icon: ">".to_string(),
                provider_id: self.id().to_string(),
                score,
                data: serde_json::json!({ "id": cmd.id }),
            })
            .collect()
    }

    fn execute(&self, result: &OmnibarResult, ctx: &mut OmnibarContext) {
        if let Some(cmd) = self.commands.iter().find(|c| c.id == result.id) {
            (cmd.action)(ctx.ipc);
        }
    }

    fn priority(&self) -> i32 {
        100
    }
}

/// Files provider (no prefix) - gitignore-aware file search
struct FilesProvider {
    matcher: SkimMatcherV2,
}

impl FilesProvider {
    fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    fn scan_files(&self, query: &str, limit: usize) -> Vec<(String, i64)> {
        use ignore::WalkBuilder;
        use std::path::Path;

        let mut results = Vec::new();
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Use ignore crate for gitignore-aware walking
        let walker = WalkBuilder::new(&cwd)
            .hidden(false) // Show hidden files
            .git_ignore(true) // Respect .gitignore
            .git_global(true) // Respect global gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .max_depth(Some(8)) // Limit depth for performance
            .build();

        let mut count = 0;
        let max_scan = 5000; // Limit total files scanned for performance

        for entry in walker.flatten() {
            if count >= max_scan {
                break;
            }

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            count += 1;

            // Get relative path for display
            let rel_path = path
                .strip_prefix(&cwd)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Skip empty paths
            if rel_path.is_empty() {
                continue;
            }

            let score = if query.is_empty() {
                // For empty query, prioritize recently modified and common files
                let extension_priority = match Path::new(&rel_path).extension().and_then(|e| e.to_str()) {
                    Some("rs") | Some("toml") | Some("md") => 10,
                    Some("ts") | Some("tsx") | Some("js") | Some("jsx") => 8,
                    Some("py") | Some("go") | Some("rb") => 7,
                    Some("json") | Some("yaml") | Some("yml") => 5,
                    _ => 1,
                };
                extension_priority
            } else {
                // Fuzzy match on filename and path
                let filename = Path::new(&rel_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let path_score = self.matcher.fuzzy_match(&rel_path, query).unwrap_or(0);
                let name_score = self.matcher.fuzzy_match(&filename, query).unwrap_or(0) * 2; // Boost filename matches

                path_score.max(name_score)
            };

            if score > 0 || query.is_empty() {
                results.push((rel_path, score));
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(limit);
        results
    }
}

impl OmnibarProvider for FilesProvider {
    fn id(&self) -> &str {
        "files"
    }

    fn name(&self) -> &str {
        "Files"
    }

    fn icon(&self) -> &str {
        "F"
    }

    fn prefix(&self) -> Option<&str> {
        None
    }

    fn query(&self, query: &str, limit: usize) -> Vec<OmnibarResult> {
        self.scan_files(query, limit)
            .into_iter()
            .map(|(file, score)| OmnibarResult {
                id: file.clone(),
                label: file.clone(),
                description: Some("File".to_string()),
                icon: "F".to_string(),
                provider_id: self.id().to_string(),
                score,
                data: serde_json::json!({ "path": file }),
            })
            .collect()
    }

    fn execute(&self, result: &OmnibarResult, ctx: &mut OmnibarContext) {
        // Open file in $EDITOR
        if let Some(path) = result.data["path"].as_str() {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            let cmd = format!("{} {}\n", editor, path);
            ctx.ipc.send(ControlMessage::Input {
                data: cmd.into_bytes(),
            });
        }
    }

    fn priority(&self) -> i32 {
        50
    }
}

/// History provider (prefix: "#") - reads real shell history
struct HistoryProvider {
    matcher: SkimMatcherV2,
}

impl HistoryProvider {
    fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    fn read_shell_history(&self, query: &str, limit: usize) -> Vec<(String, i64)> {
        use std::collections::HashSet;
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let mut results = Vec::new();
        let mut seen = HashSet::new();

        // Try to find shell history file
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));

        // Check for various shell history files in order of preference
        let history_files = [
            home.join(".zsh_history"),
            home.join(".bash_history"),
            home.join(".local/share/fish/fish_history"),
            home.join(".history"),
        ];

        for history_file in &history_files {
            if !history_file.exists() {
                continue;
            }

            if let Ok(file) = File::open(history_file) {
                let reader = BufReader::new(file);
                let lines: Vec<String> = reader
                    .lines()
                    .flatten()
                    .collect();

                // Process in reverse order (most recent first)
                for line in lines.into_iter().rev() {
                    // Skip empty lines
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Handle zsh extended history format (: timestamp:0;command)
                    let cmd = if line.starts_with(':') && line.contains(';') {
                        line.split(';').skip(1).collect::<Vec<_>>().join(";")
                    } else if line.starts_with("- cmd:") {
                        // Fish history format
                        line.strip_prefix("- cmd:").unwrap_or(line).trim().to_string()
                    } else {
                        line.to_string()
                    };

                    let cmd = cmd.trim().to_string();

                    // Skip duplicates and empty commands
                    if cmd.is_empty() || seen.contains(&cmd) {
                        continue;
                    }

                    seen.insert(cmd.clone());

                    let score = if query.is_empty() {
                        // For empty query, return most recent
                        (results.len() as i64 * -1) + 1000
                    } else {
                        self.matcher.fuzzy_match(&cmd, query).unwrap_or(0)
                    };

                    if score > 0 || query.is_empty() {
                        results.push((cmd, score));
                    }

                    // Limit scanning for performance
                    if results.len() >= 500 {
                        break;
                    }
                }

                // Found a history file, stop looking
                break;
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(limit);
        results
    }
}

impl OmnibarProvider for HistoryProvider {
    fn id(&self) -> &str {
        "history"
    }

    fn name(&self) -> &str {
        "History"
    }

    fn icon(&self) -> &str {
        "#"
    }

    fn prefix(&self) -> Option<&str> {
        Some("#")
    }

    fn query(&self, query: &str, limit: usize) -> Vec<OmnibarResult> {
        self.read_shell_history(query, limit)
            .into_iter()
            .map(|(cmd, score)| OmnibarResult {
                id: cmd.clone(),
                label: cmd.clone(),
                description: Some("Shell command".to_string()),
                icon: "#".to_string(),
                provider_id: self.id().to_string(),
                score,
                data: serde_json::json!({ "command": cmd }),
            })
            .collect()
    }

    fn execute(&self, result: &OmnibarResult, ctx: &mut OmnibarContext) {
        // Execute command from history
        if let Some(cmd) = result.data["command"].as_str() {
            let mut data = cmd.as_bytes().to_vec();
            data.push(b'\n');
            ctx.ipc.send(ControlMessage::Input { data });
        }
    }

    fn priority(&self) -> i32 {
        80
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_registry() {
        let mut registry = ProviderRegistry::default();
        registry.register(Arc::new(CommandsProvider::new()));

        assert!(registry.get("commands").is_some());
        assert!(registry.find_by_prefix(">").is_some());
    }

    #[test]
    fn test_commands_provider_fuzzy_search() {
        let provider = CommandsProvider::new();
        let results = provider.query("clear", 10);

        assert!(!results.is_empty());
        assert!(results[0].label.contains("Clear"));
    }

    #[test]
    fn test_files_provider() {
        let provider = FilesProvider::new();
        let results = provider.query("main", 10);

        assert!(!results.is_empty());
    }

    #[test]
    fn test_history_provider() {
        let provider = HistoryProvider::new();

        // Test with empty query - should return recent history (or empty if no history file)
        let results = provider.query("", 10);
        // History may be empty if no shell history file exists
        // Just verify no panic and results are properly formatted

        // If results exist, verify they have valid structure
        for result in &results {
            assert!(!result.id.is_empty());
            assert!(!result.label.is_empty());
            assert_eq!(result.provider_id, "history");
        }

        // Test fuzzy search works (may return empty if query not in history)
        let search_results = provider.query("ls", 10);
        for result in &search_results {
            assert_eq!(result.provider_id, "history");
        }
    }
}
