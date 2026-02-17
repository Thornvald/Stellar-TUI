use super::theme;
use crate::app::App;
use crate::types::DialogKind;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

/// Draw the currently active modal dialog overlay.
pub fn draw_dialog(f: &mut Frame, area: Rect, app: &App) {
    let dialog = match &app.dialog {
        Some(d) => d,
        None => return,
    };

    match dialog {
        DialogKind::PathInput { label, value, .. } => {
            draw_path_input(f, area, label, value);
        }
        DialogKind::EnginePicker => {
            draw_engine_picker(f, area, app);
        }
        DialogKind::EditorTargetPicker {
            project_index,
            candidates,
            selected,
        } => {
            draw_editor_target_picker(f, area, app, *project_index, candidates, *selected);
        }
        DialogKind::Confirm { message, .. } => {
            draw_confirm(f, area, message);
        }
        DialogKind::Help => {
            draw_help(f, area);
        }
    }
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}

fn draw_path_input(f: &mut Frame, area: Rect, label: &str, value: &str) {
    let popup = centered_rect(60, 7, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(" {} ", label),
            theme::panel_title_style(),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    // Show the text input with a cursor
    let cursor_char = if (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 500)
        % 2
        == 0
    {
        "█"
    } else {
        " "
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(theme::ACCENT_WARM)),
            Span::styled(value, Style::default().fg(theme::TEXT)),
            Span::styled(cursor_char, Style::default().fg(theme::ACCENT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter]", theme::key_hint_style()),
            Span::styled(" Confirm  ", theme::footer_style()),
            Span::styled("[Esc]", theme::key_hint_style()),
            Span::styled(" Cancel", theme::footer_style()),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_engine_picker(f: &mut Frame, area: Rect, app: &App) {
    let height = (app.engines.len() as u16 * 2 + 6).min(area.height - 4);
    let popup = centered_rect(60, height, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " Select Engine ",
            theme::panel_title_style(),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let mut lines = vec![Line::from("")];

    for (i, engine) in app.engines.iter().enumerate() {
        let selected = i == app.engine_picker_index;
        let marker = if selected { " > " } else { "   " };
        let style = if selected {
            theme::selected_style().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT)
        };
        lines.push(Line::from(vec![
            Span::styled(marker, style),
            Span::styled(&engine.name, style),
        ]));
        lines.push(Line::from(vec![
            Span::raw("   "),
            Span::styled(&engine.path, Style::default().fg(theme::TEXT_DIM)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [Enter]", theme::key_hint_style()),
        Span::styled(" Select  ", theme::footer_style()),
        Span::styled("[m]", theme::key_hint_style()),
        Span::styled(" Manual  ", theme::footer_style()),
        Span::styled("[Esc]", theme::key_hint_style()),
        Span::styled(" Cancel", theme::footer_style()),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_editor_target_picker(
    f: &mut Frame,
    area: Rect,
    app: &App,
    project_index: usize,
    candidates: &[String],
    selected_index: usize,
) {
    let height = (candidates.len() as u16 + 8).min(area.height - 4).max(8);
    let popup = centered_rect(60, height, area);
    f.render_widget(Clear, popup);

    let project_name = app
        .config
        .projects
        .get(project_index)
        .map(|p| p.name.as_str())
        .unwrap_or("Project");

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(" Editor Target - {} ", project_name),
            theme::panel_title_style(),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let mut lines = vec![Line::from(vec![Span::styled(
        "  Pick the editor target to build.",
        Style::default().fg(theme::TEXT_DIM),
    )])];
    lines.push(Line::from(""));

    for (i, candidate) in candidates.iter().enumerate() {
        let selected = i == selected_index;
        let marker = if selected { " > " } else { "   " };
        let style = if selected {
            theme::selected_style().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT)
        };
        lines.push(Line::from(vec![
            Span::styled(marker, style),
            Span::styled(candidate, style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [Enter]", theme::key_hint_style()),
        Span::styled(" Select  ", theme::footer_style()),
        Span::styled("[m]", theme::key_hint_style()),
        Span::styled(" Manual  ", theme::footer_style()),
        Span::styled("[Esc]", theme::key_hint_style()),
        Span::styled(" Cancel", theme::footer_style()),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_confirm(f: &mut Frame, area: Rect, message: &str) {
    let popup = centered_rect(50, 7, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " Confirm ",
            theme::panel_title_style(),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", message),
            Style::default().fg(theme::TEXT),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [y]", theme::key_hint_style()),
            Span::styled(" Yes  ", theme::footer_style()),
            Span::styled("[n]", theme::key_hint_style()),
            Span::styled(" No", theme::footer_style()),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let popup = centered_rect(65, 22, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " Help - Stellar TUI ",
            theme::panel_title_style(),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border_style(true))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let hl = theme::key_hint_style();
    let nl = Style::default().fg(theme::TEXT);
    let dim = Style::default().fg(theme::TEXT_DIM);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  GLOBAL",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  ←/→ or Tab", hl),
            Span::styled("      Move focus between UI elements", nl),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓", hl),
            Span::styled("            Prev/next focus (outside logs)", nl),
        ]),
        Line::from(vec![
            Span::styled("  q", hl),
            Span::styled("              Quit", nl),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C", hl),
            Span::styled("         Force quit", nl),
        ]),
        Line::from(vec![
            Span::styled("  ?", hl),
            Span::styled("              Toggle help", nl),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  PROJECTS",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  Enter", hl),
            Span::styled("          Select focused project", nl),
        ]),
        Line::from(vec![
            Span::styled("  a", hl),
            Span::styled("              Add project (manual)", nl),
        ]),
        Line::from(vec![
            Span::styled("  f", hl),
            Span::styled("              Add project (file dialog)", nl),
        ]),
        Line::from(vec![
            Span::styled("  d", hl),
            Span::styled("              Remove selected project", nl),
        ]),
        Line::from(vec![
            Span::styled("  Del", hl),
            Span::styled("            Remove focused project", nl),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  ENGINE",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  e", hl),
            Span::styled("              Set engine path / pick", nl),
        ]),
        Line::from(vec![
            Span::styled("  r", hl),
            Span::styled("              Re-detect engines", nl),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  BUILD",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  b", hl),
            Span::styled("  Build  ", nl),
            Span::styled("n", hl),
            Span::styled("  Clean rebuild  ", nl),
            Span::styled("c", hl),
            Span::styled("  Cancel  ", nl),
            Span::styled("x", hl),
            Span::styled("  Clear logs", nl),
        ]),
        Line::from(vec![
            Span::styled("  Logs: ↑/↓", hl),
            Span::styled("      Up = older, Down = follow latest", nl),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Press any key to close", dim)),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}
