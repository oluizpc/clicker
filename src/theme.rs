//! Identidade visual: painel em ardósia com um único acento âmbar.
//! Tudo que é cor/espaçamento/raio vive aqui, para a UI ficar coesa.

use egui::{Color32, FontFamily, FontId, Margin, Rounding, Stroke, TextStyle};

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub surface2: Color32,
    pub stroke: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub danger: Color32,
}

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: Color32::from_rgb(0x14, 0x16, 0x1B),
            surface: Color32::from_rgb(0x1C, 0x1F, 0x27),
            surface2: Color32::from_rgb(0x24, 0x28, 0x33),
            stroke: Color32::from_rgb(0x2E, 0x33, 0x40),
            text: Color32::from_rgb(0xE7, 0xE9, 0xF0),
            muted: Color32::from_rgb(0x86, 0x8D, 0xA0),
            accent: Color32::from_rgb(0xF5, 0xA5, 0x24),
            on_accent: Color32::from_rgb(0x19, 0x14, 0x05),
            danger: Color32::from_rgb(0xE4, 0x5A, 0x4A),
        }
    } else {
        Palette {
            bg: Color32::from_rgb(0xF2, 0xF3, 0xF6),
            surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            surface2: Color32::from_rgb(0xE9, 0xEC, 0xF1),
            stroke: Color32::from_rgb(0xD6, 0xDA, 0xE2),
            text: Color32::from_rgb(0x1A, 0x1D, 0x24),
            muted: Color32::from_rgb(0x6A, 0x71, 0x80),
            accent: Color32::from_rgb(0xD6, 0x84, 0x00),
            on_accent: Color32::from_rgb(0xFF, 0xF7, 0xEA),
            danger: Color32::from_rgb(0xC6, 0x3B, 0x2D),
        }
    }
}

/// Moldura padrão de cartão: superfície + borda fina, sem sombra.
pub fn card(p: &Palette) -> egui::Frame {
    egui::Frame {
        fill: p.surface,
        rounding: Rounding::same(10.0),
        stroke: Stroke::new(1.0_f32, p.stroke),
        inner_margin: Margin::same(14.0),
        outer_margin: Margin::ZERO,
        shadow: egui::epaint::Shadow::NONE,
    }
}

pub fn install(ctx: &egui::Context, p: &Palette) {
    let mut style = (*ctx.style()).clone();
    let mut v = if p.text.r() > 128 && p.bg.r() < 128 {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    v.panel_fill = p.bg;
    v.window_fill = p.surface;
    v.faint_bg_color = p.surface;
    v.extreme_bg_color = p.surface2;
    v.override_text_color = Some(p.text);
    v.hyperlink_color = p.accent;
    v.window_rounding = Rounding::same(12.0);
    v.menu_rounding = Rounding::same(8.0);
    v.window_stroke = Stroke::new(1.0_f32, p.stroke);

    v.selection.bg_fill = p.accent.linear_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0_f32, p.accent);

    let round = Rounding::same(7.0);
    v.widgets.noninteractive.bg_fill = p.surface;
    v.widgets.noninteractive.weak_bg_fill = p.surface;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.stroke);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.muted);
    v.widgets.noninteractive.rounding = round;

    v.widgets.inactive.bg_fill = p.surface2;
    v.widgets.inactive.weak_bg_fill = p.surface2;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.stroke);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
    v.widgets.inactive.rounding = round;
    v.widgets.inactive.expansion = 0.0;

    v.widgets.hovered.bg_fill = p.surface2;
    v.widgets.hovered.weak_bg_fill = p.surface2;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.accent);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.text);
    v.widgets.hovered.rounding = round;
    v.widgets.hovered.expansion = 1.0;

    v.widgets.active.bg_fill = p.accent;
    v.widgets.active.weak_bg_fill = p.accent;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, p.on_accent);
    v.widgets.active.rounding = round;
    v.widgets.active.expansion = 1.0;

    v.widgets.open.bg_fill = p.surface2;
    v.widgets.open.weak_bg_fill = p.surface2;
    v.widgets.open.bg_stroke = Stroke::new(1.0_f32, p.stroke);
    v.widgets.open.fg_stroke = Stroke::new(1.0_f32, p.text);
    v.widgets.open.rounding = round;

    style.visuals = v;

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.slider_width = 190.0;
    style.spacing.interact_size.y = 26.0;
    style.spacing.window_margin = Margin::same(16.0);

    style.text_styles = [
        (TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.5, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.5, FontFamily::Monospace)),
    ]
    .into();

    ctx.set_style(style);
}
