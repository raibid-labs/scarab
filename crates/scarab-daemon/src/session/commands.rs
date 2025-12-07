use super::pane::PaneId;
use super::tab::SplitDirection as SessionSplitDirection;
use super::{ClientId, SessionManager};
use anyhow::Result;
use scarab_protocol::{
    ControlMessage, DaemonMessage, PaneInfo, SessionInfo, SessionResponse,
    SplitDirection as ProtocolSplitDirection, TabInfo,
};
use std::sync::Arc;

/// Handle session-related control messages
pub async fn handle_session_command(
    msg: ControlMessage,
    session_manager: &Arc<SessionManager>,
    client_id: ClientId,
) -> Result<Option<SessionResponse>> {
    match msg {
        ControlMessage::SessionCreate { name } => {
            log::info!("Client {} creating session: {}", client_id, name);

            match session_manager.create_session(name.to_string(), 80, 24) {
                Ok(id) => {
                    let session = session_manager.get_session(&id).unwrap();
                    Ok(Some(SessionResponse::Created {
                        id: id.to_string(),
                        name: session.name.clone(),
                    }))
                }
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to create session: {}", e),
                })),
            }
        }

        ControlMessage::SessionDelete { id } => {
            log::info!("Client {} deleting session: {}", client_id, id);

            match session_manager.delete_session(&id.to_string()) {
                Ok(_) => Ok(Some(SessionResponse::Deleted { id: id.clone() })),
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to delete session: {}", e),
                })),
            }
        }

        ControlMessage::SessionList => {
            log::info!("Client {} listing sessions", client_id);

            let sessions = session_manager.list_sessions();
            let session_infos: Vec<SessionInfo> = sessions
                .into_iter()
                .map(
                    |(id, name, created_at, last_attached, attached_clients)| SessionInfo {
                        id,
                        name,
                        created_at,
                        last_attached,
                        attached_clients: attached_clients as u32,
                    },
                )
                .collect();

            Ok(Some(SessionResponse::List {
                sessions: session_infos,
            }))
        }

        ControlMessage::SessionAttach { id } => {
            log::info!("Client {} attaching to session: {}", client_id, id);

            match session_manager.attach_client(&id.to_string(), client_id) {
                Ok(_) => Ok(Some(SessionResponse::Attached { id: id.clone() })),
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to attach to session: {}", e),
                })),
            }
        }

        ControlMessage::SessionDetach { id } => {
            log::info!("Client {} detaching from session: {}", client_id, id);

            match session_manager.detach_client(&id.to_string(), client_id) {
                Ok(_) => Ok(Some(SessionResponse::Detached { id: id.clone() })),
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to detach from session: {}", e),
                })),
            }
        }

        ControlMessage::SessionRename { id, new_name } => {
            log::info!(
                "Client {} renaming session {} to {}",
                client_id,
                id,
                new_name
            );

            match session_manager.rename_session(&id.to_string(), new_name.to_string()) {
                Ok(_) => Ok(Some(SessionResponse::Renamed {
                    id: id.clone(),
                    new_name: new_name.clone(),
                })),
                Err(e) => Ok(Some(SessionResponse::Error {
                    message: format!("Failed to rename session: {}", e),
                })),
            }
        }

        _ => {
            // Not a session command
            Ok(None)
        }
    }
}

/// Result from tab command including any destroyed pane IDs
pub struct TabCommandResult {
    pub message: Option<DaemonMessage>,
    pub destroyed_pane_ids: Vec<PaneId>,
}

