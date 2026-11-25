//! A simple front/back body figure whose muscle regions can be coloured.

use std::cell::Cell;

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Shape, Stroke, Ui,
    Vec2,
};

use crate::data::Muscle;
use crate::theme::MUTED;

pub const SKIN: Color32 = Color32::from_rgb(48, 52, 60);
pub const IDLE: Color32 = Color32::from_rgb(64, 69, 79);

/// Height needed for a figure of the given width.
pub fn height_for(width: f32) -> f32 {
    width * 1.88 + 18.0
}

/// Draws the figure and returns its response plus the muscle under the pointer.
pub fn figure(
    ui: &mut Ui,
    size: Vec2,
    back: bool,
    color_of: &dyn Fn(Muscle) -> Color32,
) -> (egui::Response, Option<Muscle>) {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    let s = (rect.width() / 100.0).min((rect.height() - 18.0) / 188.0).max(0.1);
    let ox = rect.center().x - 50.0 * s;
    let oy = rect.top() + 2.0;
    let hover = resp.hover_pos();
    let hovered: Cell<Option<Muscle>> = Cell::new(None);

    let p = |x: f32, y: f32| -> Pos2 { pos2(ox + x * s, oy + y * s) };
    let r = |x: f32, y: f32, w: f32, h: f32| -> Rect { Rect::from_min_size(p(x, y), vec2(w * s, h * s)) };
    let cr = |v: f32| -> CornerRadius { CornerRadius::same((v * s).round().clamp(0.0, 120.0) as u8) };

    let part = |rect: Rect, rad: f32, m: Option<Muscle>| {
        let col = match m {
            Some(m) => color_of(m),
            None => SKIN,
        };
        painter.rect_filled(rect, cr(rad), col);
        if let (Some(m), Some(h)) = (m, hover) {
            if rect.contains(h) {
                hovered.set(Some(m));
            }
        }
    };
    let poly = |pts: Vec<Pos2>, m: Muscle| {
        let bb = Rect::from_points(&pts);
        painter.add(Shape::convex_polygon(pts, color_of(m), Stroke::NONE));
        if let Some(h) = hover {
            if bb.contains(h) {
                hovered.set(Some(m));
            }
        }
    };
    // Mirror a left-side x coordinate to the right side.
    let mx = |x: f32, w: f32| 100.0 - x - w;

    // Head and neck
    painter.circle_filled(p(50.0, 11.0), 10.0 * s, SKIN);
    part(r(45.5, 19.0, 9.0, 8.0), 2.0, None);

    // Arms are the same shape front and back.
    let arm_upper = if back { Muscle::Triceps } else { Muscle::Biceps };
    for (x, w) in [(23.0f32, 11.0f32)] {
        part(r(x, 45.0, w, 19.0), 5.5, Some(arm_upper));
        part(r(mx(x, w), 45.0, w, 19.0), 5.5, Some(arm_upper));
    }
    for (x, w) in [(20.0f32, 10.0f32)] {
        part(r(x, 65.0, w, 22.0), 5.0, Some(Muscle::Forearms));
        part(r(mx(x, w), 65.0, w, 22.0), 5.0, Some(Muscle::Forearms));
    }
    painter.circle_filled(p(25.0, 91.0), 4.5 * s, SKIN);
    painter.circle_filled(p(75.0, 91.0), 4.5 * s, SKIN);

    if !back {
        // Traps peek over the collarbones
        part(r(39.0, 25.0, 22.0, 5.0), 2.5, Some(Muscle::Traps));
        // Shoulders
        part(r(24.0, 29.0, 13.0, 15.0), 7.0, Some(Muscle::Shoulders));
        part(r(mx(24.0, 13.0), 29.0, 13.0, 15.0), 7.0, Some(Muscle::Shoulders));
        // Chest
        part(r(37.5, 30.0, 12.0, 16.0), 5.0, Some(Muscle::Chest));
        part(r(50.5, 30.0, 12.0, 16.0), 5.0, Some(Muscle::Chest));
        // Abs (six-pack grid plus obliques)
        for row in 0..3 {
            for col in 0..2 {
                part(
                    r(40.5 + col as f32 * 9.75, 48.0 + row as f32 * 10.0, 9.0, 9.0),
                    2.5,
                    Some(Muscle::Abs),
                );
            }
        }
        part(r(35.5, 48.0, 4.0, 26.0), 2.0, Some(Muscle::Abs));
        part(r(60.5, 48.0, 4.0, 26.0), 2.0, Some(Muscle::Abs));
        // Hips
        part(r(37.0, 79.0, 26.0, 9.0), 4.0, None);
        // Quads
        part(r(36.0, 89.0, 13.5, 42.0), 6.5, Some(Muscle::Quads));
        part(r(50.5, 89.0, 13.5, 42.0), 6.5, Some(Muscle::Quads));
        // Knees
        part(r(37.5, 132.0, 11.0, 6.0), 3.0, None);
        part(r(51.5, 132.0, 11.0, 6.0), 3.0, None);
        // Shins / calves
        part(r(38.0, 139.0, 10.0, 36.0), 5.0, Some(Muscle::Calves));
        part(r(52.0, 139.0, 10.0, 36.0), 5.0, Some(Muscle::Calves));
    } else {
        // Upper back under the traps
        part(r(37.0, 29.0, 26.0, 20.0), 4.0, Some(Muscle::UpperBack));
        poly(vec![p(50.0, 20.0), p(63.0, 29.0), p(50.0, 46.0), p(37.0, 29.0)], Muscle::Traps);
        // Rear shoulders
        part(r(24.0, 29.0, 13.0, 15.0), 7.0, Some(Muscle::Shoulders));
        part(r(mx(24.0, 13.0), 29.0, 13.0, 15.0), 7.0, Some(Muscle::Shoulders));
        // Lats (clockwise on screen)
        poly(vec![p(36.0, 47.0), p(49.0, 50.0), p(49.0, 66.0), p(40.0, 64.0)], Muscle::Lats);
        poly(vec![p(60.0, 64.0), p(51.0, 66.0), p(51.0, 50.0), p(64.0, 47.0)], Muscle::Lats);
        // Lower back
        part(r(41.0, 67.0, 18.0, 12.0), 3.0, Some(Muscle::LowerBack));
        // Glutes
        part(r(36.5, 80.0, 13.0, 16.0), 7.0, Some(Muscle::Glutes));
        part(r(50.5, 80.0, 13.0, 16.0), 7.0, Some(Muscle::Glutes));
        // Hamstrings
        part(r(36.0, 97.0, 13.5, 36.0), 6.0, Some(Muscle::Hamstrings));
        part(r(50.5, 97.0, 13.5, 36.0), 6.0, Some(Muscle::Hamstrings));
        // Calves
        part(r(37.5, 137.0, 11.0, 37.0), 5.5, Some(Muscle::Calves));
        part(r(51.5, 137.0, 11.0, 37.0), 5.5, Some(Muscle::Calves));
    }
    // Feet
    part(r(37.0, 177.0, 12.0, 6.0), 3.0, None);
    part(r(51.0, 177.0, 12.0, 6.0), 3.0, None);

    painter.text(
        pos2(rect.center().x, rect.bottom() - 8.0),
        Align2::CENTER_CENTER,
        if back { "BACK" } else { "FRONT" },
        FontId::proportional(11.0),
        MUTED,
    );

    (resp, hovered.get())
}
