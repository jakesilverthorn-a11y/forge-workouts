//! The window: app state, navigation, pop-ups and the screenshot option.
//! The pages themselves live in `app/workout.rs` and `app/pages.rs`.

mod pages;
mod progress;
mod workout;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, Rect,
    RichText, ScrollArea, Sense, Stroke, Ui,
};

use crate::body;
use crate::charts;
use crate::data::{self, Equip, ExerciseDef, Kind, Muscle, EXERCISES, MUSCLE_COUNT};
use crate::engine::*;
use crate::theme::*;
use crate::util::*;

const STORAGE_KEY: &str = "forge_data_v1";

/// Screens narrower than this (in points) get the phone layout.
const COMPACT_WIDTH: f32 = 720.0;
const PHONE_TOP_H: f32 = 52.0;
const PHONE_TABS_H: f32 = 60.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Workout,
    Recovery,
    Exercises,
    Log,
    Progress,
    Settings,
}

impl Tab {
    const ALL: [Tab; 6] = [
        Tab::Workout,
        Tab::Recovery,
        Tab::Exercises,
        Tab::Log,
        Tab::Progress,
        Tab::Settings,
    ];
    fn name(self) -> &'static str {
        match self {
            Tab::Workout => "Workout",
            Tab::Recovery => "Recovery",
            Tab::Exercises => "Exercises",
            Tab::Log => "Log",
            Tab::Progress => "Progress",
            Tab::Settings => "Settings",
        }
    }
    /// Label for the phone tab bar.
    fn short(self) -> &'static str {
        match self {
            Tab::Workout => "Workout",
            Tab::Recovery => "Recovery",
            Tab::Exercises => "Library",
            Tab::Log => "Log",
            Tab::Progress => "Progress",
            Tab::Settings => "Settings",
        }
    }
    fn hint(self) -> &'static str {
        match self {
            Tab::Workout => "Today's plan",
            Tab::Recovery => "Muscle readiness",
            Tab::Exercises => "Library",
            Tab::Log => "Past workouts",
            Tab::Progress => "Trends & records",
            Tab::Settings => "Goals & gym",
        }
    }
}

struct Active {
    start: i64,
    rest_until: Option<f64>,
}

pub struct ForgeApp {
    data: AppData,
    tab: Tab,
    focus: Focus,
    plan: Plan,
    seed: u64,
    active: Option<Active>,
    expanded: HashSet<usize>,
    swap_idx: Option<usize>,
    add_open: bool,
    add_search: String,
    lib_search: String,
    lib_filter: Option<Muscle>,
    lib_only_mine: bool,
    lib_selected: &'static str,
    /// Phone layout: showing one exercise's details instead of the list.
    lib_detail_open: bool,
    /// True on narrow screens (phones): one column and a bottom tab bar.
    compact: bool,
    log_selected: Option<i64>,
    progress_ex: Option<&'static str>,
    confirm_clear: bool,
    toast: Option<(String, f64)>,
    shot_path: Option<PathBuf>,
    frames: u32,
    shot_requested: bool,
}