/// Handle tab-related control messages
/// Returns a DaemonMessage response for the client and any destroyed pane IDs
pub async fn handle_tab_command(
    msg: ControlMessage,
    session_manager: &Arc<SessionManager>,
    client_id: ClientId,
) -> Result<Option<TabCommandResult>> {
    // Get the default/active session for this client
    let session = match session_manager.get_default_session() {
        Some(s) => s,
        None => {
            return Ok(Some(TabCommandResult {
                message: Some(DaemonMessage::Session(SessionResponse::Error {
                    message: "No active session".to_string(),
                })),
                destroyed_pane_ids: Vec::new(),
            }));
        }
    };

    match msg {
        ControlMessage::TabCreate { title } => {
            log::info!("Client {} creating tab: {:?}", client_id, title);

            match session.create_tab(title.map(|s| s.to_string())) {
                Ok(tab_id) => {
                    let tabs = session.list_tabs();
                    let tab_info = tabs.iter().find(|(id, _, _, _)| *id == tab_id);

                    if let Some((id, title, is_active, pane_count)) = tab_info {
                        Ok(Some(TabCommandResult {
                            message: Some(DaemonMessage::TabCreated {
                                tab: TabInfo {
                                    id: *id,
                                    title: title.clone(),
                                    session_id: Some(session.id.clone()),
                                    is_active: *is_active,
                                    pane_count: *pane_count as u32,
                                },
                            }),
                            destroyed_pane_ids: Vec::new(),
                        }))
                    } else {
                        Ok(None)
                    }
                }
                Err(e) => Ok(Some(TabCommandResult {
                    message: Some(DaemonMessage::Session(SessionResponse::Error {
                        message: format!("Failed to create tab: {}", e),
                    })),
                    destroyed_pane_ids: Vec::new(),
                })),
            }
        }

        ControlMessage::TabClose { tab_id } => {
            log::info!("Client {} closing tab: {}", client_id, tab_id);

            match session.close_tab(tab_id) {
                Ok(destroyed_panes) => Ok(Some(TabCommandResult {
                    message: Some(DaemonMessage::TabClosed { tab_id }),
                    destroyed_pane_ids: destroyed_panes,
                })),
                Err(e) => Ok(Some(TabCommandResult {
                    message: Some(DaemonMessage::Session(SessionResponse::Error {
                        message: format!("Failed to close tab: {}", e),
                    })),
                    destroyed_pane_ids: Vec::new(),
                })),
            }
        }

        ControlMessage::TabSwitch { tab_id } => {
            log::info!("Client {} switching to tab: {}", client_id, tab_id);

            match session.switch_tab(tab_id) {
                Ok(_) => Ok(Some(TabCommandResult {
                    message: Some(DaemonMessage::TabSwitched { tab_id }),
                    destroyed_pane_ids: Vec::new(),
                })),
                Err(e) => Ok(Some(TabCommandResult {
                    message: Some(DaemonMessage::Session(SessionResponse::Error {
                        message: format!("Failed to switch tab: {}", e),
                    })),
                    destroyed_pane_ids: Vec::new(),
                })),
            }
        }

        ControlMessage::TabRename { tab_id, new_title } => {
            log::info!(
                "Client {} renaming tab {} to {}",
                client_id,
                tab_id,
                new_title
            );

            match session.rename_tab(tab_id, new_title.to_string()) {
                Ok(_) => {
                    // Return updated tab list
                    let tabs = session.list_tabs();
                    let tab_infos: Vec<TabInfo> = tabs
                        .into_iter()
                        .map(|(id, title, is_active, pane_count)| TabInfo {
                            id,
                            title,
                            session_id: Some(session.id.clone()),
                            is_active,
                            pane_count: pane_count as u32,
                        })
                        .collect();
                    Ok(Some(TabCommandResult {
                        message: Some(DaemonMessage::TabListResponse { tabs: tab_infos }),
                        destroyed_pane_ids: Vec::new(),
                    }))
                }
                Err(e) => Ok(Some(TabCommandResult {
                    message: Some(DaemonMessage::Session(SessionResponse::Error {
                        message: format!("Failed to rename tab: {}", e),
                    })),
                    destroyed_pane_ids: Vec::new(),
                })),
            }
        }

        ControlMessage::TabList => {
            log::info!("Client {} listing tabs", client_id);

            let tabs = session.list_tabs();
            let tab_infos: Vec<TabInfo> = tabs
                .into_iter()
                .map(|(id, title, is_active, pane_count)| TabInfo {
                    id,
                    title,
                    session_id: Some(session.id.clone()),
                    is_active,
                    pane_count: pane_count as u32,
                })
                .collect();

            Ok(Some(TabCommandResult {
                message: Some(DaemonMessage::TabListResponse { tabs: tab_infos }),
                destroyed_pane_ids: Vec::new(),
            }))
        }

        _ => Ok(None),
    }
}

