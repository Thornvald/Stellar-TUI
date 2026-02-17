use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use crate::app::App;
use crate::types::{FocusItem, FocusPanel};
use super::theme;

pub fn draw_engine_panel(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focused_panel() == FocusPanel::Engine;
    let item_focused = app.focus == FocusItem::Engine;

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" UNREAL ENGINE PATH ", theme::panel_title_style()),
            if focused {
                Span::styled("[r]edetect ", theme::key_hint_style())
            } else {
                Span::raw("")
            },
        ]))
        .borders(Borders::ALL)
        .border_style(theme::border_style(focused))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let path_display = match &app.config.unreal_engine_path {
        Some(p) => {
            let max_len = inner.width.saturating_sub(6) as usize;
            if p.len() > max_len && max_len > 5 {
                format!("...{}", &p[p.len() - (max_len - 5)..])
            } else {
                p.clone()
            }
        }
        None => "Not set".to_string(),
    };

    let marker = if item_focused { " > " } else { "   " };
    let style = if item_focused {
        theme::selected_style().add_modifier(Modifier::BOLD)
    } else if app.config.unreal_engine_path.is_some() {
        Style::default().fg(theme::TEXT)
    } else {
        Style::default().fg(theme::TEXT_DIM)
    };

    let mut lines = vec![Line::from(vec![
        Span::styled(marker, style),
        Span::styled(path_display, style),
        if item_focused {
            Span::styled("  [Enter] edit", theme::key_hint_style())
        } else {
            Span::raw("")
        },
    ])];

    if !app.engines.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("   {} engine(s) detected", app.engines.len()),
            Style::default().fg(theme::TEXT_DIM),
        )));
    }

    f.render_widget(Paragraph::new(lines), inner);
}
