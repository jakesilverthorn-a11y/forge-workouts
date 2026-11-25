//! Training logic: settings, logs, muscle recovery and the workout generator.

use serde::{Deserialize, Serialize};

use crate::data::{self, Equip, ExerciseDef, Kind, Muscle, EXERCISES, MUSCLE_COUNT};
use crate::util::{local_midnight, Rng, DAY};

// ---------------------------------------------------------------- settings

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Goal {
    Strength,
    Hypertrophy,
    Endurance,
}

impl Goal {
    pub const ALL: [Goal; 3] = [Goal::Strength, Goal::Hypertrophy, Goal::Endurance];
    pub fn name(self) -> &'static str {
        match self {
            Self::Strength => "Get stronger",
            Self::Hypertrophy => "Build muscle",
            Self::Endurance => "Endurance",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Self::Strength => "Heavier weights, 5-8 reps, longer rests.",
            Self::Hypertrophy => "Moderate weights, 8-12 reps.",
            Self::Endurance => "Lighter weights, 15-20 reps, short rests.",
        }
    }
    pub fn rest_secs(self) -> f64 {
        match self {
            Self::Strength => 150.0,
            Self::Hypertrophy => 90.0,
            Self::Endurance => 60.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Experience {
    Beginner,
    Intermediate,
    Advanced,
}

impl Experience {
    pub const ALL: [Experience; 3] =
        [Experience::Beginner, Experience::Intermediate, Experience::Advanced];
    pub fn name(self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
        }
    }
    pub fn factor(self) -> f32 {
        match self {
            Self::Beginner => 0.6,
            Self::Intermediate => 1.0,
            Self::Advanced => 1.35,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Split {
    FullBody,
    UpperLower,
    PushPullLegs,
}

impl Split {
    pub const ALL: [Split; 3] = [Split::FullBody, Split::UpperLower, Split::PushPullLegs];
    pub fn name(self) -> &'static str {
        match self {
            Self::FullBody => "Full body",
            Self::UpperLower => "Upper / Lower",
            Self::PushPullLegs => "Push / Pull / Legs",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Self::FullBody => "Every workout hits your freshest muscles across the whole body.",
            Self::UpperLower => "Alternates upper-body and lower-body days.",
            Self::PushPullLegs => "Rotates pushing, pulling and leg days.",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Units {
    Lb,
    Kg,
}

impl Units {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lb => "lb",
            Self::Kg => "kg",
        }
    }
    pub fn from_lb(self, lb: f32) -> f32 {
        match self {
            Self::Lb => lb,
            Self::Kg => lb * 0.453_592_37,
        }
    }
    pub fn to_lb(self, v: f32) -> f32 {
        match self {
            Self::Lb => v,
            Self::Kg => v / 0.453_592_37,
        }
    }
    pub fn step(self) -> f32 {
        match self {
            Self::Lb => 5.0,
            Self::Kg => 2.5,
        }
    }
    pub fn fmt(self, lb: f32) -> String {
        let v = self.from_lb(lb);
        if (v - v.round()).abs() < 0.05 {
            format!("{:.0} {}", v, self.label())
        } else {
            format!("{:.1} {}", v, self.label())
        }
    }
    pub fn fmt_big(self, lb: f32) -> String {
        format!("{} {}", crate::util::fmt_thousands(self.from_lb(lb) as f64), self.label())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    Auto,
    Push,
    Pull,
    Legs,
    Upper,
    Lower,
    FullBody,
    Core,
}

impl Focus {
    pub const ALL: [Focus; 8] = [
        Focus::Auto,
        Focus::Push,
        Focus::Pull,
        Focus::Legs,
        Focus::Upper,
        Focus::Lower,
        Focus::FullBody,
        Focus::Core,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Focus::Auto => "Smart pick",
            Focus::Push => "Push",
            Focus::Pull => "Pull",
            Focus::Legs => "Legs",
            Focus::Upper => "Upper body",
            Focus::Lower => "Lower body",
            Focus::FullBody => "Full body",
            Focus::Core => "Core",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Focus::Auto => "Today's Workout",
            Focus::Push => "Push Day",
            Focus::Pull => "Pull Day",
            Focus::Legs => "Leg Day",
            Focus::Upper => "Upper Body",
            Focus::Lower => "Lower Body",
            Focus::FullBody => "Full Body",
            Focus::Core => "Core & Abs",
        }
    }
    pub fn muscles(self) -> Vec<Muscle> {
        use Muscle::*;
        match self {
            Focus::Push => vec![Chest, Shoulders, Triceps],
            Focus::Pull => vec![Lats, UpperBack, Biceps, Traps, Forearms],
            Focus::Legs => vec![Quads, Hamstrings, Glutes, Calves, LowerBack],
            Focus::Upper => vec![Chest, Lats, UpperBack, Shoulders, Biceps, Triceps, Traps],
            Focus::Lower => vec![Quads, Hamstrings, Glutes, Calves, LowerBack, Abs],
            Focus::Core => vec![Abs, LowerBack],
            Focus::FullBody | Focus::Auto => Muscle::ALL.to_vec(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub goal: Goal,
    pub experience: Experience,
    pub split: Split,
    pub duration_min: u32,
    pub units: Units,
    pub equipment: Vec<Equip>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            goal: Goal::Hypertrophy,
            experience: Experience::Intermediate,
            split: Split::PushPullLegs,
            duration_min: 45,
            units: Units::Lb,
            equipment: Equip::ALL.to_vec(),
        }
    }
}

impl Settings {
    pub fn has(&self, e: Equip) -> bool {
        e == Equip::Bodyweight || self.equipment.contains(&e)
    }
}

// ---------------------------------------------------------------- logs

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SetLog {
    /// Always stored in pounds; converted for display.
    pub weight: f32,
    pub reps: u32,
    pub done: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExerciseLog {
    pub id: String,
    pub sets: Vec<SetLog>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkoutLog {
    pub start: i64,
    pub duration_secs: i64,
    pub title: String,
    pub exercises: Vec<ExerciseLog>,
}

impl WorkoutLog {
    pub fn volume(&self) -> f32 {
        let mut v = 0.0;
        for e in &self.exercises {
            if let Some(d) = data::find(&e.id) {
                if d.kind == Kind::Weighted {
                    for s in &e.sets {
                        if s.done {
                            v += s.weight * s.reps as f32;
                        }
                    }
                }
            }
        }
        v
    }
    pub fn set_count(&self) -> usize {
        self.exercises.iter().map(|e| e.sets.iter().filter(|s| s.done).count()).sum()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppData {
    pub settings: Settings,
    pub history: Vec<WorkoutLog>,
}

pub struct Plan {
    pub title: String,
    pub focus: Focus,
    pub exercises: Vec<ExerciseLog>,
}

impl Plan {
    pub fn total_sets(&self) -> usize {
        self.exercises.iter().map(|e| e.sets.len()).sum()
    }
    pub fn est_minutes(&self) -> u32 {
        let sets = self.total_sets() as f32;
        (sets * 2.3 + self.exercises.len() as f32 * 2.0).round() as u32
    }
}

// ---------------------------------------------------------------- recovery

/// Percent recovered (0..100) for every muscle.
pub fn recovery(history: &[WorkoutLog], now: i64) -> [f32; MUSCLE_COUNT] {
    let mut fatigue = [0.0f32; MUSCLE_COUNT];
    for w in history {
        let hrs = (now - w.start) as f32 / 3600.0;
        if hrs < 0.0 {
            continue;
        }
        for e in &w.exercises {
            let Some(def) = data::find(&e.id) else { continue };
            let done = e.sets.iter().filter(|s| s.done).count() as f32;
            if done == 0.0 {
                continue;
            }
            let mut add = |m: Muscle, per_set: f32| {
                let decay = (1.0 - hrs / m.recovery_hours()).max(0.0);
                fatigue[m.index()] += per_set * done * decay;
            };
            add(def.primary, 13.0);
            for &m in def.secondary {
                add(m, 6.0);
            }
        }
    }
    let mut out = [100.0f32; MUSCLE_COUNT];
    for i in 0..MUSCLE_COUNT {
        out[i] = (100.0 - fatigue[i]).clamp(0.0, 100.0);
    }
    out
}

pub fn last_trained(history: &[WorkoutLog], m: Muscle) -> Option<i64> {
    let mut best: Option<i64> = None;
    for w in history {
        for e in &w.exercises {
            if !e.sets.iter().any(|s| s.done) {
                continue;
            }
            if let Some(d) = data::find(&e.id) {
                if d.primary == m || d.secondary.contains(&m) {
                    best = Some(best.map_or(w.start, |b| b.max(w.start)));
                }
            }
        }
    }
    best
}

pub fn avg_recovery(rec: &[f32; MUSCLE_COUNT], muscles: &[Muscle]) -> f32 {
    if muscles.is_empty() {
        return 100.0;
    }
    muscles.iter().map(|m| rec[m.index()]).sum::<f32>() / muscles.len() as f32
}

fn choose_focus(s: &Settings, rec: &[f32; MUSCLE_COUNT]) -> Focus {
    let options: Vec<Focus> = match s.split {
        Split::FullBody => vec![Focus::FullBody],
        Split::UpperLower => vec![Focus::Upper, Focus::Lower],
        Split::PushPullLegs => vec![Focus::Push, Focus::Pull, Focus::Legs],
    };
    let mut best = options[0];
    let mut best_score = -1.0f32;
    for f in options {
        let score = avg_recovery(rec, &f.muscles());
        if score > best_score + 0.01 {
            best = f;
            best_score = score;
        }
    }
    best
}

// ---------------------------------------------------------------- strength numbers

pub fn epley(weight: f32, reps: u32) -> f32 {
    if reps <= 1 {
        weight
    } else {
        weight * (1.0 + reps as f32 / 30.0)
    }
}

/// Every session where this exercise had at least one completed set, oldest first.
pub fn sessions<'a>(history: &'a [WorkoutLog], id: &str) -> Vec<(i64, &'a ExerciseLog)> {
    let mut v = Vec::new();
    for w in history {
        for e in &w.exercises {
            if e.id == id && e.sets.iter().any(|s| s.done) {
                v.push((w.start, e));
            }
        }
    }
    v.sort_by_key(|x| x.0);
    v
}

/// Estimated one-rep max (weighted) or best reps / seconds (others) for one session.
pub fn session_metric(def: &ExerciseDef, e: &ExerciseLog) -> f32 {
    let mut best = 0.0f32;
    for s in &e.sets {
        if !s.done {
            continue;
        }
        let v = match def.kind {
            Kind::Weighted => epley(s.weight, s.reps),
            _ => s.reps as f32,
        };
        best = best.max(v);
    }
    best
}

pub fn metric_history(history: &[WorkoutLog], def: &ExerciseDef) -> Vec<(i64, f32)> {
    sessions(history, def.id)
        .into_iter()
        .map(|(t, e)| (t, session_metric(def, e)))
        .collect()
}

/// Best set ever (by estimated 1RM, or reps for bodyweight moves).
pub fn best_set(history: &[WorkoutLog], def: &ExerciseDef) -> Option<(i64, SetLog)> {
    let mut best: Option<(i64, SetLog, f32)> = None;
    for (t, e) in sessions(history, def.id) {
        for s in &e.sets {
            if !s.done {
                continue;
            }
            let v = match def.kind {
                Kind::Weighted => epley(s.weight, s.reps),
                _ => s.reps as f32,
            };
            if best.as_ref().map_or(true, |b| v > b.2) {
                best = Some((t, *s, v));
            }
        }
    }
    best.map(|b| (b.0, b.1))
}

fn training_max(def: &ExerciseDef, s: &Settings, history: &[WorkoutLog]) -> f32 {
    let ss = sessions(history, def.id);
    if ss.is_empty() {
        return def.base * s.experience.factor();
    }
    let recent = &ss[ss.len().saturating_sub(3)..];
    let mut best = 0.0f32;
    for (_, e) in recent.iter() {
        for x in &e.sets {
            if !x.done {
                continue;
            }
            let v = match def.kind {
                // Assume about two reps were left in the tank.
                Kind::Weighted => x.weight * (1.0 + (x.reps as f32 + 2.0) / 30.0),
                _ => x.reps as f32,
            };
            best = best.max(v);
        }
    }
    match def.kind {
        // Progressive overload: nudge the target up a little every session.
        Kind::Weighted => best * 1.025,
        _ => best,
    }
}

pub fn round_weight(lb: f32, u: Units) -> f32 {
    let step = u.step();
    let v = (u.from_lb(lb) / step).round() * step;
    u.to_lb(v.max(step))
}

fn scheme(goal: Goal, compound: bool) -> (usize, u32) {
    match (goal, compound) {
        (Goal::Strength, true) => (5, 5),
        (Goal::Strength, false) => (3, 8),
        (Goal::Hypertrophy, true) => (4, 8),
        (Goal::Hypertrophy, false) => (3, 12),
        (Goal::Endurance, true) => (3, 15),
        (Goal::Endurance, false) => (3, 20),
    }
}

/// Sets, reps and weight for one exercise, based on the person's history.
pub fn prescribe(def: &ExerciseDef, s: &Settings, history: &[WorkoutLog]) -> ExerciseLog {
    let (sets, reps) = scheme(s.goal, def.compound);
    let tm = training_max(def, s, history);
    let mut out = Vec::new();
    match def.kind {
        Kind::Weighted => {
            let w = round_weight(tm / (1.0 + (reps as f32 + 2.0) / 30.0), s.units);
            for _ in 0..sets {
                out.push(SetLog { weight: w, reps, done: false });
            }
        }
        Kind::Bodyweight | Kind::Timed => {
            let has_history = !sessions(history, def.id).is_empty();
            let frac = match s.goal {
                Goal::Strength => 0.7,
                Goal::Hypertrophy => 0.8,
                Goal::Endurance => 0.9,
            };
            let target = if has_history {
                tm + if def.kind == Kind::Timed { 5.0 } else { 1.0 }
            } else {
                tm * frac
            };
            let mut r = target.round().max(3.0) as u32;
            if def.kind == Kind::Timed {
                r = ((r + 2) / 5 * 5).max(15);
            }
            for _ in 0..sets.min(4) {
                out.push(SetLog { weight: 0.0, reps: r, done: false });
            }
        }
    }
    ExerciseLog { id: def.id.to_string(), sets: out }
}

/// Other exercises for the same muscle that fit the person's equipment.
pub fn alternatives(cur: &ExerciseDef, s: &Settings, plan: &Plan) -> Vec<&'static ExerciseDef> {
    EXERCISES
        .iter()
        .filter(|d| {
            d.primary == cur.primary
                && d.id != cur.id
                && s.has(d.equipment)
                && !plan.exercises.iter().any(|e| e.id == d.id)
        })
        .collect()
}

fn exercise_count(minutes: u32) -> usize {
    match minutes {
        0..=25 => 3,
        26..=35 => 4,
        36..=50 => 5,
        51..=65 => 6,
        66..=80 => 7,
        _ => 8,
    }
}

/// Build a workout: pick the focus from muscle recovery, then pick exercises,
/// favouring fresh muscles, compound lifts first, familiar moves and some variety.
pub fn generate(s: &Settings, history: &[WorkoutLog], now: i64, focus: Focus, seed: u64) -> Plan {
    let rec = recovery(history, now);
    let focus = if focus == Focus::Auto { choose_focus(s, &rec) } else { focus };
    let mut rng = Rng::new(seed);

    let mut targets = focus.muscles();
    targets.sort_by(|a, b| {
        let ka = rec[a.index()] + if a.is_major() { 40.0 } else { 0.0 };
        let kb = rec[b.index()] + if b.is_major() { 40.0 } else { 0.0 };
        kb.partial_cmp(&ka).unwrap_or(std::cmp::Ordering::Equal)
    });
    let fresh: Vec<Muscle> = targets.iter().copied().filter(|m| rec[m.index()] >= 45.0).collect();
    if fresh.len() >= 2 {
        targets = fresh;
    }

    let n = exercise_count(s.duration_min);
    let last_ids: Vec<String> = history
        .last()
        .map(|w| w.exercises.iter().map(|e| e.id.clone()).collect())
        .unwrap_or_default();

    let mut chosen: Vec<&'static ExerciseDef> = Vec::new();
    if !targets.is_empty() {
        let mut k = 0usize;
        let mut misses = 0;
        while chosen.len() < n && misses < 40 {
            let m = targets[k % targets.len()];
            k += 1;
            let first_for_muscle = !chosen.iter().any(|d| d.primary == m);
            let mut best: Option<(&'static ExerciseDef, f32)> = None;
            for d in EXERCISES.iter() {
                if d.primary != m || !s.has(d.equipment) || chosen.iter().any(|c| c.id == d.id) {
                    continue;
                }
                let familiarity = sessions(history, d.id).len().min(5) as f32 * 0.35;
                let mut score = familiarity + rng.f32() * 1.6;
                if first_for_muscle && d.compound {
                    score += 2.0;
                }
                if !first_for_muscle && !d.compound {
                    score += 0.6;
                }
                if last_ids.iter().any(|x| x.as_str() == d.id) {
                    score -= 1.0;
                }
                if best.map_or(true, |b| score > b.1) {
                    best = Some((d, score));
                }
            }
            match best {
                Some((d, _)) => chosen.push(d),
                None => misses += 1,
            }
        }
    }
    chosen.sort_by_key(|d| if d.compound { 0 } else { 1 });

    Plan {
        title: focus.title().to_string(),
        focus,
        exercises: chosen.iter().map(|d| prescribe(d, s, history)).collect(),
    }
}

/// A few weeks of believable past workouts so the app isn't empty on first launch.
pub fn sample_history(s: &Settings, now: i64) -> Vec<WorkoutLog> {
    let mut hist: Vec<WorkoutLog> = Vec::new();
    let mut rng = Rng::new(42);
    let today = local_midnight(now);
    let mut d: i64 = 36;
    while d >= 1 {
        let start = today - d * DAY + 17 * 3600 + rng.range(90) as i64 * 60;
        let plan = generate(s, &hist, start, Focus::Auto, rng.next_u64());
        let mut exercises = plan.exercises;
        for e in &mut exercises {
            for set in &mut e.sets {
                set.done = true;
                if rng.f32() < 0.12 && set.reps > 4 {
                    set.reps -= 1 + rng.range(2) as u32;
                }
            }
        }
        let duration = 38 * 60 + rng.range(25 * 60) as i64;
        hist.push(WorkoutLog { start, duration_secs: duration, title: plan.title, exercises });
        d -= 2 + if rng.f32() < 0.3 { 1 } else { 0 };
    }
    hist
}
