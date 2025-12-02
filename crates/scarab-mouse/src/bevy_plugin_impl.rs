/// System to render context menu UI
fn render_context_menu(
    plugin_state: Res<MousePluginState>,
    mut commands: Commands,
    menu_query: Query<Entity, With<ContextMenuEntity>>,
    windows: Query<&Window>,
) {
    // Clear existing menu entities
    for entity in &menu_query {
        commands.entity(entity).despawn_recursive();
    }

    // Get the active menu from plugin state
    let Some(menu) = &plugin_state.active_menu else {
        return;
    };

    // Get window for coordinate conversion
    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Calculate menu dimensions
    let menu_width = 280.0;
    let item_height = 40.0;
    let separator_height = 10.0;

    let total_height = menu.items.iter().fold(0.0, |acc, item| {
        acc + if item.separator {
            separator_height
        } else {
            item_height
        }
    });
    let menu_height = total_height.min(500.0);

    // Clamp menu position to screen bounds
    let menu_x = cursor_pos.x.min(window.width() - menu_width - 10.0);
    let menu_y = cursor_pos.y.min(window.height() - menu_height - 10.0);

    // Create context menu container
    commands
        .spawn((
            ContextMenuEntity,
            Node {
                width: Val::Px(menu_width),
                height: Val::Px(menu_height),
                position_type: PositionType::Absolute,
                left: Val::Px(menu_x),
                top: Val::Px(menu_y),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
            BorderColor(Color::srgba(0.4, 0.4, 0.5, 0.8)),
        ))
        .with_children(|parent| {
            // Render each menu item
            for (index, item) in menu.items.iter().enumerate() {
                if item.separator {
                    // Render separator
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(separator_height),
                            margin: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|sep| {
                            sep.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(1.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.5)),
                            ));
                        });
                } else {
                    // Render menu item
                    let is_selected = index == menu.selected_index;
                    let bg_color = if !item.enabled {
                        Color::srgba(0.15, 0.15, 0.15, 0.3)
                    } else if is_selected {
                        Color::srgba(0.35, 0.4, 0.5, 0.9)
                    } else {
                        Color::srgba(0.25, 0.25, 0.25, 0.2)
                    };

                    let text_color = if !item.enabled {
                        Color::srgba(0.5, 0.5, 0.5, 0.7)
                    } else {
                        Color::WHITE
                    };

                    parent
                        .spawn((
                            ContextMenuItem { index },
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(item_height),
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                margin: UiRect::bottom(Val::Px(2.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            BackgroundColor(bg_color),
                        ))
                        .with_children(|item_row| {
                            // Left side: label
                            item_row.spawn((
                                Text::new(&item.label),
                                TextFont {
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(text_color),
                            ));

                            // Right side: shortcut
                            if let Some(shortcut) = &item.shortcut {
                                item_row.spawn((
                                    Text::new(shortcut),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.6, 0.6, 0.7, 1.0)),
                                ));
                            }
                        });
                }
            }
        });
}

/// System to handle context menu keyboard input
fn handle_context_menu_input(
    mut plugin_state: ResMut<MousePluginState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    ipc: Option<Res<MouseIpcSender>>,
) {
    let state = plugin_state.shared_state.lock();

    if !state.context_menu_visible {
        return;
    }

    let Some(menu) = &mut plugin_state.active_menu else {
        return;
    };

    // Handle Escape - close menu
    if keyboard.just_pressed(KeyCode::Escape) {
        plugin_state.active_menu = None;
        plugin_state.menu_mouse_pos = None;
        drop(state);
        plugin_state.shared_state.lock().context_menu_visible = false;
        log::debug!("Context menu closed");
        return;
    }

    // Handle arrow key navigation
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        menu.select_next();
        log::trace!("Menu selection moved to index: {}", menu.selected_index);
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        menu.select_prev();
        log::trace!("Menu selection moved to index: {}", menu.selected_index);
    }

    // Handle Enter - execute selected item
    if keyboard.just_pressed(KeyCode::Enter) {
        if let Some(item) = menu.selected_item() {
            if item.enabled {
                let item_id = item.id.clone();
                log::info!("Executing context menu action: {}", item_id);

                // Extract menu context if available (for URL/file actions)
                let menu_context = plugin_state.menu_context.clone();

                // Close menu first
                plugin_state.active_menu = None;
                plugin_state.menu_mouse_pos = None;
                plugin_state.menu_context = None;
                drop(state);
                plugin_state.shared_state.lock().context_menu_visible = false;

                // Execute the menu action
                execute_menu_action(&item_id, &ipc, &mut plugin_state, menu_context);
            }
        }
    }
}