impl ForgeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, shot_path: Option<PathBuf>) -> Self {
        apply(&cc.egui_ctx);
        let now = now_secs();
        let mut loaded: Option<AppData> = None;
        if shot_path.is_none() {
            if let Some(storage) = cc.storage {
                loaded = eframe::get_value(storage, STORAGE_KEY);
            }
        }
        let data = loaded.unwrap_or_else(|| {
            let settings = Settings::default();
            let history = sample_history(&settings, now);
            AppData { settings, history }
        });
        let seed = if shot_path.is_some() { 7 } else { (now as u64) ^ 0x5DEE_CE66 };
        let plan = generate(&data.settings, &data.history, now, Focus::Auto, seed);
        Self {
            data,
            tab: Tab::Workout,
            focus: Focus::Auto,
            plan,
            seed,
            active: None,
            expanded: HashSet::new(),
            swap_idx: None,
            add_open: false,
            add_search: String::new(),
            lib_search: String::new(),
            lib_filter: None,
            lib_only_mine: true,
            lib_selected: "bench_press",
            lib_detail_open: false,
            compact: false,
            log_selected: None,
            progress_ex: None,
            confirm_clear: false,
            toast: None,
            shot_path,
            frames: 0,
            shot_requested: false,
        }
    }

    fn regenerate(&mut self) {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.plan = generate(&self.data.settings, &self.data.history, now_secs(), self.focus, self.seed);
        self.expanded.clear();
        self.swap_idx = None;
    }

    fn show_toast(&mut self, msg: String) {
        self.toast = Some((msg, now_f()));
    }

    fn start_workout(&mut self) {
        for e in &mut self.plan.exercises {
            for s in &mut e.sets {
                s.done = false;
            }
        }
        self.active = Some(Active { start: now_secs(), rest_until: None });
    }

    fn discard_workout(&mut self) {
        self.active = None;
        for e in &mut self.plan.exercises {
            for s in &mut e.sets {
                s.done = false;
            }
        }
        self.show_toast("Workout discarded.".to_string());
    }

    fn finish_workout(&mut self) {
        let now = now_secs();
        let start = self.active.as_ref().map(|a| a.start).unwrap_or(now);
        let exercises: Vec<ExerciseLog> = self
            .plan
            .exercises
            .iter()
            .filter_map(|e| {
                let sets: Vec<SetLog> = e.sets.iter().filter(|s| s.done).copied().collect();
                if sets.is_empty() {
                    None
                } else {
                    Some(ExerciseLog { id: e.id.clone(), sets })
                }
            })
            .collect();
        self.active = None;
        if exercises.is_empty() {
            self.show_toast("No sets were checked off, so nothing was saved.".to_string());
        } else {
            let log = WorkoutLog {
                start,
                duration_secs: (now - start).max(60),
                title: self.plan.title.clone(),
                exercises,
            };
            let msg = format!(
                "Workout saved: {} sets, {} moved. Nice work!",
                log.set_count(),
                self.data.settings.units.fmt_big(log.volume())
            );
            self.data.history.push(log);
            self.data.history.sort_by_key(|w| w.start);
            self.show_toast(msg);
        }
        self.regenerate();
    }

    // ------------------------------------------------------------ screenshot

    #[cfg(target_arch = "wasm32")]
    fn handle_screenshot(&mut self, _ctx: &egui::Context) {}

    #[cfg(not(target_arch = "wasm32"))]
    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.shot_path.clone() else { return };
        self.frames += 1;
        ctx.request_repaint();
        if self.frames >= 120 && !self.shot_requested {
            self.shot_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(shot) = shot {
            let [w, h] = shot.size;
            let mut buf: Vec<u8> = Vec::with_capacity(w * h * 4);
            for px in &shot.pixels {
                buf.extend_from_slice(&px.to_srgba_unmultiplied());
            }
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            let ok = match image::RgbaImage::from_raw(w as u32, h as u32, buf) {
                Some(img) => img.save(&path).is_ok(),
                None => false,
            };
            std::process::exit(if ok { 0 } else { 2 });
        }
        if self.frames > 2000 {
            std::process::exit(3);
        }
    }

    // ------------------------------------------------------------ navigation

    fn nav(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            logo(ui, 40.0);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("Forge").size(22.0).strong().color(TEXT));
                ui.label(RichText::new("smart workout planner").size(11.5).color(MUTED));
            });
        });
        ui.add_space(24.0);
        for t in Tab::ALL {
            let live = t == Tab::Workout && self.active.is_some();
            if nav_item(ui, t.name(), t.hint(), self.tab == t, live) {
                self.tab = t;
            }
            ui.add_space(2.0);
        }

        let now = now_secs();
        let this_week = week_start_day(now);
        let mut days = [false; 7];
        let mut count = 0;
        for w in &self.data.history {
            if week_start_day(w.start) == this_week {
                count += 1;
                days[weekday_mon(day_index(w.start))] = true;
            }
        }
        let today_wd = weekday_mon(day_index(now));
        ui.add_space(18.0);
        {
            egui::Frame::default()
                .fill(CARD)
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        kicker(ui, "This week", MUTED);
                        ui.label(
                            RichText::new(format!("{} workout{}", count, if count == 1 { "" } else { "s" }))
                                .size(18.0)
                                .strong()
                                .color(TEXT),
                        );
                        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::hover());
                        let cell = rect.width() / 7.0;
                        for (i, letter) in ["M", "T", "W", "T", "F", "S", "S"].iter().enumerate() {
                            let c = pos2(rect.left() + cell * (i as f32 + 0.5), rect.center().y);
                            let fill = if days[i] { ACCENT } else { CARD2 };
                            ui.painter().circle_filled(c, 11.0, fill);
                            if i == today_wd && !days[i] {
                                ui.painter().circle_stroke(c, 11.0, Stroke::new(1.5, ACCENT));
                            }
                            ui.painter().text(
                                c,
                                Align2::CENTER_CENTER,
                                *letter,
                                FontId::proportional(10.5),
                                if days[i] { Color32::WHITE } else { MUTED },
                            );
                        }
                    });
                });
        }
    }

    // ------------------------------------------------------------ phone layout

    fn phone_top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            logo(ui, 30.0);
            ui.add_space(2.0);
            ui.label(RichText::new("Forge").size(19.0).strong().color(TEXT));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(a) = &self.active {
                    let elapsed = now_secs() - a.start;
                    ui.label(RichText::new(fmt_clock(elapsed)).monospace().size(15.0).color(TEXT));
                    let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.5, GOOD);
                } else {
                    let now = now_secs();
                    let this_week = week_start_day(now);
                    let n = self
                        .data
                        .history
                        .iter()
                        .filter(|w| week_start_day(w.start) == this_week)
                        .count();
                    pill(ui, &format!("{} this week", n), ACCENT_TEXT);
                }
            });
        });
    }

    fn phone_tab_bar(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        ui.painter().hline(rect.x_range(), rect.top(), Stroke::new(1.0, BORDER));
        let w = rect.width() / Tab::ALL.len() as f32;
        for (k, t) in Tab::ALL.into_iter().enumerate() {
            let r = Rect::from_min_size(pos2(rect.left() + w * k as f32, rect.top()), vec2(w, rect.height()));
            let resp = ui.interact(r, egui::Id::new(("tab", k)), Sense::click());
            let selected = self.tab == t;
            let col = if selected { ACCENT } else { MUTED };
            let p = ui.painter();
            if selected {
                let bar = Rect::from_center_size(pos2(r.center().x, r.top() + 1.5), vec2(28.0, 3.0));
                p.rect_filled(bar, CornerRadius::same(2), ACCENT);
            }
            tab_icon(p, t, pos2(r.center().x, r.top() + 22.0), col);
            p.text(
                pos2(r.center().x, r.top() + 44.0),
                Align2::CENTER_CENTER,
                t.short(),
                FontId::proportional(11.0),
                if selected { TEXT } else { MUTED },
            );
            if t == Tab::Workout && self.active.is_some() {
                p.circle_filled(pos2(r.center().x + 14.0, r.top() + 12.0), 4.0, GOOD);
            }
            if resp.clicked() {
                self.tab = t;
                self.lib_detail_open = false;
            }
        }
    }

    /// Phones: the rest countdown floats just above the tab bar, so it stays in view
    /// while you scroll through your sets.
    fn phone_rest_bar(&mut self, ctx: &egui::Context) {
        let Some(until) = self.active.as_ref().and_then(|a| a.rest_until) else { return };
        let left = until - now_f();
        if left <= 0.0 {
            return;
        }
        let w = ctx.screen_rect().width() - 24.0;
        let mut skip = false;
        egui::Area::new(egui::Id::new("phone_rest"))
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -(PHONE_TABS_H + 10.0)))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(ACCENT_SOFT)
                    .stroke(Stroke::new(1.0, ACCENT))
                    .corner_radius(CornerRadius::same(14))
                    .inner_margin(Margin::symmetric(14, 8))
                    .show(ui, |ui| {
                        ui.set_width(w - 28.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Rest").size(15.0).strong().color(ACCENT_TEXT));
                            ui.label(
                                RichText::new(fmt_clock(left.ceil() as i64))
                                    .monospace()
                                    .size(22.0)
                                    .color(TEXT),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.add(small_button("Skip rest").min_size(vec2(0.0, 34.0))).clicked() {
                                    skip = true;
                                }
                            });
                        });
                    });
            });
        if skip {
            if let Some(a) = self.active.as_mut() {
                a.rest_until = None;
            }
        }
    }

    /// Width for a pop-up window: its desktop size, or the screen minus a margin on phones.
    fn window_width(&self, ctx: &egui::Context, desktop: f32) -> f32 {
        desktop.min(ctx.screen_rect().width() - 24.0)
    }

    fn window_height(&self, ctx: &egui::Context, desktop: f32) -> f32 {
        let reserved = if self.compact { PHONE_TOP_H + PHONE_TABS_H + 40.0 } else { 60.0 };
        desktop.min(ctx.screen_rect().height() - reserved).max(200.0)
    }

    // ------------------------------------------------------------ pop-up windows

    fn windows(&mut self, ctx: &egui::Context) {
        if let Some(idx) = self.swap_idx {
            let mut close = false;
            let mut pick: Option<&'static ExerciseDef> = None;
            let cur: Option<&'static ExerciseDef> =
                self.plan.exercises.get(idx).and_then(|e| data::find(&e.id));
            match cur {
                Some(cur) => {
                    let alts = alternatives(cur, &self.data.settings, &self.plan);
                    let (ww, wh) = (self.window_width(ctx, 440.0), self.window_height(ctx, 420.0));
                    egui::Window::new("Swap exercise")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                        .fixed_size(vec2(ww, wh))
                        .show(ctx, |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "Replace {} with another {} exercise:",
                                    cur.name,
                                    cur.primary.name().to_lowercase()
                                ))
                                .color(MUTED),
                            );
                            ui.add_space(6.0);
                            ScrollArea::vertical().max_height(wh - 80.0).show(ui, |ui| {
                                if alts.is_empty() {
                                    ui.label(RichText::new("No alternatives fit your equipment.").color(MUTED));
                                }
                                for d in alts.iter().copied() {
                                    ui.horizontal(|ui| {
                                        thumb(ui, d, 36.0);
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(d.name).strong().color(TEXT));
                                            ui.label(RichText::new(d.equipment.name()).size(12.0).color(MUTED));
                                        });
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if ui.add(small_accent_button("Use")).clicked() {
                                                pick = Some(d);
                                            }
                                        });
                                    });
                                    ui.add_space(4.0);
                                }
                            });
                            ui.add_space(6.0);
                            if ui.add(ghost_button("Cancel")).clicked() {
                                close = true;
                            }
                        });
                }
                None => close = true,
            }
            if let Some(d) = pick {
                let e = prescribe(d, &self.data.settings, &self.data.history);
                if idx < self.plan.exercises.len() {
                    self.plan.exercises[idx] = e;
                }
                close = true;
            }
            if close {
                self.swap_idx = None;
            }
        }

        if self.add_open {
            let mut close = false;
            let mut add: Option<&'static ExerciseDef> = None;
            let settings = self.data.settings.clone();
            let (ww, wh) = (self.window_width(ctx, 460.0), self.window_height(ctx, 480.0));
            let search: &mut String = &mut self.add_search;
            egui::Window::new("Add exercise")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .fixed_size(vec2(ww, wh))
                .show(ctx, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut *search)
                            .hint_text("Search by name, muscle or equipment")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(6.0);
                    let q = search.trim().to_lowercase();
                    ScrollArea::vertical().max_height(wh - 100.0).show(ui, |ui| {
                        for d in EXERCISES.iter() {
                            let matches = settings.has(d.equipment)
                                && (q.is_empty()
                                    || d.name.to_lowercase().contains(&q)
                                    || d.primary.name().to_lowercase().contains(&q)
                                    || d.equipment.name().to_lowercase().contains(&q));
                            if !matches {
                                continue;
                            }
                            ui.horizontal(|ui| {
                                thumb(ui, d, 36.0);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(d.name).strong().color(TEXT));
                                    ui.label(
                                        RichText::new(format!("{} · {}", d.primary.name(), d.equipment.name()))
                                            .size(12.0)
                                            .color(MUTED),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.add(small_accent_button("Add")).clicked() {
                                        add = Some(d);
                                    }
                                });
                            });
                            ui.add_space(4.0);
                        }
                    });
                    ui.add_space(6.0);
                    if ui.add(ghost_button("Close")).clicked() {
                        close = true;
                    }
                });
            if let Some(d) = add {
                let e = prescribe(d, &self.data.settings, &self.data.history);
                self.plan.exercises.push(e);
                self.show_toast(format!("Added {}.", d.name));
                close = true;
            }
            if close {
                self.add_open = false;
            }
        }
    }

    fn draw_toast(&mut self, ctx: &egui::Context) {
        let Some((msg, t)) = self.toast.clone() else { return };
        if now_f() - t > 3.5 {
            self.toast = None;
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(200));
        // Centre over the page: right of the sidebar on desktop, above the tab bar on phones.
        let resting = self.active.as_ref().and_then(|a| a.rest_until).is_some_and(|u| u > now_f());
        let offset = if self.compact {
            // Sit above the floating rest bar when it is showing.
            vec2(0.0, -(PHONE_TABS_H + if resting { 70.0 } else { 14.0 }))
        } else {
            vec2(110.0, -28.0)
        };
        egui::Area::new(egui::Id::new("toast"))
            .anchor(Align2::CENTER_BOTTOM, offset)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(CARD2)
                    .stroke(Stroke::new(1.0, GOOD))
                    .corner_radius(CornerRadius::same(20))
                    .inner_margin(Margin::symmetric(18, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new(msg).size(14.0).color(TEXT));
                    });
            });
    }
}

