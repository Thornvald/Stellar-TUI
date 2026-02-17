use crate::build::{BuildHandle, BuildMode};
use crate::config;
use crate::engine;
use crate::types::*;
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Top-level application state.
pub struct App {
    pub config: Config,
    pub focus: FocusItem,
    pub selected_project: Option<usize>,
    pub engines: Vec<EngineInstall>,
    pub engine_picker_index: usize,
    pub build_state: BuildState,
    pub logs: Vec<LogLine>,
    pub log_scroll: usize,
    pub dialog: Option<DialogKind>,
    pub should_quit: bool,
    pub tick: u64,
    pub build_handle: Option<BuildHandle>,
    pub log_rx: Option<mpsc::UnboundedReceiver<String>>,
    pub auto_scroll_logs: bool,
    /// Brief status message shown in footer (e.g. "Copied!"), auto-clears.
    pub flash_message: Option<String>,
    pub flash_until: u64,
}

impl App {
    pub fn new() -> Self {
        let mut cfg = config::load_config();
        let engines = engine::detect_engines();
        let selected_project = cfg
            .selected_project_path
            .as_ref()
            .and_then(|path| cfg.projects.iter().position(|p| p.path == *path))
            .or_else(|| {
                if cfg.projects.is_empty() {
                    None
                } else {
                    Some(0)
                }
            });
        cfg.selected_project_path = selected_project
            .and_then(|i| cfg.projects.get(i))
            .map(|p| p.path.clone());

        let initial_focus = match selected_project {
            Some(i) => FocusItem::Project(i),
            None => FocusItem::AddProject,
        };
        Self {
            config: cfg,
            focus: initial_focus,
            selected_project,
            engines,
            engine_picker_index: 0,
            build_state: BuildState::Idle,
            logs: Vec::new(),
            log_scroll: 0,
            dialog: None,
            should_quit: false,
            tick: 0,
            build_handle: None,
            log_rx: None,
            auto_scroll_logs: true,
            flash_message: None,
            flash_until: 0,
        }
    }

    /// Which panel is currently focused (derived from focus item).
    pub fn focused_panel(&self) -> FocusPanel {
        self.focus.panel()
    }

    /// Build the ordered list of all focusable items given current state.
    pub fn focus_items(&self) -> Vec<FocusItem> {
        let mut items = Vec::new();

        // Projects
        if self.config.projects.is_empty() {
            items.push(FocusItem::AddProject);
        } else {
            for i in 0..self.config.projects.len() {
                items.push(FocusItem::Project(i));
            }
            items.push(FocusItem::AddProject);
        }

        // Engine
        items.push(FocusItem::Engine);

        // Build buttons
        let actions = self.available_build_actions();
        for i in 0..actions.len() {
            items.push(FocusItem::BuildButton(i));
        }

        // Logs
        items.push(FocusItem::Logs);

        items
    }

    /// Move focus to the next item in the linear order.
    pub fn focus_next(&mut self) {
        let items = self.focus_items();
        if let Some(pos) = items.iter().position(|i| i == &self.focus) {
            let next = (pos + 1) % items.len();
            self.focus = items[next].clone();
        } else {
            // Focus got stale, reset
            self.focus = items.first().cloned().unwrap_or(FocusItem::Engine);
        }
    }

    /// Move focus to the previous item in the linear order.
    pub fn focus_prev(&mut self) {
        let items = self.focus_items();
        if let Some(pos) = items.iter().position(|i| i == &self.focus) {
            let prev = (pos + items.len() - 1) % items.len();
            self.focus = items[prev].clone();
        } else {
            self.focus = items.last().cloned().unwrap_or(FocusItem::Engine);
        }
    }

    pub fn focus_next_panel(&mut self) {
        match self.focused_panel() {
            FocusPanel::Projects => self.focus = FocusItem::Engine,
            FocusPanel::Engine => self.focus = FocusItem::BuildButton(0),
            FocusPanel::Build => {
                self.focus = FocusItem::Logs;
                self.follow_latest_logs();
            }
            FocusPanel::Logs => self.focus = self.projects_anchor_item(),
        }
    }

    pub fn focus_prev_panel(&mut self) {
        match self.focused_panel() {
            FocusPanel::Projects => self.focus = FocusItem::Logs,
            FocusPanel::Engine => self.focus = self.projects_anchor_item(),
            FocusPanel::Build => self.focus = FocusItem::Engine,
            FocusPanel::Logs => self.focus = FocusItem::BuildButton(0),
        }

        if self.focus == FocusItem::Logs {
            self.follow_latest_logs();
        }
    }

