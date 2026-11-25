//! Recovery, exercise library and log pages.

use super::*;

impl ForgeApp {
    // ------------------------------------------------------------ recovery page

    pub(super) fn page_recovery(&mut self, ui: &mut Ui) {
        let now = now_secs();
        let rec = recovery(&self.data.history, now);
        page_header(
            ui,
            "Muscle Recovery",
            "How rested each muscle group is, based on the sets you've logged. Fresh muscles get picked first.",
        );
        let avail = ui.available_width();
        let right_w = (avail * 0.4).clamp(300.0, 420.0);
        let left_w = (avail - right_w - 20.0).max(320.0);
        let history = &self.data.history;

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(left_w);
                card_frame().show(ui, |ui| {
                    ui.vertical(|ui| {
                        let inner = left_w - 34.0;
                        ui.set_width(inner);
                        let fw = ((inner - 30.0) / 2.0).min(220.0);
                        let color_of = |m: Muscle| -> Color32 { rec_color(rec[m.index()]) };
                        ui.horizontal(|ui| {
                            let pad = ((inner - fw * 2.0 - 30.0) / 2.0).max(0.0);
                            ui.add_space(pad);
                            for back in [false, true] {
                                let (resp, hov) =
                                    body::figure(ui, vec2(fw, body::height_for(fw)), back, &color_of);
                                if let Some(m) = hov {
                                    resp.on_hover_text(format!(
                                        "{}: {:.0}% recovered",
                                        m.name(),
                                        rec[m.index()]
                                    ));
                                }
                                ui.add_space(22.0);
                            }
                        });
                        ui.add_space(8.0);
                        // Gradient legend
                        let (rect, _) = ui.allocate_exact_size(vec2(inner, 10.0), Sense::hover());
                        let steps = 40;
                        for k in 0..steps {
                            let x0 = rect.left() + rect.width() * k as f32 / steps as f32;
                            let x1 = rect.left() + rect.width() * (k + 1) as f32 / steps as f32 + 0.5;
                            let r = Rect::from_min_max(
                                pos2(x0, rect.top()),
                                pos2(x1.min(rect.right()), rect.bottom()),
                            );
                            ui.painter().rect_filled(
                                r,
                                CornerRadius::same(0),
                                rec_color(k as f32 * 100.0 / (steps - 1) as f32),
                            );
                        }
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Needs rest").size(12.0).color(MUTED));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new("Fully recovered").size(12.0).color(MUTED));
                            });
                        });
                    });
                });
                ui.add_space(12.0);
                let ready: Vec<&str> = Muscle::ALL
                    .iter()
                    .filter(|m| rec[m.index()] >= 90.0)
                    .map(|m| m.name())
                    .collect();
                let tired: Vec<&str> = Muscle::ALL
                    .iter()
                    .filter(|m| rec[m.index()] < 60.0)
                    .map(|m| m.name())
                    .collect();
                let overall = rec.iter().sum::<f32>() / MUSCLE_COUNT as f32;
                card_frame().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_width(left_w - 34.0);
                        kicker(ui, "Overall readiness", MUTED);
                        ui.label(
                            RichText::new(format!("{:.0}%", overall))
                                .size(30.0)
                                .strong()
                                .color(rec_color(overall)),
                        );
                        let ready_text = if ready.is_empty() {
                            "Nothing is fully fresh yet. A light day or rest could help.".to_string()
                        } else {
                            format!("Ready to train: {}", ready.join(", "))
                        };
                        ui.label(RichText::new(ready_text).color(TEXT));
                        if !tired.is_empty() {
                            ui.label(
                                RichText::new(format!("Still recovering: {}", tired.join(", "))).color(MUTED),
                            );
                        }
                    });
                });
            });
            ui.add_space(20.0);
            ui.allocate_ui_with_layout(vec2(right_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(right_w);
                card_frame().show(ui, |ui| {
                    ui.vertical(|ui| {
                        let inner = right_w - 34.0;
                        ui.set_width(inner);
                        section_title(ui, "All muscle groups");
                        let mut order: Vec<Muscle> = Muscle::ALL.to_vec();
                        order.sort_by(|a, b| {
                            rec[a.index()]
                                .partial_cmp(&rec[b.index()])
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        for m in order {
                            let sub = match last_trained(history, m) {
                                Some(t) => format!("trained {}", fmt_ago(t, now).to_lowercase()),
                                None => "not trained recently".to_string(),
                            };
                            rec_bar(ui, m.name(), rec[m.index()], Some(sub.as_str()), inner);
                        }
                    });
                });
            });
        });
    }

    // ------------------------------------------------------------ exercise library

    pub(super) fn page_exercises(&mut self, ui: &mut Ui) {
        let settings = self.data.settings.clone();
        let available = EXERCISES.iter().filter(|d| settings.has(d.equipment)).count();
        page_header(
            ui,
            "Exercise Library",
            &format!(
                "{} exercises, {} of them work with your equipment.",
                EXERCISES.len(),
                available
            ),
        );
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.lib_search)
                    .hint_text("Search exercises")
                    .desired_width(260.0),
            );
            ui.add_space(8.0);
            ui.checkbox(&mut self.lib_only_mine, "Only my equipment");
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if chip(ui, self.lib_filter.is_none(), "All").clicked() {
                self.lib_filter = None;
            }
            for m in Muscle::ALL {
                if chip(ui, self.lib_filter == Some(m), m.name()).clicked() {
                    self.lib_filter = Some(m);
                }
            }
        });
        ui.add_space(12.0);

        let q = self.lib_search.trim().to_lowercase();
        let only_mine = self.lib_only_mine;
        let filter = self.lib_filter;
        let list: Vec<&'static ExerciseDef> = EXERCISES
            .iter()
            .filter(|d| {
                (!only_mine || settings.has(d.equipment))
                    && filter.map_or(true, |m| d.primary == m || d.secondary.contains(&m))
                    && (q.is_empty()
                        || d.name.to_lowercase().contains(&q)
                        || d.primary.name().to_lowercase().contains(&q)
                        || d.equipment.name().to_lowercase().contains(&q))
            })
            .collect();

        let avail = ui.available_width();
        let left_w = (avail * 0.42).clamp(300.0, 420.0);
        let right_w = (avail - left_w - 20.0).max(320.0);
        let mut add_to_plan: Option<&'static ExerciseDef> = None;
        let mut select: Option<&'static str> = None;
        let current = self.lib_selected;

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(left_w);
                if list.is_empty() {
                    ui.label(RichText::new("No exercises match.").color(MUTED));
                }
                for d in list.iter().copied() {
                    let selected = current == d.id;
                    let resp = egui::Frame::default()
                        .fill(if selected { CARD2 } else { CARD })
                        .stroke(Stroke::new(1.0, if selected { ACCENT } else { BORDER }))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.set_width(left_w - 22.0);
                                thumb(ui, d, 40.0);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(d.name).size(14.5).strong().color(TEXT));
                                    ui.label(
                                        RichText::new(format!(
                                            "{} · {}",
                                            d.primary.name(),
                                            d.equipment.name()
                                        ))
                                        .size(12.0)
                                        .color(MUTED),
                                    );
                                });
                            });
                        })
                        .response;
                    if resp
                        .interact(Sense::click())
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        select = Some(d.id);
                    }
                    ui.add_space(2.0);
                }
            });
            ui.add_space(20.0);
            ui.allocate_ui_with_layout(vec2(right_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(right_w);
                if let Some(d) = data::find(current) {
                    let in_plan = self.plan.exercises.iter().any(|e| e.id == d.id);
                    if self.exercise_detail(ui, d, right_w, in_plan) {
                        add_to_plan = Some(d);
                    }
                }
            });
        });

        if let Some(id) = select {
            self.lib_selected = id;
        }
        if let Some(d) = add_to_plan {
            let e = prescribe(d, &self.data.settings, &self.data.history);
            self.plan.exercises.push(e);
            self.show_toast(format!("Added {} to today's workout.", d.name));
        }
    }

    /// Returns true when "Add to today's workout" was clicked.
    fn exercise_detail(&self, ui: &mut Ui, d: &'static ExerciseDef, w: f32, in_plan: bool) -> bool {
        let units = self.data.settings.units;
        let mut clicked = false;
        let helper = Color32::from_rgb(140, 72, 62);
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                let inner = w - 34.0;
                ui.set_width(inner);
                ui.horizontal(|ui| {
                    thumb(ui, d, 64.0);
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(d.name).size(24.0).strong().color(TEXT));
                        ui.horizontal(|ui| {
                            pill(ui, d.primary.name(), d.primary.group().color());
                            pill(ui, d.equipment.name(), MUTED);
                            pill(ui, if d.compound { "Compound" } else { "Isolation" }, MUTED);
                        });
                    });
                });
                ui.add_space(8.0);
                ui.label(RichText::new(d.cue).size(14.0).color(TEXT));
                if !d.secondary.is_empty() {
                    let names: Vec<&str> = d.secondary.iter().map(|m| m.name()).collect();
                    ui.label(
                        RichText::new(format!("Also works: {}", names.join(", ")))
                            .size(13.0)
                            .color(MUTED),
                    );
                }
                ui.add_space(8.0);

                let sessions_n = sessions(&self.data.history, d.id).len();
                let best = best_set(&self.data.history, d);
                ui.horizontal_top(|ui| {
                    let fw = 92.0;
                    let color_of = |m: Muscle| -> Color32 {
                        if m == d.primary {
                            ACCENT
                        } else if d.secondary.contains(&m) {
                            helper
                        } else {
                            body::IDLE
                        }
                    };
                    for back in [false, true] {
                        let _ = body::figure(ui, vec2(fw, body::height_for(fw)), back, &color_of);
                    }
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        kicker(ui, "Times logged", MUTED);
                        ui.label(RichText::new(sessions_n.to_string()).size(22.0).strong().color(TEXT));
                        ui.add_space(4.0);
                        match (d.kind, best) {
                            (Kind::Weighted, Some((t, s))) => {
                                kicker(ui, "Best set", MUTED);
                                ui.label(
                                    RichText::new(format!("{} x {}", units.fmt(s.weight), s.reps))
                                        .size(18.0)
                                        .strong()
                                        .color(TEXT),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "Estimated 1-rep max {} · {}",
                                        units.fmt(epley(s.weight, s.reps)),
                                        fmt_short_date(t)
                                    ))
                                    .size(12.0)
                                    .color(MUTED),
                                );
                            }
                            (_, Some((t, s))) => {
                                kicker(ui, "Best set", MUTED);
                                let unit = if d.kind == Kind::Timed { "sec" } else { "reps" };
                                ui.label(
                                    RichText::new(format!("{} {}", s.reps, unit))
                                        .size(18.0)
                                        .strong()
                                        .color(TEXT),
                                );
                                ui.label(RichText::new(fmt_short_date(t)).size(12.0).color(MUTED));
                            }
                            (_, None) => {
                                ui.label(RichText::new("You haven't logged this one yet.").color(MUTED));
                            }
                        }
                    });
                });
                ui.add_space(8.0);
                if sessions_n > 0 {
                    let label = match d.kind {
                        Kind::Weighted => "Estimated 1-rep max over time",
                        Kind::Timed => "Best hold over time",
                        Kind::Bodyweight => "Best reps over time",
                    };
                    section_title(ui, label);
                    let pts: Vec<(i64, f32)> = metric_history(&self.data.history, d)
                        .into_iter()
                        .map(|(t, v)| (t, if d.kind == Kind::Weighted { units.from_lb(v) } else { v }))
                        .collect();
                    let unit_label = match d.kind {
                        Kind::Weighted => units.label(),
                        Kind::Timed => "s",
                        Kind::Bodyweight => "reps",
                    };
                    charts::line_chart(ui, vec2(inner, 170.0), &pts, ACCENT, &|v: f32| {
                        format!("{:.0} {}", v, unit_label)
                    });
                    ui.add_space(8.0);
                }
                let text = if in_plan { "Already in today's workout" } else { "Add to today's workout" };
                if ui
                    .add_enabled(!in_plan, accent_button(text).min_size(vec2(inner, 40.0)))
                    .clicked()
                {
                    clicked = true;
                }
            });
        });
        clicked
    }

    // ------------------------------------------------------------ log page

    pub(super) fn page_log(&mut self, ui: &mut Ui) {
        let now = now_secs();
        let units = self.data.settings.units;
        page_header(
            ui,
            "Workout Log",
            &format!(
                "{} workouts logged. Click one to see every set.",
                self.data.history.len()
            ),
        );
        let w = ui.available_width();
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(w - 34.0);
                section_title(ui, "Last 6 weeks");
                calendar(ui, &self.data.history, now);
            });
        });
        ui.add_space(14.0);

        let mut toggle: Option<i64> = None;
        let mut delete: Option<i64> = None;
        if self.data.history.is_empty() {
            ui.label(
                RichText::new("No workouts yet. Finish one and it will show up here.").color(MUTED),
            );
        }
        let selected = self.log_selected;
        for wl in self.data.history.iter().rev() {
            let sel = selected == Some(wl.start);
            card_frame()
                .stroke(Stroke::new(1.0, if sel { ACCENT } else { BORDER }))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_width(w - 34.0);
                        let header = ui
                            .horizontal(|ui| {
                                ui.vertical(|ui| {
                                    kicker(
                                        ui,
                                        &format!("{} · {}", fmt_day_label(wl.start), fmt_ago(wl.start, now)),
                                        MUTED,
                                    );
                                    ui.label(
                                        RichText::new(wl.title.as_str()).size(18.0).strong().color(TEXT),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(if sel { "Hide" } else { "Details" })
                                            .size(12.5)
                                            .color(ACCENT_TEXT),
                                    );
                                    ui.add_space(8.0);
                                    pill(ui, &units.fmt_big(wl.volume()), ACCENT_TEXT);
                                    pill(ui, &format!("{} sets", wl.set_count()), TEXT);
                                    pill(ui, &format!("{} exercises", wl.exercises.len()), TEXT);
                                    pill(ui, &fmt_minutes(wl.duration_secs), TEXT);
                                });
                            })
                            .response;
                        let hit = ui.interact(
                            header.rect,
                            egui::Id::new(("log_header", wl.start)),
                            Sense::click(),
                        );
                        if hit.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            toggle = Some(wl.start);
                        }
                        if sel {
                            ui.add_space(6.0);
                            for e in &wl.exercises {
                                let Some(d) = data::find(&e.id) else { continue };
                                ui.separator();
                                ui.horizontal_top(|ui| {
                                    thumb(ui, d, 34.0);
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(d.name).size(14.5).strong().color(TEXT));
                                        let sets: Vec<String> = e
                                            .sets
                                            .iter()
                                            .map(|s| match d.kind {
                                                Kind::Weighted => {
                                                    format!("{} x {}", units.fmt(s.weight), s.reps)
                                                }
                                                Kind::Timed => format!("{} s", s.reps),
                                                Kind::Bodyweight => format!("{} reps", s.reps),
                                            })
                                            .collect();
                                        ui.label(
                                            RichText::new(sets.join("   ·   ")).size(13.0).color(MUTED),
                                        );
                                    });
                                });
                            }
                            ui.add_space(8.0);
                            if ui.add(small_button("Delete this workout")).clicked() {
                                delete = Some(wl.start);
                            }
                        }
                    });
                });
            ui.add_space(8.0);
        }
        if let Some(t) = toggle {
            self.log_selected = if self.log_selected == Some(t) { None } else { Some(t) };
        }
        if let Some(t) = delete {
            self.data.history.retain(|w| w.start != t);
            self.log_selected = None;
            self.show_toast("Workout deleted.".to_string());
            if self.active.is_none() {
                self.regenerate();
            }
        }
    }
}

