//! The "Workout" page: today's plan, live logging and the rest timer.

use super::*;

enum CardAction {
    Remove(usize),
    Swap(usize),
    Toggle(usize),
    AddSet(usize),
    RemoveSet(usize),
    SetChecked,
    OpenAdd,
}

impl ForgeApp {
    pub(super) fn page_workout(&mut self, ui: &mut Ui) {
        let now = now_secs();
        let units = self.data.settings.units;
        let rec = recovery(&self.data.history, now);
        let live = self.active.is_some();

        let kicker_text = if live {
            "WORKOUT IN PROGRESS".to_string()
        } else {
            fmt_long_date(now).to_uppercase()
        };
        let total_sets = self.plan.total_sets();
        let done_sets: usize = self
            .plan
            .exercises
            .iter()
            .map(|e| e.sets.iter().filter(|s| s.done).count())
            .sum();
        let elapsed = self.active.as_ref().map(|a| now - a.start).unwrap_or(0);

        let mut start_clicked = false;
        let mut finish = false;
        let mut discard = false;
        let mut regen = false;
        let mut focus = self.focus;
        let mut duration = self.data.settings.duration_min;

        // Header
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(kicker_text)
                        .size(12.5)
                        .strong()
                        .color(if live { ACCENT } else { MUTED }),
                );
                ui.label(RichText::new(self.plan.title.as_str()).size(30.0).strong().color(TEXT));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if live {
                    if ui.add(accent_button("Finish workout")).clicked() {
                        finish = true;
                    }
                    if ui.add(ghost_button("Discard")).clicked() {
                        discard = true;
                    }
                    ui.add_space(14.0);
                    ui.label(RichText::new(fmt_clock(elapsed)).monospace().size(28.0).color(TEXT));
                } else {
                    let can_start = !self.plan.exercises.is_empty();
                    if ui.add_enabled(can_start, accent_button("Start workout")).clicked() {
                        start_clicked = true;
                    }
                }
            });
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            pill(ui, &format!("{} exercises", self.plan.exercises.len()), TEXT);
            pill(ui, &format!("{} sets", total_sets), TEXT);
            pill(ui, &format!("about {} min", self.plan.est_minutes()), TEXT);
            pill(ui, self.data.settings.goal.name(), ACCENT_TEXT);
            if live {
                pill(ui, &format!("{} / {} sets done", done_sets, total_sets), GOOD);
            }
        });

        if !live {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Focus").color(MUTED));
                egui::ComboBox::from_id_salt("focus_combo")
                    .selected_text(focus.name())
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        for f in Focus::ALL {
                            ui.selectable_value(&mut focus, f, f.name());
                        }
                    });
                ui.add_space(10.0);
                ui.label(RichText::new("Length").color(MUTED));
                egui::ComboBox::from_id_salt("duration_combo")
                    .selected_text(format!("{} min", duration))
                    .width(90.0)
                    .show_ui(ui, |ui| {
                        for d in [20u32, 30, 45, 60, 75, 90] {
                            ui.selectable_value(&mut duration, d, format!("{} min", d));
                        }
                    });
                ui.add_space(10.0);
                if ui
                    .add(ghost_button("New suggestion"))
                    .on_hover_text("Build a different workout with the same settings")
                    .clicked()
                {
                    regen = true;
                }
            });
        }

        // Rest timer
        let mut skip_rest = false;
        if let Some(until) = self.active.as_ref().and_then(|a| a.rest_until) {
            let left = until - now_f();
            if left > 0.0 {
                ui.add_space(10.0);
                egui::Frame::default()
                    .fill(ACCENT_SOFT)
                    .stroke(Stroke::new(1.0, ACCENT))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(16, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Rest").size(15.0).strong().color(ACCENT_TEXT));
                            ui.label(
                                RichText::new(fmt_clock(left.ceil() as i64))
                                    .monospace()
                                    .size(22.0)
                                    .color(TEXT),
                            );
                            ui.label(RichText::new("before your next set").color(MUTED));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.add(small_button("Skip rest")).clicked() {
                                    skip_rest = true;
                                }
                            });
                        });
                    });
            }
        }

        ui.add_space(14.0);
        let avail = ui.available_width();
        let right_w = (avail * 0.32).clamp(260.0, 340.0);
        let left_w = (avail - right_w - 20.0).max(300.0);
        let mut acts: Vec<CardAction> = Vec::new();

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(left_w);
                if self.plan.exercises.is_empty() {
                    card_frame().show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(left_w - 34.0);
                            section_title(ui, "No exercises yet");
                            ui.label(
                                RichText::new("Add exercises yourself or ask for a new suggestion.")
                                    .color(MUTED),
                            );
                        });
                    });
                    ui.add_space(10.0);
                }
                let n = self.plan.exercises.len();
                for i in 0..n {
                    let expanded = self.expanded.contains(&i);
                    let last = last_session_text(&self.data.history, &self.plan.exercises[i].id, units);
                    let ex = &mut self.plan.exercises[i];
                    acts.extend(exercise_card(ui, i, ex, units, live, expanded, last));
                    ui.add_space(10.0);
                }
                if ui
                    .add(ghost_button("+  Add exercise").min_size(vec2(left_w, 42.0)))
                    .clicked()
                {
                    acts.push(CardAction::OpenAdd);
                }
            });
            ui.add_space(20.0);
            ui.allocate_ui_with_layout(vec2(right_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(right_w);
                self.workout_sidebar(ui, &rec, right_w);
            });
        });

        // Apply what happened this frame.
        for a in acts {
            match a {
                CardAction::Remove(i) => {
                    if i < self.plan.exercises.len() {
                        self.plan.exercises.remove(i);
                        self.expanded.clear();
                    }
                }
                CardAction::Swap(i) => self.swap_idx = Some(i),
                CardAction::Toggle(i) => {
                    if !self.expanded.remove(&i) {
                        self.expanded.insert(i);
                    }
                }
                CardAction::AddSet(i) => {
                    if let Some(e) = self.plan.exercises.get_mut(i) {
                        let base = e
                            .sets
                            .last()
                            .copied()
                            .unwrap_or(SetLog { weight: 0.0, reps: 8, done: false });
                        e.sets.push(SetLog { done: false, ..base });
                    }
                }
                CardAction::RemoveSet(i) => {
                    if let Some(e) = self.plan.exercises.get_mut(i) {
                        if e.sets.len() > 1 {
                            e.sets.pop();
                        }
                    }
                }
                CardAction::SetChecked => {
                    let rest = self.data.settings.goal.rest_secs();
                    if let Some(a) = self.active.as_mut() {
                        a.rest_until = Some(now_f() + rest);
                    }
                }
                CardAction::OpenAdd => {
                    self.add_open = true;
                    self.add_search.clear();
                }
            }
        }
        if skip_rest {
            if let Some(a) = self.active.as_mut() {
                a.rest_until = None;
            }
        }
        if focus != self.focus || duration != self.data.settings.duration_min {
            self.focus = focus;
            self.data.settings.duration_min = duration;
            regen = true;
        }
        if regen && !live {
            self.regenerate();
        }
        if start_clicked {
            self.start_workout();
        }
        if discard {
            self.discard_workout();
        }
        if finish {
            self.finish_workout();
        }
    }

    fn workout_sidebar(&self, ui: &mut Ui, rec: &[f32; MUSCLE_COUNT], w: f32) {
        let mut primary: Vec<Muscle> = Vec::new();
        let mut secondary: Vec<Muscle> = Vec::new();
        for e in &self.plan.exercises {
            if let Some(d) = data::find(&e.id) {
                if !primary.contains(&d.primary) {
                    primary.push(d.primary);
                }
                for m in d.secondary {
                    if !secondary.contains(m) {
                        secondary.push(*m);
                    }
                }
            }
        }
        secondary.retain(|m| !primary.contains(m));
        let inner = w - 34.0;
        let helper = Color32::from_rgb(140, 72, 62);

        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(inner);
                section_title(ui, "Muscles worked");
                let fw = ((inner - 12.0) / 2.0).min(150.0);
                let color_of = |m: Muscle| -> Color32 {
                    if primary.contains(&m) {
                        ACCENT
                    } else if secondary.contains(&m) {
                        helper
                    } else {
                        body::IDLE
                    }
                };
                ui.horizontal(|ui| {
                    let pad = ((inner - fw * 2.0 - 12.0) / 2.0).max(0.0);
                    ui.add_space(pad);
                    for back in [false, true] {
                        let (resp, hov) = body::figure(ui, vec2(fw, body::height_for(fw)), back, &color_of);
                        if let Some(m) = hov {
                            let role = if primary.contains(&m) {
                                "main target"
                            } else if secondary.contains(&m) {
                                "helper muscle"
                            } else {
                                "not trained today"
                            };
                            resp.on_hover_text(format!("{}: {}", m.name(), role));
                        }
                    }
                });
                ui.horizontal(|ui| {
                    legend_dot(ui, ACCENT, "Main target");
                    ui.add_space(10.0);
                    legend_dot(ui, helper, "Helper");
                });
            });
        });
        ui.add_space(12.0);
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(inner);
                section_title(ui, "Readiness of today's muscles");
                if primary.is_empty() {
                    ui.label(RichText::new("Add exercises to see readiness.").color(MUTED));
                }
                for m in &primary {
                    rec_bar(ui, m.name(), rec[m.index()], None, inner);
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "Workouts are built around your freshest muscles. Weights come from your recent \
                         sets and rise a little each time you complete them.",
                    )
                    .size(12.0)
                    .color(MUTED),
                );
            });
        });
    }
}

