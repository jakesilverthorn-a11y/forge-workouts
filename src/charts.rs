//! Tiny hand-drawn charts (no plotting library needed).

use eframe::egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2,
};

use crate::theme::{GRID, MUTED, TEXT};
use crate::util::fmt_short_date;

pub fn bar_chart(
    ui: &mut Ui,
    size: Vec2,
    bars: &[(String, f32)],
    color: Color32,
    fmt: &dyn Fn(f32) -> String,
) {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    let max = bars.iter().map(|b| b.1).fold(0.0f32, f32::max).max(1.0);
    let n = bars.len().max(1) as f32;
    let plot = Rect::from_min_max(rect.min + vec2(0.0, 20.0), rect.max - vec2(0.0, 22.0));
    for k in 0..=3 {
        let y = plot.bottom() - plot.height() * k as f32 / 3.0;
        painter.line_segment([pos2(plot.left(), y), pos2(plot.right(), y)], Stroke::new(1.0, GRID));
    }
    let slot = plot.width() / n;
    let bw = (slot * 0.56).min(48.0);
    let hover_x = resp.hover_pos().map(|p| p.x);
    let mut tip: Option<String> = None;
    for (i, (label, v)) in bars.iter().enumerate() {
        let cx = plot.left() + slot * (i as f32 + 0.5);
        let h = (plot.height() * v / max).max(if *v > 0.0 { 3.0 } else { 0.0 });
        let br = Rect::from_min_max(pos2(cx - bw / 2.0, plot.bottom() - h), pos2(cx + bw / 2.0, plot.bottom()));
        let last = i + 1 == bars.len();
        let hovered = hover_x.map_or(false, |x| (x - cx).abs() <= slot / 2.0);
        let fill = if last || hovered { color } else { color.gamma_multiply(0.5) };
        if h > 0.0 {
            painter.rect_filled(br, CornerRadius::same(5), fill);
        }
        painter.text(pos2(cx, rect.bottom() - 9.0), Align2::CENTER_CENTER, label, FontId::proportional(11.0), MUTED);
        if *v > 0.0 && (last || hovered) {
            painter.text(pos2(cx, br.top() - 10.0), Align2::CENTER_CENTER, fmt(*v), FontId::proportional(11.5), TEXT);
        }
        if hovered {
            tip = Some(format!("Week of {}: {}", label, fmt(*v)));
        }
    }
    if let Some(t) = tip {
        resp.on_hover_text(t);
    }
}

pub fn line_chart(
    ui: &mut Ui,
    size: Vec2,
    pts: &[(i64, f32)],
    color: Color32,
    fmt_y: &dyn Fn(f32) -> String,
) {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    if pts.is_empty() {
        painter.text(rect.center(), Align2::CENTER_CENTER, "No sessions logged yet", FontId::proportional(13.0), MUTED);
        return;
    }
    let plot = Rect::from_min_max(rect.min + vec2(58.0, 10.0), rect.max - vec2(14.0, 24.0));
    let mut y0 = f32::MAX;
    let mut y1 = f32::MIN;
    for p in pts {
        y0 = y0.min(p.1);
        y1 = y1.max(p.1);
    }
    let pad = ((y1 - y0) * 0.2).max(y1.abs() * 0.05).max(1.0);
    let y0 = (y0 - pad).max(0.0);
    let y1 = y1 + pad;
    let x0 = pts[0].0 as f64;
    let x1 = pts[pts.len() - 1].0 as f64;
    let xs = (x1 - x0).max(1.0);
    let single = pts.len() == 1;
    let to_screen = |t: i64, v: f32| -> Pos2 {
        let fx = if single { 0.5 } else { ((t as f64 - x0) / xs) as f32 };
        pos2(
            plot.left() + fx * plot.width(),
            plot.bottom() - (v - y0) / (y1 - y0) * plot.height(),
        )
    };
    for k in 0..=4 {
        let v = y0 + (y1 - y0) * k as f32 / 4.0;
        let y = plot.bottom() - plot.height() * k as f32 / 4.0;
        painter.line_segment([pos2(plot.left(), y), pos2(plot.right(), y)], Stroke::new(1.0, GRID));
        painter.text(pos2(plot.left() - 8.0, y), Align2::RIGHT_CENTER, fmt_y(v), FontId::proportional(11.0), MUTED);
    }
    let screen: Vec<Pos2> = pts.iter().map(|p| to_screen(p.0, p.1)).collect();
    // Soft drop lines under each point.
    for sp in &screen {
        painter.line_segment([*sp, pos2(sp.x, plot.bottom())], Stroke::new(1.0, color.gamma_multiply(0.15)));
    }
    if screen.len() >= 2 {
        painter.add(Shape::line(screen.clone(), Stroke::new(2.5, color)));
    }
    for (i, sp) in screen.iter().enumerate() {
        let last = i + 1 == screen.len();
        painter.circle_filled(*sp, if last { 5.0 } else { 3.2 }, color);
    }
    painter.text(pos2(plot.left(), rect.bottom() - 9.0), Align2::LEFT_CENTER, fmt_short_date(pts[0].0), FontId::proportional(11.0), MUTED);
    if !single {
        painter.text(pos2(plot.right(), rect.bottom() - 9.0), Align2::RIGHT_CENTER, fmt_short_date(pts[pts.len() - 1].0), FontId::proportional(11.0), MUTED);
    }
    let mut tip: Option<String> = None;
    if let Some(hp) = resp.hover_pos() {
        let mut best = 0usize;
        let mut best_d = f32::MAX;
        for (i, sp) in screen.iter().enumerate() {
            let d = (sp.x - hp.x).abs();
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        let sp = screen[best];
        painter.line_segment([pos2(sp.x, plot.top()), pos2(sp.x, plot.bottom())], Stroke::new(1.0, MUTED.gamma_multiply(0.5)));
        painter.circle_stroke(sp, 7.0, Stroke::new(2.0, color));
        tip = Some(format!("{}: {}", fmt_short_date(pts[best].0), fmt_y(pts[best].1)));
    }
    if let Some(t) = tip {
        resp.on_hover_text(t);
    }
}
