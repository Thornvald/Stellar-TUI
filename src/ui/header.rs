use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use super::theme;

const SPARKLE_CHARS: &[char] = &['.', '+', '*', '+', '.', ' '];

pub fn draw_header(f: &mut Frame, area: Rect, app: &crate::app::App) {
    let tick = app.tick as usize;

    // Sparkle animation: cycle through characters at different phases
    let left_sparkle = SPARKLE_CHARS[tick / 4 % SPARKLE_CHARS.len()];
    let right_sparkle = SPARKLE_CHARS[(tick / 4 + 3) % SPARKLE_CHARS.len()];

    let lines = vec![
        Line::from(vec![
            Span::styled("  UNREAL BUILD DESK", theme::eyebrow_style()),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  {} S t e l l a r {}", left_sparkle, right_sparkle),
                theme::title_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Build Unreal projects from your terminal.",
                theme::subtitle_style(),
            ),
        ]),
        Line::from(""),
    ];

    f.render_widget(Paragraph::new(lines), area);
}
