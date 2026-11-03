//! The settings window's colours and sizes, in the SDK's roles
//! (`rea_launcher_ui::theme`).
//!
//! A Welsh forest stage in the wet, like the hero photo: forest-shadow
//! green-grey for the base, wet stone for panels, lichen for lines, chalk
//! for text, the limestone of stage-marker boards for a chosen look, and
//! DiRT Rally's own orange kept for the one action that means "go": Play.
//! The tests below keep it within WCAG AA and the text-size rules.

use rea_launcher_ui::egui::Color32;
use rea_launcher_ui::theme::{Metrics, Palette, tint};

pub const BASE: Color32 = Color32::from_rgb(0x1f, 0x24, 0x20);
pub const PANEL: Color32 = Color32::from_rgb(0x2b, 0x31, 0x2c);
pub const LINE: Color32 = Color32::from_rgb(0x8b, 0x95, 0x8c);
pub const INK: Color32 = Color32::from_rgb(0xf1, 0xee, 0xe6);
pub const MUTED: Color32 = Color32::from_rgb(0xba, 0xc2, 0xb8);
/// DiRT Rally's orange: Play only.
pub const ACCENT: Color32 = Color32::from_rgb(0xf0, 0x7a, 0x28);
/// Limestone, for the chosen look and good news ("Found …").
pub const CHALK: Color32 = Color32::from_rgb(0xd8, 0xcf, 0xb8);
pub const ERROR: Color32 = Color32::from_rgb(0xff, 0x9c, 0x8a);

pub const PALETTE: Palette = Palette {
    base: BASE,
    panel: PANEL,
    line: LINE,
    ink: INK,
    muted: MUTED,
    action: ACCENT,
    action_hover: Color32::from_rgb(0xff, 0x93, 0x4a),
    on_action: Color32::from_rgb(0x1a, 0x12, 0x0b),
    select: CHALK,
    on_select: BASE,
    error: ERROR,
    disabled_ink: Color32::from_rgb(0x7d, 0x86, 0x7e),
    disabled_line: Color32::from_rgb(0x3a, 0x41, 0x3b),
    focus: INK,
    selection: tint(CHALK, 110),
};

pub const METRICS: Metrics = Metrics::DEFAULT;

#[cfg(test)]
mod tests {
    use super::*;
    use rea_launcher_ui::theme;

    #[test]
    fn the_palette_meets_wcag_aa() {
        assert_eq!(
            theme::contrast_problems(&PALETTE, &METRICS),
            Vec::<String>::new()
        );
    }

    #[test]
    fn no_text_is_too_small() {
        assert_eq!(theme::text_size_problems(&METRICS), Vec::<String>::new());
    }
}