impl eframe::App for ForgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_screenshot(ctx);
        if self.active.is_some() {
            ctx.request_repaint_after(Duration::from_millis(250));
        }

        self.compact = ctx.screen_rect().width() < COMPACT_WIDTH;

        let page_margin = if self.compact {
            egui::TopBottomPanel::top("phone_top")
                .exact_height(PHONE_TOP_H)
                .frame(egui::Frame::default().fill(PANEL).inner_margin(Margin::symmetric(14, 0)))
                .show(ctx, |ui| self.phone_top_bar(ui));
            egui::TopBottomPanel::bottom("phone_tabs")
                .exact_height(PHONE_TABS_H)
                .frame(egui::Frame::default().fill(PANEL).inner_margin(Margin::ZERO))
                .show(ctx, |ui| self.phone_tab_bar(ui));
            Margin::symmetric(14, 14)
        } else {
            egui::SidePanel::left("nav")
                .exact_width(224.0)
                .resizable(false)
                .frame(egui::Frame::default().fill(PANEL).inner_margin(Margin::same(16)))
                .show(ctx, |ui| self.nav(ui));
            Margin::symmetric(28, 22)
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(BG).inner_margin(page_margin))
            .show(ctx, |ui| {
                let tab = self.tab;
                let edge = if self.compact { 0.0 } else { 6.0 };
                ui.push_id(tab.name(), |ui| {
                    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.set_width(ui.available_width() - edge);
                        match tab {
                            Tab::Workout => self.page_workout(ui),
                            Tab::Recovery => self.page_recovery(ui),
                            Tab::Exercises => self.page_exercises(ui),
                            Tab::Log => self.page_log(ui),
                            Tab::Progress => self.page_progress(ui),
                            Tab::Settings => self.page_settings(ui),
                        }
                        ui.add_space(24.0);
                    });
                });
            });

        if self.compact {
            self.phone_rest_bar(ctx);
        }
        self.windows(ctx);
        self.draw_toast(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.shot_path.is_none() {
            eframe::set_value(storage, STORAGE_KEY, &self.data);
        }
    }
}

