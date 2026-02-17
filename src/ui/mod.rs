pub mod build_controls;
pub mod dialogs;
pub mod engine_panel;
pub mod header;
pub mod layout;
pub mod log_panel;
pub mod projects;
pub mod starfield;
pub mod theme;

use crate::app::App;
use ratatui::Frame;

/// Master render function: draws starfield, layout, panels, then modal overlay.
pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    // Layer 0: starfield background
    starfield::draw_starfield(f, area, app.tick);

    // Layer 1: main layout with panels
    layout::draw_layout(f, area, app);

    // Layer 2: modal dialog overlay (if any)
    if app.dialog.is_some() {
        dialogs::draw_dialog(f, area, app);
    }
}