/// Execute a context menu action
fn execute_menu_action(
    action_id: &str,
    ipc: &Option<Res<MouseIpcSender>>,
    plugin_state: &mut ResMut<MousePluginState>,
    menu_context: Option<MenuContext>,
) {
    match action_id {
        "copy" => {
            // Copy selected text to clipboard
            let state = plugin_state.shared_state.lock();
            if let Some(selection) = &state.selection {
                // Extract text from selection using grid access
                // For now, we log the selection bounds
                log::info!("Copying selection: {:?}", selection);

                // TODO: Extract actual text from SharedState grid
                // This would require access to SharedState or a grid reader
                // For now, we'll just copy placeholder text
                use arboard::Clipboard;
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Err(e) = clipboard.set_text("[Selected Text]") {
                        log::error!("Failed to copy to clipboard: {}", e);
                    } else {
                        log::info!("Text copied to clipboard successfully");
                    }
                }
            } else {
                log::warn!("No selection to copy");
            }
        }
        "paste" => {
            // Paste from clipboard
            use arboard::Clipboard;
            if let Ok(mut clipboard) = Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    if let Some(ipc) = ipc {
                        ipc.0.send(ControlMessage::Input {
                            data: text.into_bytes(),
                        });
                        log::info!("Pasted {} bytes from clipboard", text.len());
                    }
                } else {
                    log::warn!("Failed to read from clipboard");
                }
            } else {
                log::error!("Failed to initialize clipboard");
            }
        }
        "select_all" => {
            // Select entire terminal buffer
            let mut state = plugin_state.shared_state.lock();

            // Create a selection spanning the entire visible grid
            // Using terminal dimensions from protocol (200x100 default)
            let cols = 200u16;
            let rows = 100u16;

            state.selection = Some(crate::selection::Selection {
                start: crate::types::Position { x: 0, y: 0 },
                end: crate::types::Position {
                    x: cols.saturating_sub(1),
                    y: rows.saturating_sub(1)
                },
                kind: crate::selection::SelectionKind::Normal,
            });

            log::info!("Selected all terminal content ({}x{})", cols, rows);
        }
        "clear_selection" => {
            // Clear current selection
            let mut state = plugin_state.shared_state.lock();
            if state.selection.is_some() {
                state.selection = None;
                log::info!("Selection cleared");
            } else {
                log::debug!("No selection to clear");
            }
        }
        "search" => {
            // Open search overlay
            // This would typically emit a Bevy event that the search overlay system listens for
            // For now, we log the intent
            log::info!("Search overlay requested");

            // TODO: Send SearchOverlayEvent or set a resource flag that search system monitors
            // Example: events.send(SearchOverlayEvent::Show);
        }
        "new_tab" => {
            // Create new terminal tab via IPC
            if let Some(ipc) = ipc {
                ipc.0.send(ControlMessage::TabCreate { title: None });
                log::info!("Sent TabCreate command to daemon");
            } else {
                log::error!("Cannot create new tab: IPC not available");
            }
        }
        "split_horizontal" => {
            // Split current pane horizontally via IPC
            if let Some(ipc) = ipc {
                // Assuming current pane ID is 0 (would need to track active pane)
                ipc.0.send(ControlMessage::PaneSplit {
                    pane_id: 0,
                    direction: scarab_protocol::SplitDirection::Horizontal,
                });
                log::info!("Sent PaneSplit horizontal command to daemon");
            } else {
                log::error!("Cannot split pane: IPC not available");
            }
        }
        "split_vertical" => {
            // Split current pane vertically via IPC
            if let Some(ipc) = ipc {
                // Assuming current pane ID is 0 (would need to track active pane)
                ipc.0.send(ControlMessage::PaneSplit {
                    pane_id: 0,
                    direction: scarab_protocol::SplitDirection::Vertical,
                });
                log::info!("Sent PaneSplit vertical command to daemon");
            } else {
                log::error!("Cannot split pane: IPC not available");
            }
        }
        "open_url" => {
            // Open URL in default browser
            if let Some(context) = menu_context {
                if let Some(url) = context.url {
                    log::info!("Opening URL: {}", url);

                    #[cfg(target_os = "linux")]
                    let cmd = "xdg-open";
                    #[cfg(target_os = "macos")]
                    let cmd = "open";
                    #[cfg(target_os = "windows")]
                    let cmd = "cmd";

                    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
                    {
                        log::error!("URL opening not supported on this platform");
                        return;
                    }

                    #[cfg(any(target_os = "linux", target_os = "macos"))]
                    {
                        match std::process::Command::new(cmd).arg(&url).spawn() {
                            Ok(_) => log::info!("Successfully opened URL in browser"),
                            Err(e) => log::error!("Failed to open URL: {}", e),
                        }
                    }

                    #[cfg(target_os = "windows")]
                    {
                        match std::process::Command::new(cmd)
                            .args(&["/c", "start", &url])
                            .spawn()
                        {
                            Ok(_) => log::info!("Successfully opened URL in browser"),
                            Err(e) => log::error!("Failed to open URL: {}", e),
                        }
                    }
                } else {
                    log::warn!("No URL available in menu context");
                }
            } else {
                log::warn!("No menu context available for URL action");
            }
        }
        "copy_url" => {
            // Copy URL to clipboard
            if let Some(context) = menu_context {
                if let Some(url) = context.url {
                    use arboard::Clipboard;
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Err(e) = clipboard.set_text(&url) {
                            log::error!("Failed to copy URL to clipboard: {}", e);
                        } else {
                            log::info!("URL copied to clipboard: {}", url);
                        }
                    } else {
                        log::error!("Failed to initialize clipboard");
                    }
                } else {
                    log::warn!("No URL available in menu context");
                }
            } else {
                log::warn!("No menu context available for URL copy action");
            }
        }
        "open_file" => {
            // Open file in default editor
            if let Some(context) = menu_context {
                if let Some(path) = context.file_path {
                    log::info!("Opening file: {}", path);

                    // Use $EDITOR environment variable or fallback to platform defaults
                    let editor = std::env::var("EDITOR").ok();

                    #[cfg(target_os = "linux")]
                    let default_cmd = "xdg-open";
                    #[cfg(target_os = "macos")]
                    let default_cmd = "open";
                    #[cfg(target_os = "windows")]
                    let default_cmd = "notepad";

                    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
                    {
                        log::error!("File opening not supported on this platform");
                        return;
                    }

                    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                    {
                        let cmd = editor.as_deref().unwrap_or(default_cmd);
                        match std::process::Command::new(cmd).arg(&path).spawn() {
                            Ok(_) => log::info!("Successfully opened file in editor"),
                            Err(e) => log::error!("Failed to open file: {}", e),
                        }
                    }
                } else {
                    log::warn!("No file path available in menu context");
                }
            } else {
                log::warn!("No menu context available for file action");
            }
        }
        "copy_path" => {
            // Copy file path to clipboard
            if let Some(context) = menu_context {
                if let Some(path) = context.file_path {
                    use arboard::Clipboard;
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Err(e) = clipboard.set_text(&path) {
                            log::error!("Failed to copy path to clipboard: {}", e);
                        } else {
                            log::info!("File path copied to clipboard: {}", path);
                        }
                    } else {
                        log::error!("Failed to initialize clipboard");
                    }
                } else {
                    log::warn!("No file path available in menu context");
                }
            } else {
                log::warn!("No menu context available for file path copy action");
            }
        }
        _ => {
            log::warn!("Unknown context menu action: {}", action_id);
        }
    }
}

/// Context information for menu actions
#[derive(Clone, Debug)]
struct MenuContext {
    url: Option<String>,
    file_path: Option<String>,
}
