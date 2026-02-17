use super::theme;
use crate::app::App;
use crate::types::FocusPanel;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_log_panel(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focused_panel() == FocusPanel::Logs;

    let dot_style = ratatui::style::Style::default().fg(theme::TEXT_DIM);
    let title_spans = vec![
        Span::styled(" ", dot_style),
        Span::styled("● ", dot_style),
        Span::styled("● ", dot_style),
        Span::styled("● ", dot_style),
        Span::styled("BUILD LOG ", theme::panel_title_style()),
    ];

    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_style(theme::border_style(focused))
        .style(ratatui::style::Style::default().bg(ratatui::style::Color::Rgb(7, 7, 7)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.logs.is_empty() {
        let empty = Paragraph::new(vec![Line::from(Span::styled(
            "  > No logs yet.",
            ratatui::style::Style::default()
                .fg(theme::TEXT_DIM)
                .add_modifier(ratatui::style::Modifier::ITALIC),
        ))]);
        f.render_widget(empty, inner);
        return;
    }

    let visible_height = inner.height as usize;
    let total = app.logs.len();

    let max_top = total.saturating_sub(visible_height);

    // auto_scroll_logs=true: always follow latest lines.
    // auto_scroll_logs=false: app.log_scroll acts like a cursor at the bottom edge,
    // then we derive the top line from that cursor so Up/Down move immediately.
    let scroll = if app.auto_scroll_logs {
        max_top
    } else {
        app.log_scroll
            .saturating_sub(visible_height.saturating_sub(1))
            .min(max_top)
    };

    let end = (scroll + visible_height).min(total);
    let visible_logs = &app.logs[scroll..end];

    let lines: Vec<Line> = visible_logs
        .iter()
        .map(|log| {
            Line::from(vec![
                Span::styled(" > ", ratatui::style::Style::default().fg(theme::TEXT_DIM)),
                Span::styled(&log.text, theme::log_style(&log.level)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}
