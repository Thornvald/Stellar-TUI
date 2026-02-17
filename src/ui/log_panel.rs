use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use crate::app::App;
use crate::types::FocusPanel;
use super::theme;

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

    // Auto-scroll: if enabled, keep scroll at bottom
    let scroll = if app.auto_scroll_logs {
        total.saturating_sub(visible_height)
    } else {
        app.log_scroll.min(total.saturating_sub(visible_height))
    };

    let end = (scroll + visible_height).min(total);
    let visible_logs = &app.logs[scroll..end];

    let lines: Vec<Line> = visible_logs
        .iter()
        .map(|log| {
            Line::from(vec![
                Span::styled(
                    " > ",
                    ratatui::style::Style::default().fg(theme::TEXT_DIM),
                ),
                Span::styled(&log.text, theme::log_style(&log.level)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}
