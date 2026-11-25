//! Progress and Settings pages.

use super::*;

impl ForgeApp {
    // ------------------------------------------------------------ progress page

    pub(super) fn page_progress(&mut self, ui: &mut Ui) {
        let now = now_secs();
        let units = self.data.settings.units;
        page_header(ui, "Progress", "Training volume, strength trends and personal records.");

        let month_ago = now - 30 * DAY;
        let this_week = week_start_day(now);
        let (w30, vol30, wk_count, streak, total_minutes) = {
            let hist = &self.data.history;
            let w30 = hist.iter().filter(|w| w.start >= month_ago).count();
            let vol30: f32 = hist
                .iter()
                .filter(|w| w.start >= month_ago)
                .map(|w| w.volume())
                .sum();
            let wk_count = hist
                .iter()
                .filter(|w| week_start_day(w.start) == this_week)
                .count();
            let weeks: HashSet<i64> = hist.iter().map(|w| week_start_day(w.start)).collect();
            let mut streak = 0usize;
            let mut wk = if weeks.contains(&this_week) { this_week } else { this_week - 7 };
            while weeks.contains(&wk) {
                streak += 1;
                wk -= 7;
            }
            let total_minutes: i64 = hist
                .iter()
                .filter(|w| w.start >= month_ago)
                .map(|w| w.duration_secs)
                .sum::<i64>()
                / 60;
            (w30, vol30, wk_count, streak, total_minutes)
        };

        let avail = ui.available_width();
        let gap = 12.0;
        let cw = (avail - gap * 3.0) / 4.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            stat_card(
                ui,
                cw,
                "Last 30 days",
                &format!("{} workouts", w30),
                &format!("{} min of training", total_minutes),
                TEXT,
            );
            stat_card(ui, cw, "This week", &format!("{}", wk_count), "workouts so far", ACCENT_TEXT);
            stat_card(ui, cw, "Volume, 30 days", &units.fmt_big(vol30), "weight x reps, all sets", TEXT);
            stat_card(
                ui,
                cw,
                "Streak",
                &format!("{} week{}", streak, if streak == 1 { "" } else { "s" }),
                "in a row with a workout",
                GOOD,
            );
        });
        ui.add_space(14.0);

        // Weekly volume
        let mut bars: Vec<(String, f32)> = Vec::new();
        for k in (0..8i64).rev() {
            let wsd = this_week - 7 * k;
            let v: f32 = self
                .data
                .history
                .iter()
                .filter(|w| week_start_day(w.start) == wsd)
                .map(|w| w.volume())
                .sum();
            bars.push((fmt_short_date(day_to_ts(wsd)), units.from_lb(v)));
        }
        let half = (avail - 16.0) / 2.0;

        // Weighted exercises with history, most-logged first.
        let mut counts: HashMap<&'static str, usize> = HashMap::new();
        for w in &self.data.history {
            for e in &w.exercises {
                if let Some(d) = data::find(&e.id) {
                    if d.kind == Kind::Weighted {
                        *counts.entry(d.id).or_insert(0) += 1;
                    }
                }
            }
        }
        let mut options: Vec<(&'static str, usize)> = counts.into_iter().collect();
        options.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let current: Option<&'static str> = match self.progress_ex {
            Some(id) if options.iter().any(|o| o.0 == id) => Some(id),
            _ => options.first().map(|o| o.0),
        };
        let mut picked = current;
        let trend: Vec<(i64, f32)> = match current.and_then(|id| data::find(id)) {
            Some(d) => metric_history(&self.data.history, d)
                .into_iter()
                .map(|(t, v)| (t, units.from_lb(v)))
                .collect(),
            None => Vec::new(),
        };

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            card_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(half - 34.0);
                    section_title(ui, "Weekly volume");
                    ui.label(
                        RichText::new(format!("Total weight lifted per week ({})", units.label()))
                            .size(12.0)
                            .color(MUTED),
                    );
                    charts::bar_chart(ui, vec2(half - 34.0, 210.0), &bars, ACCENT, &|v: f32| {
                        if v >= 10_000.0 {
                            format!("{:.0}k", v / 1000.0)
                        } else if v >= 1000.0 {
                            format!("{:.1}k", v / 1000.0)
                        } else {
                            format!("{:.0}", v)
                        }
                    });
                });
            });
            card_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(half - 34.0);
                    ui.horizontal(|ui| {
                        section_title(ui, "Strength trend");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let label = match picked.and_then(|id| data::find(id)) {
                                Some(d) => d.name,
                                None => "No exercises yet",
                            };
                            egui::ComboBox::from_id_salt("trend_combo")
                                .selected_text(label)
                                .width(210.0)
                                .show_ui(ui, |ui| {
                                    for (id, n) in options.iter() {
                                        if let Some(d) = data::find(id) {
                                            ui.selectable_value(
                                                &mut picked,
                                                Some(*id),
                                                format!("{}  ({}x)", d.name, n),
                                            );
                                        }
                                    }
                                });
                        });
                    });
                    ui.label(
                        RichText::new(format!("Estimated 1-rep max ({})", units.label()))
                            .size(12.0)
                            .color(MUTED),
                    );
                    let ul = units.label();
                    charts::line_chart(ui, vec2(half - 34.0, 210.0), &trend, BLUE, &|v: f32| {
                        format!("{:.0} {}", v, ul)
                    });
                });
            });
        });
        if picked != current {
            self.progress_ex = picked;
        }
        ui.add_space(14.0);

        // Personal records
        let mut prs: Vec<(&'static ExerciseDef, i64, SetLog, f32)> = Vec::new();
        for d in EXERCISES.iter().filter(|d| d.kind == Kind::Weighted) {
            if let Some((t, s)) = best_set(&self.data.history, d) {
                prs.push((d, t, s, epley(s.weight, s.reps)));
            }
        }
        prs.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        prs.truncate(10);
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(avail - 34.0);
                section_title(ui, "Personal records");
                if prs.is_empty() {
                    ui.label(RichText::new("Log some weighted sets to see records here.").color(MUTED));
                }
                egui::Grid::new("pr_grid")
                    .num_columns(4)
                    .spacing(vec2(36.0, 10.0))
                    .show(ui, |ui| {
                        for h in ["EXERCISE", "BEST SET", "EST. 1-REP MAX", "DATE"] {
                            ui.label(RichText::new(h).size(11.0).strong().color(MUTED));
                        }
                        ui.end_row();
                        for (d, t, s, e1) in prs.iter() {
                            ui.horizontal(|ui| {
                                thumb(ui, d, 26.0);
                                ui.label(RichText::new(d.name).color(TEXT));
                            });
                            ui.label(
                                RichText::new(format!("{} x {}", units.fmt(s.weight), s.reps)).color(TEXT),
                            );
                            ui.label(RichText::new(units.fmt(*e1)).strong().color(ACCENT_TEXT));
                            ui.label(RichText::new(fmt_short_date(*t)).color(MUTED));
                            ui.end_row();
                        }
                    });
            });
        });
    }

    // ------------------------------------------------------------ settings page

    pub(super) fn page_settings(&mut self, ui: &mut Ui) {
        page_header(
            ui,
            "Settings",
            "Tell the planner about you and your gym. Your next workout adapts right away.",
        );
        let before = format!("{:?}", self.data.settings);
        let w = ui.available_width().min(900.0);
        let inner = w - 34.0;

        {
            let s = &mut self.data.settings;
            card_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(inner);
                    section_title(ui, "Training goal");
                    let tw = (inner - 2.0 * 10.0) / 3.0;
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        for g in Goal::ALL {
                            if option_tile(ui, s.goal == g, g.name(), g.blurb(), tw) {
                                s.goal = g;
                            }
                        }
                    });
                    ui.add_space(12.0);
                    section_title(ui, "Workout split");
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        for sp in Split::ALL {
                            if option_tile(ui, s.split == sp, sp.name(), sp.blurb(), tw) {
                                s.split = sp;
                            }
                        }
                    });
                });
            });
            ui.add_space(12.0);
            card_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(inner);
                    section_title(ui, "Experience");
                    ui.horizontal(|ui| {
                        for e in Experience::ALL {
                            if chip(ui, s.experience == e, e.name()).clicked() {
                                s.experience = e;
                            }
                        }
                    });
                    ui.label(
                        RichText::new("Sets starting weights for exercises you haven't logged yet.")
                            .size(12.0)
                            .color(MUTED),
                    );
                    ui.add_space(10.0);
                    section_title(ui, "Workout length");
                    ui.horizontal(|ui| {
                        for m in [20u32, 30, 45, 60, 75, 90] {
                            if chip(ui, s.duration_min == m, &format!("{} min", m)).clicked() {
                                s.duration_min = m;
                            }
                        }
                    });
                    ui.add_space(10.0);
                    section_title(ui, "Units");
                    ui.horizontal(|ui| {
                        if chip(ui, s.units == Units::Lb, "Pounds (lb)").clicked() {
                            s.units = Units::Lb;
                        }
                        if chip(ui, s.units == Units::Kg, "Kilograms (kg)").clicked() {
                            s.units = Units::Kg;
                        }
                    });
                });
            });
            ui.add_space(12.0);
            card_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(inner);
                    section_title(ui, "Available equipment");
                    ui.label(
                        RichText::new("Only exercises you can do with this equipment are suggested.")
                            .size(12.0)
                            .color(MUTED),
                    );
                    ui.horizontal_wrapped(|ui| {
                        for e in Equip::ALL {
                            if e == Equip::Bodyweight {
                                let _ = chip(ui, true, "Bodyweight (always)");
                                continue;
                            }
                            let on = s.equipment.contains(&e);
                            if chip(ui, on, e.name()).clicked() {
                                if on {
                                    s.equipment.retain(|x| *x != e);
                                } else {
                                    s.equipment.push(e);
                                }
                            }
                        }
                    });
                });
            });
        }

        let after = format!("{:?}", self.data.settings);
        if before != after && self.active.is_none() {
            self.regenerate();
        }

        ui.add_space(12.0);
        let mut load_sample = false;
        let mut clear = false;
        let mut ask_clear = false;
        let mut cancel_clear = false;
        let confirm = self.confirm_clear;
        card_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(inner);
                section_title(ui, "Your data");
                ui.label(
                    RichText::new("Workouts are saved on this computer automatically.")
                        .size(12.0)
                        .color(MUTED),
                );
                ui.horizontal(|ui| {
                    if ui.add(ghost_button("Replace with sample history")).clicked() {
                        load_sample = true;
                    }
                    if confirm {
                        ui.label(RichText::new("Delete every logged workout?").color(BAD));
                        if ui.add(small_accent_button("Yes, delete")).clicked() {
                            clear = true;
                        }
                        if ui.add(small_button("Cancel")).clicked() {
                            cancel_clear = true;
                        }
                    } else if ui.add(ghost_button("Clear all history")).clicked() {
                        ask_clear = true;
                    }
                });
            });
        });
        if ask_clear {
            self.confirm_clear = true;
        }
        if cancel_clear {
            self.confirm_clear = false;
        }
        if load_sample {
            self.data.history = sample_history(&self.data.settings, now_secs());
            self.log_selected = None;
            self.show_toast("Sample history loaded.".to_string());
            if self.active.is_none() {
                self.regenerate();
            }
        }
        if clear {
            self.data.history.clear();
            self.confirm_clear = false;
            self.log_selected = None;
            self.show_toast("All history cleared.".to_string());
            if self.active.is_none() {
                self.regenerate();
            }
        }
    }
}