fn last_session_text(history: &[WorkoutLog], id: &str, units: Units) -> Option<String> {
    let d = data::find(id)?;
    let ss = sessions(history, id);
    let (t, e) = *ss.last()?;
    let best = e.sets.iter().filter(|s| s.done).max_by(|a, b| {
        epley(a.weight, a.reps)
            .partial_cmp(&epley(b.weight, b.reps))
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;
    let now = now_secs();
    let ago = fmt_ago(t, now).to_lowercase();
    Some(match d.kind {
        Kind::Weighted => format!(
            "Last time ({}): {} x {} · {} sets",
            ago,
            units.fmt(best.weight),
            best.reps,
            e.sets.len()
        ),
        Kind::Timed => format!("Last time ({}): {} s hold", ago, best.reps),
        Kind::Bodyweight => format!("Last time ({}): {} reps", ago, best.reps),
    })
}

fn set_summary(ex: &ExerciseLog, def: &ExerciseDef, units: Units) -> String {
    let n = ex.sets.len();
    if n == 0 {
        return "No sets".to_string();
    }
    let reps_same = ex.sets.iter().all(|s| s.reps == ex.sets[0].reps);
    let w_same = ex.sets.iter().all(|s| (s.weight - ex.sets[0].weight).abs() < 0.01);
    let reps_txt = if reps_same {
        format!("{}", ex.sets[0].reps)
    } else {
        let lo = ex.sets.iter().map(|s| s.reps).min().unwrap_or(0);
        let hi = ex.sets.iter().map(|s| s.reps).max().unwrap_or(0);
        format!("{}-{}", lo, hi)
    };
    match def.kind {
        Kind::Weighted => {
            let w_txt = if w_same {
                units.fmt(ex.sets[0].weight)
            } else {
                let lo = ex.sets.iter().map(|s| s.weight).fold(f32::MAX, f32::min);
                let hi = ex.sets.iter().map(|s| s.weight).fold(0.0f32, f32::max);
                format!("{} - {}", units.fmt(lo), units.fmt(hi))
            };
            format!("{} sets x {} reps · {}", n, reps_txt, w_txt)
        }
        Kind::Bodyweight => format!("{} sets x {} reps · bodyweight", n, reps_txt),
        Kind::Timed => format!("{} sets x {} s hold", n, reps_txt),
    }
}

fn exercise_card(
    ui: &mut Ui,
    i: usize,
    ex: &mut ExerciseLog,
    units: Units,
    live: bool,
    expanded: bool,
    last: Option<String>,
) -> Vec<CardAction> {
    let mut acts: Vec<CardAction> = Vec::new();
    let Some(def) = data::find(&ex.id) else { return acts };
    let all_done = live && !ex.sets.is_empty() && ex.sets.iter().all(|s| s.done);
    let summary = set_summary(ex, def, units);
    card_frame()
        .stroke(Stroke::new(
            if all_done { 1.5 } else { 1.0 },
            if all_done { GOOD } else { BORDER },
        ))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    thumb(ui, def, 54.0);
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(def.name).size(16.5).strong().color(TEXT));
                        ui.label(
                            RichText::new(format!("{} · {}", def.primary.name(), def.equipment.name()))
                                .size(12.5)
                                .color(MUTED),
                        );
                        ui.label(RichText::new(summary.as_str()).size(13.5).color(ACCENT_TEXT));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(small_button("Remove")).clicked() {
                            acts.push(CardAction::Remove(i));
                        }
                        if ui
                            .add(small_button("Swap"))
                            .on_hover_text("Pick another exercise for the same muscle")
                            .clicked()
                        {
                            acts.push(CardAction::Swap(i));
                        }
                        if !live
                            && ui
                                .add(small_button(if expanded { "Hide sets" } else { "Edit sets" }))
                                .clicked()
                        {
                            acts.push(CardAction::Toggle(i));
                        }
                        if all_done {
                            ui.label(RichText::new("Done").strong().color(GOOD));
                        }
                    });
                });
                if let Some(l) = &last {
                    ui.label(RichText::new(l.as_str()).size(12.0).color(MUTED));
                }
                if live || expanded {
                    ui.add_space(4.0);
                    sets_grid(ui, i, &mut *ex, def, units, live, &mut acts);
                    let many = ex.sets.len() > 1;
                    ui.horizontal(|ui| {
                        if ui.add(small_button("+ Add set")).clicked() {
                            acts.push(CardAction::AddSet(i));
                        }
                        if many && ui.add(small_button("- Remove set")).clicked() {
                            acts.push(CardAction::RemoveSet(i));
                        }
                    });
                }
            });
        });
    acts
}

