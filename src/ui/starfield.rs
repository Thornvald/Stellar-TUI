use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;
use ratatui::Frame;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn draw_starfield(f: &mut Frame, area: Rect, tick: u64) {
    let widget = StarfieldWidget { tick };
    f.render_widget(widget, area);
}

struct StarfieldWidget {
    tick: u64,
}

impl Widget for StarfieldWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let mut rng = StdRng::seed_from_u64(42_424_242);
        let cell_count = (area.width as usize) * (area.height as usize);
        let count = (cell_count / 8).max(40).min(500);
        let t = self.tick as f64;

        for _ in 0..count {
            let layer: u8 = if rng.gen_bool(0.4) {
                0
            } else if rng.gen_bool(0.5) {
                1
            } else {
                2
            };
            let base_brightness: f64 = match layer {
                0 => rng.gen_range(0.15..0.45),
                1 => rng.gen_range(0.35..0.65),
                _ => rng.gen_range(0.55..0.90),
            };
            let x_frac: f64 = rng.gen_range(0.0..1.0);
            let y_frac: f64 = rng.gen_range(0.0..1.0);
            let twinkle_speed: f64 = rng.gen_range(0.03..0.12);
            let twinkle_offset: f64 = rng.gen_range(0.0..std::f64::consts::TAU);

            let col = (x_frac * area.width as f64) as u16;
            let row = (y_frac * area.height as f64) as u16;

            if col >= area.width || row >= area.height {
                continue;
            }

            // Twinkle: sinusoidal brightness modulation
            let twinkle = (t * twinkle_speed + twinkle_offset).sin() * 0.5 + 0.5;
            let brightness = base_brightness * (0.4 + 0.6 * twinkle);

            let (ch, color) = star_appearance(layer, brightness);

            let cell = &mut buf[(area.x + col, area.y + row)];
            cell.set_char(ch);
            cell.set_style(Style::default().fg(color));
        }
    }
}

fn star_appearance(layer: u8, brightness: f64) -> (char, Color) {
    let b = (brightness * 255.0).clamp(0.0, 255.0) as u8;
    let ch = match layer {
        0 => if brightness > 0.35 { '∙' } else { '·' },
        1 => if brightness > 0.55 { '•' } else { '∙' },
        _ => if brightness > 0.8 { '✦' } else if brightness > 0.6 { '*' } else { '•' },
    };
    (ch, Color::Rgb(b, b, b))
}
