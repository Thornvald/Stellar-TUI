use super::{build_controls, engine_panel, header, log_panel, projects};
use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

/// Draws the full two-column layout with header and footer.
pub fn draw_layout(f: &mut Frame, area: Rect, app: &App) {
    // Vertical: header | body | footer
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // header
            Constraint::Min(10),   // body
            Constraint::Length(1), // footer
        ])
        .split(area);

    header::draw_header(f, vert[0], app);
    draw_body(f, vert[1], app);
    draw_footer(f, vert[2], app);
}

fn draw_body(f: &mut Frame, area: Rect, app: &App) {
    // Two columns: left (projects) | right (engine + build + logs)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left column: projects panel
    projects::draw_projects(f, cols[0], app);

    // Right column: split into engine / build controls / logs
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // engine path
            Constraint::Length(5), // build controls
            Constraint::Min(5),    // logs
        ])
        .split(cols[1]);

    engine_panel::draw_engine_panel(f, right[0], app);
    build_controls::draw_build_controls(f, right[1], app);
    log_panel::draw_log_panel(f, right[2], app);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    use super::theme;
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    // Show flash message if active, otherwise normal footer
    if let Some(msg) = &app.flash_message {
        if app.tick < app.flash_until {
            let footer = Line::from(vec![Span::styled(
                format!(" {} ", msg),
                ratatui::style::Style::default()
                    .fg(theme::SUCCESS)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )]);
            f.render_widget(Paragraph::new(footer), area);
            return;
        }
    }

    let footer = Line::from(vec![
        Span::styled(" [←→]", theme::key_hint_style()),
        Span::styled(" Navigate UI  ", theme::footer_style()),
        Span::styled("[↑]", theme::key_hint_style()),
        Span::styled(" Older logs  ", theme::footer_style()),
        Span::styled("[↓]", theme::key_hint_style()),
        Span::styled(" Follow latest  ", theme::footer_style()),
        Span::styled("[Enter]", theme::key_hint_style()),
        Span::styled(" Select  ", theme::footer_style()),
        Span::styled("[?]", theme::key_hint_style()),
        Span::styled(" Help  ", theme::footer_style()),
        Span::styled("[q]", theme::key_hint_style()),
        Span::styled(" Quit", theme::footer_style()),
        Span::styled(
            "                                     Stellar TUI",
            theme::footer_style(),
        ),
    ]);

    f.render_widget(Paragraph::new(footer), area);
}