    pub fn follow_latest_logs(&mut self) {
        self.auto_scroll_logs = true;
        self.log_scroll = self.logs.len().saturating_sub(1);
    }

    fn projects_anchor_item(&self) -> FocusItem {
        if self.config.projects.is_empty() {
            FocusItem::AddProject
        } else if let FocusItem::Project(i) = self.focus {
            if i < self.config.projects.len() {
                FocusItem::Project(i)
            } else {
                FocusItem::Project(0)
            }
        } else if let Some(i) = self.selected_project {
            FocusItem::Project(i.min(self.config.projects.len() - 1))
        } else {
            FocusItem::Project(0)
        }
    }

    /// Get the selected project index (if any).
    pub fn selected_project_index(&self) -> Option<usize> {
        self.selected_project
    }

    /// Get the currently focused build button index (if any).
    pub fn focused_build_button(&self) -> Option<usize> {
        match &self.focus {
            FocusItem::BuildButton(i) => Some(*i),
            _ => None,
        }
    }

    pub fn save_config(&self) {
        let _ = config::save_config(&self.config);
    }

    pub fn selected_project(&self) -> Option<&ProjectConfig> {
        self.selected_project_index()
            .and_then(|i| self.config.projects.get(i))
    }

    pub fn add_project(&mut self, path: String) {
        let pb = PathBuf::from(&path);
        let name = pb
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".into());
        self.config.projects.push(ProjectConfig {
            name,
            path: path.clone(),
            editor_target: None,
        });
        // Focus the newly added project
        let idx = self.config.projects.len() - 1;
        self.selected_project = Some(idx);
        self.config.selected_project_path = Some(path);
        self.focus = FocusItem::Project(idx);
        self.save_config();
    }

    pub fn select_project(&mut self, index: usize) {
        if index < self.config.projects.len() {
            self.selected_project = Some(index);
            self.config.selected_project_path = Some(self.config.projects[index].path.clone());
            self.save_config();
            self.flash_message = Some(format!(
                "Selected project: {}",
                self.config.projects[index].name
            ));
            self.flash_until = self.tick + 60;
        }
    }

    pub fn remove_project(&mut self, index: usize) {
        if index < self.config.projects.len() {
            self.config.projects.remove(index);
            self.selected_project = match self.selected_project {
                None => None,
                Some(_) if self.config.projects.is_empty() => None,
                Some(selected) if selected == index => {
                    if index >= self.config.projects.len() {
                        Some(self.config.projects.len() - 1)
                    } else {
                        Some(index)
                    }
                }
                Some(selected) if selected > index => Some(selected - 1),
                Some(selected) => Some(selected),
            };
            self.config.selected_project_path = self
                .selected_project
                .and_then(|i| self.config.projects.get(i))
                .map(|p| p.path.clone());
            if self.config.projects.is_empty() {
                self.focus = FocusItem::AddProject;
            } else {
                let new_idx = if index >= self.config.projects.len() {
                    self.config.projects.len() - 1
                } else {
                    index
                };
                self.focus = FocusItem::Project(new_idx);
            }
            self.save_config();
        }
    }

    pub fn set_engine_path(&mut self, path: String) {
        self.config.unreal_engine_path = Some(path);
        self.save_config();
    }

    pub fn pick_engine(&mut self, index: usize) {
        if let Some(install) = self.engines.get(index) {
            self.set_engine_path(install.path.clone());
        }
    }

    pub fn re_detect_engines(&mut self) {
        self.engines = engine::detect_engines();
        self.engine_picker_index = 0;
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
        self.log_scroll = 0;
        self.auto_scroll_logs = true;
    }

    pub fn push_log(&mut self, text: String) {
        let text = sanitize_log_text(&text);
        if text.is_empty() {
            return;
        }
        let level = classify_log_line(&text);
        self.logs.push(LogLine { text, level });
        if self.logs.len() > 10_000 {
            self.logs.drain(0..1000);
            self.log_scroll = self.log_scroll.saturating_sub(1000);
        }
        if self.auto_scroll_logs {
            self.log_scroll = self.logs.len().saturating_sub(1);
        }
    }