// ---------------------------------------------------------------- shared helpers

fn logo(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same((size * 0.28) as u8), ACCENT);
    let c = rect.center();
    let white = Color32::WHITE;
    p.line_segment(
        [c + vec2(-size * 0.26, 0.0), c + vec2(size * 0.26, 0.0)],
        Stroke::new(size * 0.07, white),
    );
    for sx in [-1.0f32, 1.0] {
        let big = Rect::from_center_size(c + vec2(sx * size * 0.2, 0.0), vec2(size * 0.1, size * 0.42));
        let small = Rect::from_center_size(c + vec2(sx * size * 0.31, 0.0), vec2(size * 0.08, size * 0.26));
        p.rect_filled(big, CornerRadius::same(2), white);
        p.rect_filled(small, CornerRadius::same(2), white);
    }
}

/// Small line icons for the phone tab bar.
fn tab_icon(p: &egui::Painter, tab: Tab, c: egui::Pos2, col: Color32) {
    let s = Stroke::new(1.8, col);
    match tab {
        Tab::Workout => {
            p.line_segment([c + vec2(-7.0, 0.0), c + vec2(7.0, 0.0)], s);
            for sx in [-1.0f32, 1.0] {
                p.rect_filled(Rect::from_center_size(c + vec2(sx * 6.5, 0.0), vec2(3.5, 13.0)), CornerRadius::same(1), col);
                p.rect_filled(Rect::from_center_size(c + vec2(sx * 10.0, 0.0), vec2(2.5, 8.0)), CornerRadius::same(1), col);
            }
        }
        Tab::Recovery => {
            p.circle_stroke(c, 9.0, s);
            let pts = [
                c + vec2(-6.0, 1.0),
                c + vec2(-2.5, 1.0),
                c + vec2(-1.0, -4.0),
                c + vec2(1.5, 4.0),
                c + vec2(3.0, 1.0),
                c + vec2(6.0, 1.0),
            ];
            for w in pts.windows(2) {
                p.line_segment([w[0], w[1]], s);
            }
        }
        Tab::Exercises => {
            for dy in [-6.0f32, 0.0, 6.0] {
                p.circle_filled(c + vec2(-8.0, dy), 1.6, col);
                p.line_segment([c + vec2(-4.0, dy), c + vec2(9.0, dy)], s);
            }
        }
        Tab::Log => {
            let r = Rect::from_center_size(c + vec2(0.0, 1.0), vec2(18.0, 16.0));
            p.rect_stroke(r, CornerRadius::same(3), s, egui::StrokeKind::Middle);
            p.line_segment([r.left_top() + vec2(0.0, 5.0), r.right_top() + vec2(0.0, 5.0)], s);
            for dx in [-4.5f32, 4.5] {
                p.line_segment([c + vec2(dx, -10.0), c + vec2(dx, -5.0)], s);
            }
        }
        Tab::Progress => {
            for (dx, h) in [(-6.0f32, 7.0f32), (0.0, 12.0), (6.0, 17.0)] {
                let r = Rect::from_min_max(pos2(c.x + dx - 2.0, c.y + 9.0 - h), pos2(c.x + dx + 2.0, c.y + 9.0));
                p.rect_filled(r, CornerRadius::same(1), col);
            }
        }
        Tab::Settings => {
            for dy in [-6.0f32, 0.0, 6.0] {
                p.line_segment([c + vec2(-9.0, dy), c + vec2(9.0, dy)], s);
            }
            for (dx, dy) in [(4.0f32, -6.0f32), (-4.0, 0.0), (2.0, 6.0)] {
                p.circle_filled(c + vec2(dx, dy), 3.0, PANEL);
                p.circle_stroke(c + vec2(dx, dy), 3.0, s);
            }
        }
    }
}

