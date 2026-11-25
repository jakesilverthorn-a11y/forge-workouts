//! Colours, style and small reusable widgets.

use eframe::egui::{
    self, vec2, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, RichText, Sense,
    Stroke, TextStyle, Ui,
};

use crate::data::ExerciseDef;

pub const BG: Color32 = Color32::from_rgb(17, 18, 21);
pub const PANEL: Color32 = Color32::from_rgb(23, 25, 29);
pub const CARD: Color32 = Color32::from_rgb(30, 33, 38);
pub const CARD2: Color32 = Color32::from_rgb(40, 44, 51);
pub const BORDER: Color32 = Color32::from_rgb(46, 50, 58);
pub const GRID: Color32 = Color32::from_rgb(40, 43, 50);
pub const TEXT: Color32 = Color32::from_rgb(236, 237, 240);
pub const MUTED: Color32 = Color32::from_rgb(148, 154, 165);
pub const ACCENT: Color32 = Color32::from_rgb(255, 92, 66);
pub const ACCENT_TEXT: Color32 = Color32::from_rgb(255, 146, 126);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(56, 33, 31);
pub const GOOD: Color32 = Color32::from_rgb(59, 209, 127);
pub const WARN: Color32 = Color32::from_rgb(245, 166, 35);
pub const BAD: Color32 = Color32::from_rgb(240, 82, 74);
pub const BLUE: Color32 = Color32::from_rgb(82, 166, 255);

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(TextStyle::Body, FontId::proportional(14.5));
    style.text_styles.insert(TextStyle::Button, FontId::proportional(14.0));
    style.text_styles.insert(TextStyle::Small, FontId::proportional(11.5));
    style.text_styles.insert(TextStyle::Heading, FontId::proportional(24.0));
    style.text_styles.insert(TextStyle::Monospace, FontId::monospace(13.5));
    style.spacing.item_spacing = vec2(8.0, 7.0);
    style.spacing.button_padding = vec2(12.0, 6.0);
    style.spacing.interact_size.y = 28.0;
    style.interaction.selectable_labels = false;

    let v = &mut style.visuals;
    v.dark_mode = true;
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.extreme_bg_color = Color32::from_rgb(13, 14, 17);
    v.faint_bg_color = CARD;
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.selection.bg_fill = Color32::from_rgb(150, 60, 46);
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.hyperlink_color = ACCENT;

    v.widgets.noninteractive.bg_fill = CARD;
    v.widgets.noninteractive.weak_bg_fill = CARD;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);

    v.widgets.inactive.bg_fill = CARD2;
    v.widgets.inactive.weak_bg_fill = CARD2;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);

    let hover = Color32::from_rgb(52, 57, 66);
    v.widgets.hovered.bg_fill = hover;
    v.widgets.hovered.weak_bg_fill = hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(120, 70, 62));
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT);

    v.widgets.active.bg_fill = ACCENT;
    v.widgets.active.weak_bg_fill = ACCENT;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    v.widgets.open.bg_fill = CARD2;
    v.widgets.open.weak_bg_fill = CARD2;
    v.widgets.open.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.open.fg_stroke = Stroke::new(1.0, TEXT);

    ctx.set_style(style);
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| -> u8 { (x as f32 + (y as f32 - x as f32) * t).round() as u8 };
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

/// Red (tired) -> amber -> green (fresh).
pub fn rec_color(pct: f32) -> Color32 {
    let p = (pct / 100.0).clamp(0.0, 1.0);
    if p < 0.5 {
        lerp_color(BAD, WARN, p * 2.0)
    } else {
        lerp_color(WARN, GOOD, (p - 0.5) * 2.0)
    }
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(CARD)
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(16))
        .stroke(Stroke::new(1.0, BORDER))
}

pub fn accent_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text).size(15.0).strong().color(Color32::WHITE))
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(10))
        .min_size(vec2(150.0, 40.0))
}

pub fn ghost_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text).size(14.0).color(TEXT))
        .fill(CARD2)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(10))
        .min_size(vec2(0.0, 36.0))
}

pub fn small_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text).size(12.5).color(TEXT))
        .fill(CARD2)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(8))
        .min_size(vec2(0.0, 28.0))
}

pub fn small_accent_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text).size(12.5).strong().color(Color32::WHITE))
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(8))
        .min_size(vec2(56.0, 28.0))
}

pub fn chip(ui: &mut Ui, selected: bool, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(13.0).color(if selected {
            Color32::WHITE
        } else {
            TEXT
        }))
        .fill(if selected { ACCENT } else { CARD2 })
        .stroke(Stroke::new(1.0, if selected { ACCENT } else { BORDER }))
        .corner_radius(CornerRadius::same(16))
        .min_size(vec2(0.0, 30.0)),
    )
}

