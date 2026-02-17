use crate::app::App;
use crate::types::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // If a dialog is open, route input there
    if app.dialog.is_some() {
        handle_dialog_key(app, key);
        return;
    }

    // Shift+Tab
    if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT) {
        app.focus_prev_panel();
        return;
    }

    // Global keys
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('?') => {
            app.open_help();
            return;
        }
        // Arrow keys: panel navigation, with build-button horizontal navigation.
        KeyCode::Right | KeyCode::Tab => {
            if matches!(app.focus, FocusItem::BuildButton(_)) {
                app.focus_next();
            } else {
                app.focus_next_panel();
            }
            return;
        }
        KeyCode::Left | KeyCode::BackTab => {
            if matches!(app.focus, FocusItem::BuildButton(_)) {
                app.focus_prev();
            } else {
                app.focus_prev_panel();
            }
            return;
        }
        KeyCode::Down => {
            if app.focus == FocusItem::Logs {
                if app.auto_scroll_logs {
                    app.follow_latest_logs();
                } else {
                    app.log_scroll = app.log_scroll.saturating_add(1);
                    if app.log_scroll >= app.logs.len().saturating_sub(1) {
                        app.follow_latest_logs();
                    }
                }
            } else {
                app.focus_next();
            }
            return;
        }
        KeyCode::Up => {
            if app.focus == FocusItem::Logs {
                if app.auto_scroll_logs {
                    app.log_scroll = app.logs.len().saturating_sub(2);
                } else {
                    app.log_scroll = app.log_scroll.saturating_sub(1);
                }
                app.auto_scroll_logs = false;
            } else {
                app.focus_prev();
            }
            return;
        }
        _ => {}
    }

    // Context-specific keys based on what's focused
    match &app.focus.clone() {
        FocusItem::Project(idx) => handle_project_key(app, key, *idx),
        FocusItem::AddProject => handle_add_project_key(app, key),
        FocusItem::Engine => handle_engine_key(app, key),
        FocusItem::BuildButton(idx) => handle_build_button_key(app, key, *idx),
        FocusItem::Logs => handle_logs_key(app, key),
    }
}

fn handle_project_key(app: &mut App, key: KeyEvent, index: usize) {
    match key.code {
        KeyCode::Enter => {
            app.select_project(index);
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            let name = app
                .config
                .projects
                .get(index)
                .map(|p| p.name.clone())
                .unwrap_or_default();
            app.dialog = Some(DialogKind::Confirm {
                message: format!("Remove project \"{}\"?", name),
                action: ConfirmAction::RemoveProject(index),
            });
        }
        KeyCode::Char('a') => {
            app.open_add_project_dialog();
        }
        KeyCode::Char('f') => {
            app.open_add_project_file_dialog();
        }
        _ => {}
    }
}

fn handle_add_project_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.open_add_project_file_dialog();
        }
        KeyCode::Char('a') => {
            app.open_add_project_dialog();
        }
        KeyCode::Char('f') => {
            app.open_add_project_file_dialog();
        }
        _ => {}
    }
}

fn handle_engine_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter | KeyCode::Char('e') => {
            app.open_set_engine_dialog();
        }
        KeyCode::Char('r') => {
            app.re_detect_engines();
        }
        _ => {}
    }
}

fn handle_build_button_key(app: &mut App, key: KeyEvent, index: usize) {
    match key.code {
        KeyCode::Enter => {
            app.activate_build_button(index);
        }
        // Shortcut keys still work
        KeyCode::Char('b') => {
            if app.build_state != BuildState::Running {
                app.start_build();
            }
        }
        KeyCode::Char('n') => {
            if app.build_state != BuildState::Running {
                app.start_clean_rebuild();
            }
        }
        KeyCode::Char('c') => {
            app.cancel_build();
        }
        KeyCode::Char('x') => {
            if app.build_state != BuildState::Running {
                app.clear_logs();
                app.build_state = BuildState::Idle;
                app.focus = FocusItem::BuildButton(0);
            }
        }
        KeyCode::Char('y') => {
            app.copy_logs();
        }
        _ => {}
    }
}

