//! Visual style matching the Character Forge: a dark, warm parchment-on-ember
//! look with terracotta accents, Cinzel small-caps headings and EB Garamond
//! body text.

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
    Visuals,
};
use std::sync::Arc;

pub const BG: Color32 = Color32::from_rgb(0x1b, 0x18, 0x15);
pub const PANEL: Color32 = Color32::from_rgb(0x21, 0x1d, 0x1a);
pub const CARD: Color32 = Color32::from_rgb(0x24, 0x20, 0x1c);
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x2b, 0x26, 0x22);
pub const BORDER: Color32 = Color32::from_rgb(0x3b, 0x34, 0x2e);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(0x2e, 0x29, 0x25);
pub const ACCENT: Color32 = Color32::from_rgb(0xc7, 0x5c, 0x4b);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xd6, 0x6b, 0x59);
pub const ACCENT_TEXT: Color32 = Color32::from_rgb(0xd2, 0x6f, 0x5a);
pub const TEXT: Color32 = Color32::from_rgb(0xef, 0xe6, 0xd6);
pub const TEXT_SOFT: Color32 = Color32::from_rgb(0xd8, 0xcc, 0xb8);
pub const MUTED: Color32 = Color32::from_rgb(0x9a, 0x8f, 0x82);
pub const GREEN: Color32 = Color32::from_rgb(0x4f, 0xb5, 0x6f);
pub const INPUT: Color32 = Color32::from_rgb(0x2a, 0x25, 0x21);

pub const CINZEL: &str = "cinzel";
pub const CINZEL_BOLD: &str = "cinzel-bold";
pub const GARAMOND: &str = "garamond";
pub const GARAMOND_ITALIC: &str = "garamond-italic";

pub fn family(name: &'static str) -> FontFamily {
    FontFamily::Name(name.into())
}

pub fn heading(size: f32) -> FontId {
    FontId::new(size, family(CINZEL_BOLD))
}

pub fn title(size: f32) -> FontId {
    FontId::new(size, family(CINZEL))
}

pub fn body(size: f32) -> FontId {
    FontId::new(size, family(GARAMOND))
}

pub fn italic(size: f32) -> FontId {
    FontId::new(size, family(GARAMOND_ITALIC))
}

pub fn ui(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        CINZEL.into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Cinzel-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        CINZEL_BOLD.into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Cinzel-Bold.ttf"
        ))),
    );
    fonts.font_data.insert(
        GARAMOND.into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/EBGaramond.ttf"
        ))),
    );
    fonts.font_data.insert(
        GARAMOND_ITALIC.into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/EBGaramond-Italic.ttf"
        ))),
    );
    // Each custom family falls back to the default proportional fonts so
    // symbols and non-Latin glyphs still render.
    let fallback: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for name in [CINZEL, CINZEL_BOLD, GARAMOND, GARAMOND_ITALIC] {
        let mut list = vec![name.to_owned()];
        list.extend(fallback.iter().cloned());
        fonts.families.insert(family(name), list);
    }
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, heading(24.0)),
        (TextStyle::Body, body(18.0)),
        (TextStyle::Button, ui(15.0)),
        (TextStyle::Small, ui(12.5)),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        ),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    style.spacing.slider_width = 160.0;
    style.spacing.interact_size.y = 30.0;

    let mut v = Visuals::dark();
    v.override_text_color = Some(TEXT);
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.extreme_bg_color = INPUT;
    v.faint_bg_color = CARD;
    v.window_stroke = Stroke::new(1.0_f32, BORDER);
    v.selection.bg_fill = ACCENT.gamma_multiply(0.55);
    v.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    v.hyperlink_color = ACCENT_TEXT;
    v.widgets.noninteractive.bg_fill = PANEL;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER_SOFT);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_SOFT);
    v.widgets.inactive.bg_fill = INPUT;
    v.widgets.inactive.weak_bg_fill = INPUT;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    v.widgets.hovered.bg_fill = CARD_HOVER;
    v.widgets.hovered.weak_bg_fill = CARD_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, MUTED);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    v.widgets.active.bg_fill = ACCENT.gamma_multiply(0.7);
    v.widgets.active.weak_bg_fill = ACCENT.gamma_multiply(0.7);
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(8);
        w.expansion = 0.0;
    }
    v.slider_trailing_fill = true;
    style.visuals = v;
    ctx.set_style(style);
}

/// A 64x64 window icon drawn procedurally: ember-red diamond on dark ground,
/// so the executable needs no image file at all.
pub fn icon() -> egui::IconData {
    let n = 64usize;
    let mut rgba = vec![0u8; n * n * 4];
    let c = (n as f32 - 1.0) / 2.0;
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f32 - c).abs();
            let dy = (y as f32 - c).abs();
            let r = ((x as f32 - c).powi(2) + (y as f32 - c).powi(2)).sqrt();
            let i = (y * n + x) * 4;
            let (col, a) = if r > c {
                (BG, 0)
            } else if dx + dy < 20.0 {
                (ACCENT, 255)
            } else if dx + dy < 23.0 {
                (TEXT, 255)
            } else {
                (BG, 255)
            };
            rgba[i] = col.r();
            rgba[i + 1] = col.g();
            rgba[i + 2] = col.b();
            rgba[i + 3] = a;
        }
    }
    egui::IconData {
        rgba,
        width: n as u32,
        height: n as u32,
    }
}