pub fn pill(ui: &mut Ui, text: &str, color: Color32) {
    egui::Frame::default()
        .fill(color.gamma_multiply(0.16))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.5).color(color));
        });
}

pub fn page_header(ui: &mut Ui, title: &str, sub: &str) {
    ui.label(RichText::new(title).size(30.0).strong().color(TEXT));
    ui.label(RichText::new(sub).size(14.0).color(MUTED));
    ui.add_space(14.0);
}

pub fn section_title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(16.5).strong().color(TEXT));
    ui.add_space(4.0);
}

pub fn kicker(ui: &mut Ui, text: &str, color: Color32) {
    ui.label(RichText::new(text.to_uppercase()).size(11.5).strong().color(color));
}

/// A coloured tile standing in for an exercise picture.
pub fn thumb(ui: &mut Ui, def: &ExerciseDef, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let p = ui.painter();
    let c = def.primary.group().color();
    let radius = (size * 0.24).clamp(2.0, 40.0) as u8;
    p.rect_filled(rect, CornerRadius::same(radius), c.gamma_multiply(0.2));
    // A tiny dumbbell drawn across the top half.
    let cy = rect.top() + size * 0.38;
    let half = size * 0.26;
    let bar = Stroke::new((size * 0.05).max(1.5), c);
    p.line_segment([egui::pos2(rect.center().x - half, cy), egui::pos2(rect.center().x + half, cy)], bar);
    for sx in [-1.0f32, 1.0] {
        let x = rect.center().x + sx * half;
        let plate = egui::Rect::from_center_size(egui::pos2(x, cy), vec2(size * 0.09, size * 0.26));
        p.rect_filled(plate, CornerRadius::same(2), c);
    }
    p.text(
        egui::pos2(rect.center().x, rect.top() + size * 0.74),
        Align2::CENTER_CENTER,
        def.equipment.short(),
        FontId::proportional((size * 0.24).max(9.0)),
        c,
    );
}

/// Round check button used to tick off sets.
pub fn done_toggle(ui: &mut Ui, done: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 26.0), Sense::click());
    let c = rect.center();
    let p = ui.painter();
    if done {
        p.circle_filled(c, 12.0, GOOD);
        let s = Stroke::new(2.4, Color32::WHITE);
        p.line_segment([c + vec2(-5.0, 0.5), c + vec2(-1.5, 4.0)], s);
        p.line_segment([c + vec2(-1.5, 4.0), c + vec2(5.5, -4.0)], s);
    } else {
        let col = if resp.hovered() { GOOD } else { MUTED };
        p.circle_stroke(c, 11.0, Stroke::new(1.8, col));
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Muscle name, percent and a coloured bar.
pub fn rec_bar(ui: &mut Ui, label: &str, pct: f32, sub: Option<&str>, w: f32) {
    ui.allocate_ui_with_layout(vec2(w, 20.0), Layout::left_to_right(Align::Center), |ui| {
        ui.set_width(w);
        ui.label(RichText::new(label).size(13.5).color(TEXT));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(format!("{:.0}%", pct)).size(13.5).strong().color(rec_color(pct)));
            if let Some(s) = sub {
                ui.label(RichText::new(s).size(11.5).color(MUTED));
            }
        });
    });
    let (rect, _) = ui.allocate_exact_size(vec2(w, 7.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(4), CARD2);
    let mut fill = rect;
    fill.set_width((rect.width() * pct / 100.0).max(4.0));
    ui.painter().rect_filled(fill, CornerRadius::same(4), rec_color(pct));
    ui.add_space(6.0);
}

/// A selectable tile with a title and a one-line explanation.
pub fn option_tile(ui: &mut Ui, selected: bool, title: &str, blurb: &str, w: f32) -> bool {
    let resp = egui::Frame::default()
        .fill(if selected { ACCENT_SOFT } else { CARD2 })
        .stroke(Stroke::new(
            if selected { 1.5 } else { 1.0 },
            if selected { ACCENT } else { BORDER },
        ))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(w - 27.0);
                ui.label(RichText::new(title).size(15.0).strong().color(if selected {
                    ACCENT_TEXT
                } else {
                    TEXT
                }));
                ui.label(RichText::new(blurb).size(12.0).color(MUTED));
            });
        })
        .response;
    resp.interact(Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

pub fn stat_card(ui: &mut Ui, w: f32, title: &str, value: &str, sub: &str, color: Color32) {
    card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.set_width(w - 34.0);
            kicker(ui, title, MUTED);
            ui.label(RichText::new(value).size(26.0).strong().color(color));
            ui.label(RichText::new(sub).size(12.0).color(MUTED));
        });
    });
}