fn handle_logs_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') => {
            app.log_scroll = app.log_scroll.saturating_add(1);
            app.auto_scroll_logs = false;
        }
        KeyCode::Char('k') => {
            app.log_scroll = app.log_scroll.saturating_sub(1);
            app.auto_scroll_logs = false;
        }
        KeyCode::Char('g') => {
            app.log_scroll = 0;
            app.auto_scroll_logs = false;
        }
        KeyCode::Char('G') => {
            app.log_scroll = app.logs.len().saturating_sub(1);
            app.auto_scroll_logs = true;
        }
        KeyCode::PageDown => {
            app.log_scroll = app.log_scroll.saturating_add(20);
            app.auto_scroll_logs = false;
        }
        KeyCode::PageUp => {
            app.log_scroll = app.log_scroll.saturating_sub(20);
            app.auto_scroll_logs = false;
        }
        KeyCode::Char('x') => {
            if app.build_state != BuildState::Running {
                app.clear_logs();
                app.build_state = BuildState::Idle;
            }
        }
        KeyCode::Char('y') => {
            app.copy_logs();
        }
        _ => {}
    }
}

fn handle_dialog_key(app: &mut App, key: KeyEvent) {
    match &app.dialog {
        Some(DialogKind::PathInput { .. }) => handle_path_input_key(app, key),
        Some(DialogKind::EnginePicker) => handle_engine_picker_key(app, key),
        Some(DialogKind::EditorTargetPicker { .. }) => handle_editor_target_picker_key(app, key),
        Some(DialogKind::Confirm { .. }) => handle_confirm_key(app, key),
        Some(DialogKind::Help) => {
            app.close_dialog();
        }
        None => {}
    }
}

fn handle_path_input_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_dialog(),
        KeyCode::Enter => app.confirm_dialog(),
        KeyCode::Backspace => {
            if let Some(DialogKind::PathInput { value, .. }) = &mut app.dialog {
                value.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Some(DialogKind::PathInput { value, .. }) = &mut app.dialog {
                value.push(c);
            }
        }
        _ => {}
    }
}

fn handle_engine_picker_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_dialog(),
        KeyCode::Enter => app.confirm_dialog(),
        KeyCode::Char('j') | KeyCode::Down => {
            let len = app.engines.len();
            if len > 0 {
                app.engine_picker_index = (app.engine_picker_index + 1) % len;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let len = app.engines.len();
            if len > 0 {
                app.engine_picker_index = (app.engine_picker_index + len - 1) % len;
            }
        }
        KeyCode::Char('m') => {
            app.dialog = Some(DialogKind::PathInput {
                label: "Set Unreal Engine Path".into(),
                value: app.config.unreal_engine_path.clone().unwrap_or_default(),
                target: PathInputTarget::SetEnginePath,
            });
        }
        _ => {}
    }
}

fn handle_editor_target_picker_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.close_dialog(),
        KeyCode::Enter => app.confirm_dialog(),
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(DialogKind::EditorTargetPicker {
                candidates,
                selected,
                ..
            }) = &mut app.dialog
            {
                let len = candidates.len();
                if len > 0 {
                    *selected = (*selected + 1) % len;
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(DialogKind::EditorTargetPicker {
                candidates,
                selected,
                ..
            }) = &mut app.dialog
            {
                let len = candidates.len();
                if len > 0 {
                    *selected = (*selected + len - 1) % len;
                }
            }
        }
        KeyCode::Char('m') => {
            if let Some(DialogKind::EditorTargetPicker { project_index, .. }) = &app.dialog {
                let value = app
                    .config
                    .projects
                    .get(*project_index)
                    .and_then(|p| p.editor_target.clone())
                    .unwrap_or_default();
                app.dialog = Some(DialogKind::PathInput {
                    label: "Set Editor Target (e.g. MyGameEditor)".into(),
                    value,
                    target: PathInputTarget::SetEditorTarget(*project_index),
                });
            }
        }
        _ => {}
    }
}

fn handle_confirm_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => app.confirm_dialog(),
        KeyCode::Char('n') | KeyCode::Esc => app.close_dialog(),
        _ => {}
    }
}
