use super::theme;
use crate::app::App;
use crate::types::FocusPanel;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn draw_build_controls(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focused_panel() == FocusPanel::Build;

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " BUILD ",
            theme::panel_title_style(),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border_style(focused))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Status line with spinner
    let status_text = match &app.build_state {
        crate::types::BuildState::Running => {
            let frame = SPINNER_FRAMES[app.tick as usize / 3 % SPINNER_FRAMES.len()];
            format!("  {} STATUS: {}", frame, app.build_state)
        }
        _ => format!("  STATUS: {}", app.build_state),
    };

    let actions = app.available_build_actions();
    let focused_btn = app.focused_build_button();

    // Build button spans
    let mut button_spans = vec![Span::raw("  ")];
    for (i, &label) in actions.iter().enumerate() {
        let is_selected = focused_btn == Some(i);
        let btn_style = if is_selected {
            Style::default()
                .fg(theme::SURFACE)
                .bg(theme::TEXT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::ACCENT).bg(theme::SURFACE_ALT)
        };

        let shortcut = match label {
            "Build" => "b",
            "Clean Rebuild" => "n",
            "Cancel" => "c",
            "Clear" => "x",
            "Copy Log" => "y",
            _ => "",
        };

        if is_selected {
            button_spans.push(Span::styled(
                format!(" > {} ({}) ", label, shortcut),
                btn_style,
            ));
        } else {
            button_spans.push(Span::styled(
                format!("  {} ({})  ", label, shortcut),
                btn_style,
            ));
        }
        button_spans.push(Span::raw(" "));
    }

    let lines = vec![
        Line::from(Span::styled(
            status_text,
            theme::status_style(&app.build_state),
        )),
        Line::from(""),
        Line::from(button_spans),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}