fn nav_item(ui: &mut Ui, label: &str, hint: &str, selected: bool, live: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::click());
    let p = ui.painter();
    let bg = if selected {
        ACCENT_SOFT
    } else if resp.hovered() {
        CARD
    } else {
        Color32::TRANSPARENT
    };
    p.rect_filled(rect, CornerRadius::same(10), bg);
    if selected {
        let bar = Rect::from_min_size(rect.left_top() + vec2(0.0, 12.0), vec2(3.0, rect.height() - 24.0));
        p.rect_filled(bar, CornerRadius::same(2), ACCENT);
    }
    let dot_c = rect.left_center() + vec2(20.0, 0.0);
    p.circle_filled(dot_c, 5.0, if selected { ACCENT } else { Color32::from_rgb(90, 95, 105) });
    p.text(
        rect.left_center() + vec2(36.0, -8.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(15.0),
        if selected { TEXT } else { Color32::from_rgb(205, 208, 214) },
    );
    p.text(
        rect.left_center() + vec2(36.0, 10.0),
        Align2::LEFT_CENTER,
        hint,
        FontId::proportional(11.0),
        MUTED,
    );
    if live {
        let c = rect.right_center() - vec2(18.0, 0.0);
        p.circle_filled(c, 5.0, GOOD);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// Two columns side by side on desktop; stacked (left first) on phones.
fn two_columns(
    ui: &mut Ui,
    compact: bool,
    left_w: f32,
    right_w: f32,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) {
    if compact {
        ui.vertical(|ui| {
            left(ui);
            ui.add_space(14.0);
            right(ui);
        });
    } else {
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(left_w);
                left(ui);
            });
            ui.add_space(20.0);
            ui.allocate_ui_with_layout(vec2(right_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(right_w);
                right(ui);
            });
        });
    }
}

fn legend_dot(ui: &mut Ui, color: Color32, text: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 5.0, color);
    ui.label(RichText::new(text).size(12.0).color(MUTED));
}