/// Handle pane-related control messages
/// Returns a DaemonMessage response for the client
pub async fn handle_pane_command(
    msg: ControlMessage,
    session_manager: &Arc<SessionManager>,
    client_id: ClientId,
) -> Result<Option<DaemonMessage>> {
    // Get the default/active session for this client
    let session = match session_manager.get_default_session() {
        Some(s) => s,
        None => {
            return Ok(Some(DaemonMessage::Session(SessionResponse::Error {
                message: "No active session".to_string(),
            })));
        }
    };

    match msg {
        ControlMessage::PaneSplit {
            pane_id: _,
            direction,
        } => {
            log::info!("Client {} splitting pane: {:?}", client_id, direction);

            // Convert protocol direction to session direction
            let session_direction = match direction {
                ProtocolSplitDirection::Horizontal => SessionSplitDirection::Horizontal,
                ProtocolSplitDirection::Vertical => SessionSplitDirection::Vertical,
            };

            match session.split_pane(session_direction) {
                Ok(new_pane_id) => {
                    // Get the new pane info
                    if let Some(pane) = session.get_active_pane() {
                        Ok(Some(DaemonMessage::PaneCreated {
                            pane: PaneInfo {
                                id: new_pane_id,
                                x: pane.viewport.x,
                                y: pane.viewport.y,
                                width: pane.viewport.width,
                                height: pane.viewport.height,
                                is_focused: true,
                            },
                        }))
                    } else {
                        Ok(None)
                    }
                }
                Err(e) => Ok(Some(DaemonMessage::Session(SessionResponse::Error {
                    message: format!("Failed to split pane: {}", e),
                }))),
            }
        }

        ControlMessage::PaneClose { pane_id } => {
            log::info!("Client {} closing pane: {}", client_id, pane_id);

            match session.close_pane(pane_id) {
                Ok(_) => Ok(Some(DaemonMessage::PaneClosed { pane_id })),
                Err(e) => Ok(Some(DaemonMessage::Session(SessionResponse::Error {
                    message: format!("Failed to close pane: {}", e),
                }))),
            }
        }

        ControlMessage::PaneFocus { pane_id } => {
            log::info!("Client {} focusing pane: {}", client_id, pane_id);

            match session.focus_pane(pane_id) {
                Ok(_) => Ok(Some(DaemonMessage::PaneFocused { pane_id })),
                Err(e) => Ok(Some(DaemonMessage::Session(SessionResponse::Error {
                    message: format!("Failed to focus pane: {}", e),
                }))),
            }
        }

        ControlMessage::PaneResize {
            pane_id: _,
            width,
            height,
        } => {
            log::info!("Client {} resizing pane to {}x{}", client_id, width, height);

            match session.resize(width, height) {
                Ok(_) => {
                    // Return updated layout
                    if let Some(pane) = session.get_active_pane() {
                        Ok(Some(DaemonMessage::PaneLayoutUpdate {
                            panes: vec![PaneInfo {
                                id: pane.id,
                                x: pane.viewport.x,
                                y: pane.viewport.y,
                                width: pane.viewport.width,
                                height: pane.viewport.height,
                                is_focused: true,
                            }],
                        }))
                    } else {
                        Ok(None)
                    }
                }
                Err(e) => Ok(Some(DaemonMessage::Session(SessionResponse::Error {
                    message: format!("Failed to resize pane: {}", e),
                }))),
            }
        }

        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_session_commands() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let manager = Arc::new(SessionManager::new(db_path).unwrap());

        // Create session
        let response = handle_session_command(
            ControlMessage::SessionCreate {
                name: "test".to_string().into(),
            },
            &manager,
            1,
        )
        .await
        .unwrap();

        assert!(matches!(response, Some(SessionResponse::Created { .. })));

        // List sessions
        let response = handle_session_command(ControlMessage::SessionList, &manager, 1)
            .await
            .unwrap();

        if let Some(SessionResponse::List { sessions }) = response {
            assert_eq!(sessions.len(), 1);
        } else {
            panic!("Expected List response");
        }
    }
}