fn sets_grid(
    ui: &mut Ui,
    i: usize,
    ex: &mut ExerciseLog,
    def: &ExerciseDef,
    units: Units,
    live: bool,
    acts: &mut Vec<CardAction>,
) {
    let weight_header = match def.kind {
        Kind::Weighted => format!("WEIGHT ({})", units.label().to_uppercase()),
        _ => "WEIGHT".to_string(),
    };
    let reps_header = if def.kind == Kind::Timed { "SECONDS" } else { "REPS" };
    egui::Grid::new(("sets_grid", i))
        .num_columns(4)
        .spacing(vec2(22.0, 6.0))
        .show(ui, |ui| {
            ui.label(RichText::new("SET").size(11.0).strong().color(MUTED));
            ui.label(RichText::new(weight_header).size(11.0).strong().color(MUTED));
            ui.label(RichText::new(reps_header).size(11.0).strong().color(MUTED));
            ui.label(RichText::new(if live { "DONE" } else { "" }).size(11.0).strong().color(MUTED));
            ui.end_row();
            for (j, s) in ex.sets.iter_mut().enumerate() {
                let num_color = if s.done { GOOD } else { TEXT };
                ui.label(RichText::new(format!("{}", j + 1)).strong().color(num_color));
                match def.kind {
                    Kind::Weighted => {
                        let mut w = units.from_lb(s.weight);
                        let r = ui.add(
                            egui::DragValue::new(&mut w)
                                .speed(0.5)
                                .range(0.0..=2000.0)
                                .max_decimals(1)
                                .suffix(format!(" {}", units.label())),
                        );
                        if r.changed() {
                            s.weight = units.to_lb(w);
                        }
                    }
                    _ => {
                        ui.label(RichText::new("Bodyweight").color(MUTED));
                    }
                }
                let suffix = if def.kind == Kind::Timed { " s" } else { " reps" };
                ui.add(egui::DragValue::new(&mut s.reps).speed(0.2).range(0..=600).suffix(suffix));
                if live {
                    if done_toggle(ui, s.done).clicked() {
                        s.done = !s.done;
                        if s.done {
                            acts.push(CardAction::SetChecked);
                        }
                    }
                } else {
                    ui.label("");
                }
                ui.end_row();
            }
        });
}
