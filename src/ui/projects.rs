use super::theme;
use crate::app::App;
use crate::types::{FocusItem, FocusPanel};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_projects(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focused_panel() == FocusPanel::Projects;

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " PROJECTS ",
            theme::panel_title_style(),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border_style(focused))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();

    if app.config.projects.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  No projects yet.",
            theme::subtitle_style(),
        )));
        lines.push(Line::from(""));
    } else {
        for (i, project) in app.config.projects.iter().enumerate() {
            let is_focused = app.focus == FocusItem::Project(i);
            let is_selected = app.selected_project_index() == Some(i);
            let marker = if is_focused {
                " > "
            } else if is_selected {
                " * "
            } else {
                "   "
            };
            let name_style = if is_focused {
                theme::selected_style().add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT)
            };

            let max_path_len = inner.width.saturating_sub(6) as usize;
            let path_display = truncate_path(&project.path, max_path_len);

            lines.push(Line::from(vec![
                Span::styled(marker, name_style),
                Span::styled(&project.name, name_style),
                if is_focused {
                    Span::styled(
                        "  [Enter] select  [Del]/[d] remove",
                        theme::key_hint_style(),
                    )
                } else {
                    Span::raw("")
                },
            ]));
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(path_display, Style::default().fg(theme::TEXT_DIM)),
            ]));

            if i < app.config.projects.len() - 1 {
                lines.push(Line::from(""));
            }
        }
        lines.push(Line::from(""));
    }

    // "Add Project" item
    let add_focused = app.focus == FocusItem::AddProject;
    let add_style = if add_focused {
        Style::default()
            .fg(theme::SURFACE)
            .bg(theme::TEXT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::ACCENT)
    };
    let add_marker = if add_focused { " > " } else { "   " };
    lines.push(Line::from(vec![
        Span::styled(add_marker, add_style),
        Span::styled("+ Add Project", add_style),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else if max_len > 5 {
        format!("...{}", &path[path.len() - (max_len - 3)..])
    } else {
        path[..max_len].to_string()
    }
}
