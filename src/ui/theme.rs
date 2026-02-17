#![allow(dead_code)]
use ratatui::style::{Color, Modifier, Style};

// ── Base palette ────────────────────────────────────────────────
pub const BG: Color = Color::Rgb(6, 6, 6);
pub const SURFACE: Color = Color::Rgb(18, 18, 18);
pub const SURFACE_ALT: Color = Color::Rgb(24, 24, 24);
pub const BORDER: Color = Color::Rgb(50, 50, 50);
pub const BORDER_FOCUS: Color = Color::Rgb(140, 140, 140);

pub const TEXT: Color = Color::Rgb(235, 235, 235);
pub const TEXT_DIM: Color = Color::Rgb(130, 130, 130);
pub const ACCENT: Color = Color::Rgb(235, 235, 235);
pub const ACCENT_WARM: Color = Color::Rgb(235, 235, 235);

pub const SUCCESS: Color = Color::Rgb(0, 255, 0);
pub const ERROR: Color = Color::Rgb(255, 50, 50);
pub const WARNING: Color = Color::Rgb(200, 200, 200);

pub const STAR_DIM: Color = Color::Rgb(60, 60, 70);
pub const STAR_MID: Color = Color::Rgb(130, 130, 150);
pub const STAR_BRIGHT: Color = Color::Rgb(220, 220, 240);

// ── Composite styles ────────────────────────────────────────────
pub fn title_style() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn eyebrow_style() -> Style {
    Style::default().fg(ACCENT_WARM)
}

pub fn subtitle_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn panel_title_style() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn key_hint_style() -> Style {
    Style::default()
        .fg(ACCENT_WARM)
        .add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().fg(TEXT).bg(Color::Rgb(40, 40, 45))
}

pub fn status_style(status: &crate::types::BuildState) -> Style {
    use crate::types::BuildState;
    match status {
        BuildState::Idle => Style::default().fg(TEXT_DIM),
        BuildState::Running => Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        BuildState::Success => Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
        BuildState::Error => Style::default().fg(ERROR).add_modifier(Modifier::BOLD),
        BuildState::Cancelled => Style::default().fg(WARNING).add_modifier(Modifier::BOLD),
    }
}

pub fn log_style(level: &crate::types::LogLevel) -> Style {
    use crate::types::LogLevel;
    match level {
        LogLevel::Info => Style::default().fg(Color::Rgb(216, 216, 216)),
        LogLevel::Warning => Style::default().fg(WARNING),
        LogLevel::Error => Style::default().fg(ERROR),
        LogLevel::Success => Style::default().fg(SUCCESS),
    }
}

pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(BORDER_FOCUS)
    } else {
        Style::default().fg(BORDER)
    }
}

pub fn footer_style() -> Style {
    Style::default().fg(TEXT_DIM)
}
