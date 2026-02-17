use serde::{Deserialize, Serialize};

/// A project entry in the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub path: String,
}

/// Top-level persisted config (compatible with the Tauri app's JSON format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub projects: Vec<ProjectConfig>,
    #[serde(rename = "unrealEnginePath")]
    pub unreal_engine_path: Option<String>,
    #[serde(rename = "selectedProjectPath", default)]
    pub selected_project_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            projects: vec![],
            unreal_engine_path: None,
            selected_project_path: None,
        }
    }
}

/// A detected Unreal Engine installation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EngineInstall {
    pub id: String,
    pub name: String,
    pub path: String,
    pub version: Option<String>,
}

/// The current state of a build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildState {
    Idle,
    Running,
    Success,
    Error,
    Cancelled,
}

impl std::fmt::Display for BuildState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildState::Idle => write!(f, "Idle"),
            BuildState::Running => write!(f, "Building..."),
            BuildState::Success => write!(f, "Build Succeeded"),
            BuildState::Error => write!(f, "Build Failed"),
            BuildState::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// A single line of build output with a severity hint.
#[derive(Debug, Clone)]
pub struct LogLine {
    pub text: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// A single focusable UI element in the linear navigation order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusItem {
    /// A project in the list (by index).
    Project(usize),
    /// The "Add Project" action.
    AddProject,
    /// The engine path panel.
    Engine,
    /// A build action button (by index into available_build_actions).
    BuildButton(usize),
    /// The log panel.
    Logs,
}

impl FocusItem {
    /// Which panel does this focus item belong to?
    pub fn panel(&self) -> FocusPanel {
        match self {
            FocusItem::Project(_) | FocusItem::AddProject => FocusPanel::Projects,
            FocusItem::Engine => FocusPanel::Engine,
            FocusItem::BuildButton(_) => FocusPanel::Build,
            FocusItem::Logs => FocusPanel::Logs,
        }
    }
}

/// Which panel is highlighted (derived from FocusItem).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Projects,
    Engine,
    Build,
    Logs,
}

/// Active modal dialog type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    /// Manual path text input (label, current text, callback target).
    PathInput {
        label: String,
        value: String,
        target: PathInputTarget,
    },
    /// Pick from a list of detected engine installs.
    EnginePicker,
    /// Confirm an action (message, confirmed action tag).
    Confirm {
        message: String,
        action: ConfirmAction,
    },
    /// Help overlay.
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathInputTarget {
    AddProject,
    SetEnginePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    RemoveProject(usize),
}