fn calendar(ui: &mut Ui, history: &[WorkoutLog], now: i64) {
    let today = day_index(now);
    let start = today - weekday_mon(today) as i64 - 7 * 5;
    let cell = 26.0;
    let gap = 6.0;
    let label_w = 58.0;
    let size = vec2(label_w + 7.0 * (cell + gap), 20.0 + 6.0 * (cell + gap));
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter_at(rect);
    let mut vols: HashMap<i64, f32> = HashMap::new();
    for w in history {
        *vols.entry(day_index(w.start)).or_insert(0.0) += w.volume().max(1.0);
    }
    let maxv = vols.values().copied().fold(1.0f32, f32::max);
    for (c, name) in ["M", "T", "W", "T", "F", "S", "S"].iter().enumerate() {
        p.text(
            pos2(rect.left() + label_w + c as f32 * (cell + gap) + cell / 2.0, rect.top() + 8.0),
            Align2::CENTER_CENTER,
            *name,
            FontId::proportional(11.0),
            MUTED,
        );
    }
    let hover = resp.hover_pos();
    let mut tip: Option<String> = None;
    for wk in 0..6i64 {
        let y = rect.top() + 20.0 + wk as f32 * (cell + gap);
        p.text(
            pos2(rect.left(), y + cell / 2.0),
            Align2::LEFT_CENTER,
            fmt_short_date(day_to_ts(start + wk * 7)),
            FontId::proportional(11.0),
            MUTED,
        );
        for c in 0..7i64 {
            let day = start + wk * 7 + c;
            let x = rect.left() + label_w + c as f32 * (cell + gap);
            let r = Rect::from_min_size(pos2(x, y), vec2(cell, cell));
            let fill = match vols.get(&day) {
                Some(v) => lerp_color(Color32::from_rgb(120, 52, 42), ACCENT, (*v / maxv).sqrt()),
                None if day > today => BG,
                None => CARD2,
            };
            p.rect_filled(r, CornerRadius::same(6), fill);
            if day == today {
                p.circle_filled(pos2(r.center().x, r.bottom() - 5.0), 2.5, TEXT);
            }
            if let Some(hp) = hover {
                if r.contains(hp) {
                    let names: Vec<&str> = history
                        .iter()
                        .filter(|w| day_index(w.start) == day)
                        .map(|w| w.title.as_str())
                        .collect();
                    tip = Some(if names.is_empty() {
                        format!("{}: rest day", fmt_day_label(day_to_ts(day)))
                    } else {
                        format!("{}: {}", fmt_day_label(day_to_ts(day)), names.join(", "))
                    });
                }
            }
        }
    }
    if let Some(t) = tip {
        resp.on_hover_text(t);
    }
}