    /// Returns the list of available build action labels based on current state.
    pub fn available_build_actions(&self) -> Vec<&'static str> {
        let mut actions = Vec::new();
        match self.build_state {
            BuildState::Running => {
                actions.push("Cancel");
            }
            BuildState::Idle => {
                actions.push("Build");
                actions.push("Clean Rebuild");
            }
            _ => {
                actions.push("Build");
                actions.push("Clean Rebuild");
                actions.push("Clear");
            }
        }
        if !self.logs.is_empty() {
            actions.push("Copy Log");
        }
        actions
    }

    /// Execute whichever build action is currently selected.
    pub fn activate_build_button(&mut self, index: usize) {
        let actions = self.available_build_actions();
        if let Some(&label) = actions.get(index) {
            match label {
                "Build" => self.start_build(),
                "Clean Rebuild" => self.start_clean_rebuild(),
                "Cancel" => self.cancel_build(),
                "Clear" => {
                    self.clear_logs();
                    self.build_state = BuildState::Idle;
                    self.focus = FocusItem::BuildButton(0);
                }
                "Copy Log" => self.copy_logs(),
                _ => {}
            }
        }
    }

    pub fn copy_logs(&mut self) {
        if self.logs.is_empty() {
            self.flash_message = Some("No logs to copy.".into());
            self.flash_until = self.tick + 60;
            return;
        }
        let text: String = self
            .logs
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
            Ok(_) => {
                self.flash_message = Some("Copied to clipboard!".into());
            }
            Err(e) => {
                self.flash_message = Some(format!("Copy failed: {}", e));
            }
        }
        self.flash_until = self.tick + 60;
    }

    pub fn start_build(&mut self) {
        self.start_build_with_mode(BuildMode::Standard);
    }

    pub fn start_clean_rebuild(&mut self) {
        self.start_build_with_mode(BuildMode::CleanRebuild);
    }

    fn start_build_with_mode(&mut self, mode: BuildMode) {
        let project = match self.selected_project() {
            Some(p) => p.clone(),
            None => {
                self.push_log(
                    "No project selected. Select one in Projects and press Enter.".into(),
                );
                return;
            }
        };
        let engine_path = match &self.config.unreal_engine_path {
            Some(p) => p.clone(),
            None => {
                self.push_log("No engine path set.".into());
                return;
            }
        };

        self.clear_logs();
        self.build_state = BuildState::Running;
        self.auto_scroll_logs = true;

        let (tx, rx) = mpsc::unbounded_channel();
        self.log_rx = Some(rx);

        match crate::build::spawn_build(
            project.path.clone(),
            engine_path,
            project.editor_target.clone(),
            tx,
            mode,
        ) {
            Ok(handle) => {
                self.build_handle = Some(handle);
            }
            Err(e) => {
                self.push_log(format!("Failed to start build: {}", e));
                if crate::build::is_ambiguous_target_error(&e) {
                    let _ = self.prompt_editor_target_resolution(
                        "Multiple editor targets were detected. Choose one and build again.",
                    );
                }
                self.build_state = BuildState::Error;
                self.log_rx = None;
            }
        }
    }

    pub fn cancel_build(&mut self) {
        if self.build_state != BuildState::Running {
            return;
        }
        if let Some(handle) = self.build_handle.take() {
            handle.cancel();
        }
        self.build_state = BuildState::Cancelled;
        self.push_log("Build cancelled by user.".into());
    }

    /// Called every tick to drain log messages and check build completion.
    pub fn poll_build(&mut self) {
        let mut lines = Vec::new();
        let mut disconnected = false;
        if let Some(rx) = &mut self.log_rx {
            loop {
                match rx.try_recv() {
                    Ok(line) => lines.push(line),
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }
        for line in lines {
            self.push_log(line);
        }
        if disconnected {
            self.log_rx = None;
        }

        if self.build_state == BuildState::Running {
            let finished = self.build_handle.as_ref().and_then(|h| h.try_finished());
            if let Some(success) = finished {
                self.build_state = if success {
                    BuildState::Success
                } else {
                    BuildState::Error
                };
                if success {
                    self.push_log("Build completed successfully.".into());
                    crate::notify::on_build_success();
                } else {
                    self.push_log("Build finished with errors.".into());
                    crate::notify::on_build_failed();
                    if self
                        .logs
                        .iter()
                        .rev()
                        .take(200)
                        .any(|l| crate::build::looks_like_target_error(&l.text))
                    {
                        let _ = self.prompt_editor_target_resolution(
                            "Build failed with a target-related error. Choose the correct editor target and rebuild.",
                        );
                    }
                }
                self.follow_latest_logs();
                self.build_handle = None;
            }
        }
    }

    fn set_editor_target(&mut self, project_index: usize, editor_target: String) -> bool {
        let trimmed = editor_target.trim().to_string();
        if trimmed.is_empty() {
            return false;
        }
        if let Some(project) = self.config.projects.get_mut(project_index) {
            let project_name = project.name.clone();
            project.editor_target = Some(trimmed.clone());
            self.save_config();
            self.flash_message = Some(format!(
                "Editor target for {} set to {}",
                project_name, trimmed
            ));
            self.flash_until = self.tick + 90;
            return true;
        }
        false
    }

    fn prompt_editor_target_resolution(&mut self, reason: &str) -> bool {
        let Some(project_index) = self.selected_project_index() else {
            return false;
        };
        let Some(project) = self.config.projects.get(project_index) else {
            return false;
        };

        let candidates = match crate::build::discover_editor_targets(&project.path) {
            Ok(v) => v,
            Err(e) => {
                self.push_log(format!("Failed to detect editor targets: {}", e));
                return false;
            }
        };

        if candidates.len() == 1 {
            if self.set_editor_target(project_index, candidates[0].clone()) {
                self.push_log(format!("{} (auto-selected: {}).", reason, candidates[0]));
                return true;
            }
            return false;
        }

        if !candidates.is_empty() {
            self.dialog = Some(DialogKind::EditorTargetPicker {
                project_index,
                candidates,
                selected: 0,
            });
            self.push_log(reason.to_string());
            return true;
        }

        self.dialog = Some(DialogKind::PathInput {
            label: "Set Editor Target (e.g. MyGameEditor)".into(),
            value: String::new(),
            target: PathInputTarget::SetEditorTarget(project_index),
        });
        self.push_log(format!(
            "{} No *Editor.Target.cs files were found, so enter the target manually.",
            reason
        ));
        true
    }

    pub fn open_add_project_dialog(&mut self) {
        self.dialog = Some(DialogKind::PathInput {
            label: "Add Project (.uproject path)".into(),
            value: String::new(),
            target: PathInputTarget::AddProject,
        });
    }

    pub fn open_add_project_file_dialog(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Unreal Project", &["uproject"])
            .pick_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            self.add_project(path_str);
        }
    }

    pub fn open_set_engine_dialog(&mut self) {
        if !self.engines.is_empty() {
            self.engine_picker_index = 0;
            self.dialog = Some(DialogKind::EnginePicker);
        } else {
            self.dialog = Some(DialogKind::PathInput {
                label: "Set Unreal Engine Path".into(),
                value: self.config.unreal_engine_path.clone().unwrap_or_default(),
                target: PathInputTarget::SetEnginePath,
            });
        }
    }

    pub fn open_help(&mut self) {
        self.dialog = Some(DialogKind::Help);
    }

    pub fn close_dialog(&mut self) {
        self.dialog = None;
    }

    pub fn confirm_dialog(&mut self) {
        let dialog = match self.dialog.take() {
            Some(d) => d,
            None => return,
        };
        match dialog {
            DialogKind::PathInput { value, target, .. } => {
                let trimmed = value.trim().to_string();
                if !trimmed.is_empty() {
                    match target {
                        PathInputTarget::AddProject => self.add_project(trimmed),
                        PathInputTarget::SetEnginePath => self.set_engine_path(trimmed),
                        PathInputTarget::SetEditorTarget(project_index) => {
                            let _ = self.set_editor_target(project_index, trimmed);
                        }
                    }
                }
            }
            DialogKind::EnginePicker => {
                self.pick_engine(self.engine_picker_index);
            }
            DialogKind::EditorTargetPicker {
                project_index,
                candidates,
                selected,
            } => {
                if let Some(choice) = candidates.get(selected) {
                    let _ = self.set_editor_target(project_index, choice.clone());
                }
            }
            DialogKind::Confirm { action, .. } => match action {
                ConfirmAction::RemoveProject(idx) => self.remove_project(idx),
            },
            DialogKind::Help => {}
        }
    }
}

fn classify_log_line(line: &str) -> LogLevel {
    let lower = line.to_lowercase();
    if lower.contains("error") || lower.contains("fatal") {
        LogLevel::Error
    } else if lower.contains("warning") || lower.contains("warn") {
        LogLevel::Warning
    } else if lower.contains("success") || lower.contains("complete") {
        LogLevel::Success
    } else {
        LogLevel::Info
    }
}

fn sanitize_log_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                let _ = chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
                continue;
            }
            continue;
        }

        match ch {
            '\r' | '\n' => out.push(' '),
            '\t' => out.push_str("    "),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }

    out.trim_end().to_string()
}
