//! The exercise library and the muscle / equipment vocabulary.

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

pub const MUSCLE_COUNT: usize = 14;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Muscle {
    Chest,
    Shoulders,
    Triceps,
    Biceps,
    Forearms,
    Traps,
    Lats,
    UpperBack,
    LowerBack,
    Abs,
    Glutes,
    Quads,
    Hamstrings,
    Calves,
}

impl Muscle {
    pub const ALL: [Muscle; MUSCLE_COUNT] = [
        Muscle::Chest,
        Muscle::Shoulders,
        Muscle::Triceps,
        Muscle::Biceps,
        Muscle::Forearms,
        Muscle::Traps,
        Muscle::Lats,
        Muscle::UpperBack,
        Muscle::LowerBack,
        Muscle::Abs,
        Muscle::Glutes,
        Muscle::Quads,
        Muscle::Hamstrings,
        Muscle::Calves,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Muscle::Chest => "Chest",
            Muscle::Shoulders => "Shoulders",
            Muscle::Triceps => "Triceps",
            Muscle::Biceps => "Biceps",
            Muscle::Forearms => "Forearms",
            Muscle::Traps => "Traps",
            Muscle::Lats => "Lats",
            Muscle::UpperBack => "Upper Back",
            Muscle::LowerBack => "Lower Back",
            Muscle::Abs => "Abs",
            Muscle::Glutes => "Glutes",
            Muscle::Quads => "Quads",
            Muscle::Hamstrings => "Hamstrings",
            Muscle::Calves => "Calves",
        }
    }

    pub fn is_major(self) -> bool {
        matches!(
            self,
            Muscle::Chest
                | Muscle::Lats
                | Muscle::UpperBack
                | Muscle::Quads
                | Muscle::Hamstrings
                | Muscle::Glutes
                | Muscle::Shoulders
        )
    }

    /// Hours until a hard-trained muscle is fully fresh again.
    pub fn recovery_hours(self) -> f32 {
        if self.is_major() {
            72.0
        } else {
            52.0
        }
    }

    pub fn group(self) -> Group {
        match self {
            Muscle::Chest | Muscle::Shoulders | Muscle::Triceps => Group::Push,
            Muscle::Biceps | Muscle::Forearms | Muscle::Traps | Muscle::Lats | Muscle::UpperBack => {
                Group::Pull
            }
            Muscle::LowerBack | Muscle::Glutes | Muscle::Quads | Muscle::Hamstrings | Muscle::Calves => {
                Group::Legs
            }
            Muscle::Abs => Group::Core,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Push,
    Pull,
    Legs,
    Core,
}

impl Group {
    pub fn color(self) -> Color32 {
        match self {
            Group::Push => Color32::from_rgb(255, 112, 84),
            Group::Pull => Color32::from_rgb(82, 166, 255),
            Group::Legs => Color32::from_rgb(164, 120, 255),
            Group::Core => Color32::from_rgb(255, 196, 82),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Equip {
    Barbell,
    Dumbbell,
    Cable,
    Machine,
    Kettlebell,
    PullUpBar,
    Bodyweight,
}

impl Equip {
    pub const ALL: [Equip; 7] = [
        Equip::Barbell,
        Equip::Dumbbell,
        Equip::Cable,
        Equip::Machine,
        Equip::Kettlebell,
        Equip::PullUpBar,
        Equip::Bodyweight,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Equip::Barbell => "Barbell",
            Equip::Dumbbell => "Dumbbells",
            Equip::Cable => "Cable",
            Equip::Machine => "Machine",
            Equip::Kettlebell => "Kettlebell",
            Equip::PullUpBar => "Pull-up bar",
            Equip::Bodyweight => "Bodyweight",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Equip::Barbell => "BB",
            Equip::Dumbbell => "DB",
            Equip::Cable => "CB",
            Equip::Machine => "MC",
            Equip::Kettlebell => "KB",
            Equip::PullUpBar => "PU",
            Equip::Bodyweight => "BW",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Logged as weight x reps.
    Weighted,
    /// Logged as reps only.
    Bodyweight,
    /// Logged as seconds.
    Timed,
}

pub struct ExerciseDef {
    pub id: &'static str,
    pub name: &'static str,
    pub primary: Muscle,
    pub secondary: &'static [Muscle],
    pub equipment: Equip,
    pub kind: Kind,
    /// Starting point for an intermediate lifter: training max in lb
    /// (weighted), max reps (bodyweight) or max seconds (timed).
    pub base: f32,
    pub compound: bool,
    pub cue: &'static str,
}

pub fn find(id: &str) -> Option<&'static ExerciseDef> {
    EXERCISES.iter().find(|d| d.id == id)
}

#[allow(clippy::too_many_arguments)]
const fn ex(
    id: &'static str,
    name: &'static str,
    primary: Muscle,
    secondary: &'static [Muscle],
    equipment: Equip,
    kind: Kind,
    base: f32,
    compound: bool,
    cue: &'static str,
) -> ExerciseDef {
    ExerciseDef { id, name, primary, secondary, equipment, kind, base, compound, cue }
}

mod table {
    use super::Muscle::*;
    use super::{ex, Equip, ExerciseDef, Kind};

    const BB: Equip = Equip::Barbell;
    const DB: Equip = Equip::Dumbbell;
    const CB: Equip = Equip::Cable;
    const MC: Equip = Equip::Machine;
    const KB: Equip = Equip::Kettlebell;
    const PB: Equip = Equip::PullUpBar;
    const BWE: Equip = Equip::Bodyweight;
    const W: Kind = Kind::Weighted;
    const B: Kind = Kind::Bodyweight;
    const T: Kind = Kind::Timed;

    pub static EXERCISES: &[ExerciseDef] = &[
        // ---- Chest
        ex("bench_press", "Barbell Bench Press", Chest, &[Triceps, Shoulders], BB, W, 205.0, true,
           "Lower the bar to mid-chest with elbows about 45 degrees from your sides, then press up and slightly back."),
        ex("incline_bench", "Incline Barbell Press", Chest, &[Shoulders, Triceps], BB, W, 170.0, true,
           "Set the bench to about 30 degrees. Touch the upper chest and drive the bar straight up."),
        ex("db_bench", "Dumbbell Bench Press", Chest, &[Triceps, Shoulders], DB, W, 75.0, true,
           "Press the dumbbells up and slightly together; lower slowly until you feel a stretch."),
        ex("incline_db", "Incline Dumbbell Press", Chest, &[Shoulders, Triceps], DB, W, 65.0, true,
           "Keep your shoulder blades pinned to the bench and press in a slight arc."),
        ex("db_fly", "Dumbbell Fly", Chest, &[Shoulders], DB, W, 35.0, false,
           "Keep a soft bend in the elbows and open wide until the chest stretches, then hug the weights back up."),
        ex("cable_cross", "Cable Crossover", Chest, &[Shoulders], CB, W, 45.0, false,
           "Step forward from the stack and sweep the handles together in front of your lower chest."),
        ex("machine_press", "Machine Chest Press", Chest, &[Triceps, Shoulders], MC, W, 180.0, true,
           "Line the handles up with mid-chest and press without letting your shoulders roll forward."),
        ex("push_up", "Push-Up", Chest, &[Triceps, Shoulders, Abs], BWE, B, 25.0, true,
           "Keep a straight line from head to heels; lower until your chest nearly touches the floor."),
        ex("dip", "Chest Dip", Chest, &[Triceps, Shoulders], BWE, B, 14.0, true,
           "Lean your torso slightly forward and lower until the upper arms are parallel to the floor."),
        // ---- Shoulders
        ex("ohp", "Overhead Press", Shoulders, &[Triceps, Traps], BB, W, 125.0, true,
           "Squeeze your glutes, press the bar overhead and move your head through once it passes your forehead."),
        ex("db_shoulder_press", "Seated Dumbbell Press", Shoulders, &[Triceps], DB, W, 55.0, true,
           "Start with the dumbbells at ear height and press up until the arms are nearly straight."),
        ex("lateral_raise", "Dumbbell Lateral Raise", Shoulders, &[Traps], DB, W, 25.0, false,
           "Lead with the elbows and raise to shoulder height; control the way down."),
        ex("cable_lateral", "Cable Lateral Raise", Shoulders, &[], CB, W, 20.0, false,
           "Hold the handle across your body and raise it out to the side in a wide arc."),
        ex("arnold", "Arnold Press", Shoulders, &[Triceps], DB, W, 45.0, true,
           "Start palms facing you, rotate the palms forward as you press overhead."),
        ex("rear_fly", "Rear Delt Fly", Shoulders, &[UpperBack], DB, W, 25.0, false,
           "Hinge forward with a flat back and raise the weights out wide, squeezing the back of the shoulders."),
        ex("face_pull", "Face Pull", Shoulders, &[UpperBack, Traps], CB, W, 60.0, false,
           "Pull the rope toward your eyes, splitting the ends apart and finishing with the hands beside your ears."),
        ex("machine_shoulder", "Machine Shoulder Press", Shoulders, &[Triceps], MC, W, 130.0, true,
           "Keep your back against the pad and press until the arms are almost locked."),
        ex("kb_press", "Kettlebell Press", Shoulders, &[Triceps, Abs], KB, W, 40.0, true,
           "Rack the bell at the shoulder, brace your core and press straight up."),
        ex("pike_push_up", "Pike Push-Up", Shoulders, &[Triceps], BWE, B, 15.0, true,
           "Hips high, lower the top of your head toward the floor between your hands."),
        // ---- Triceps
        ex("close_grip", "Close-Grip Bench Press", Triceps, &[Chest, Shoulders], BB, W, 175.0, true,
           "Hands about shoulder-width apart; keep the elbows tucked as you lower to the lower chest."),
        ex("pushdown", "Cable Tricep Pushdown", Triceps, &[], CB, W, 75.0, false,
           "Pin the elbows to your sides and push the bar down until the arms are straight."),
        ex("oh_extension", "Overhead Dumbbell Extension", Triceps, &[], DB, W, 55.0, false,
           "Hold one dumbbell overhead with both hands and lower it behind your head."),
        ex("skull_crusher", "Skull Crusher", Triceps, &[], BB, W, 80.0, false,
           "Lower the bar toward your forehead by bending only at the elbows."),
        ex("bench_dip", "Bench Dip", Triceps, &[Chest, Shoulders], BWE, B, 20.0, false,
           "Hands on a bench behind you; lower until the elbows reach 90 degrees."),
        // ---- Biceps
        ex("bb_curl", "Barbell Curl", Biceps, &[Forearms], BB, W, 95.0, false,
           "Keep the elbows still at your sides and curl the bar to your shoulders."),
        ex("db_curl", "Dumbbell Curl", Biceps, &[Forearms], DB, W, 35.0, false,
           "Curl with palms up and squeeze at the top; lower slowly."),
        ex("hammer_curl", "Hammer Curl", Biceps, &[Forearms], DB, W, 40.0, false,
           "Palms face each other the whole time; curl without swinging."),
        ex("cable_curl", "Cable Curl", Biceps, &[Forearms], CB, W, 60.0, false,
           "Stand tall and curl the bar while keeping constant tension from the cable."),
        ex("preacher", "Machine Preacher Curl", Biceps, &[], MC, W, 75.0, false,
           "Upper arms flat on the pad; curl up and lower all the way down."),
        // ---- Lats
        ex("pull_up", "Pull-Up", Lats, &[Biceps, UpperBack], PB, B, 10.0, true,
           "Hands just wider than shoulders, palms away. Pull your chest toward the bar."),
        ex("chin_up", "Chin-Up", Lats, &[Biceps], PB, B, 11.0, true,
           "Palms facing you; pull until your chin clears the bar."),
        ex("lat_pulldown", "Lat Pulldown", Lats, &[Biceps, UpperBack], CB, W, 160.0, true,
           "Lean back slightly and pull the bar to your upper chest, driving the elbows down."),
        ex("straight_arm", "Straight-Arm Pulldown", Lats, &[Triceps], CB, W, 60.0, false,
           "Arms almost straight, sweep the bar down to your thighs."),
        ex("one_arm_row", "One-Arm Dumbbell Row", Lats, &[UpperBack, Biceps], DB, W, 85.0, true,
           "Support yourself on a bench and pull the dumbbell toward your hip."),
        // ---- Upper back
        ex("bb_row", "Barbell Row", UpperBack, &[Lats, Biceps, LowerBack], BB, W, 175.0, true,
           "Hinge to about 45 degrees with a flat back and row the bar to your belly button."),
        ex("cable_row", "Seated Cable Row", UpperBack, &[Lats, Biceps], CB, W, 150.0, true,
           "Sit tall and pull the handle to your stomach, squeezing the shoulder blades together."),
        ex("machine_row", "Chest-Supported Machine Row", UpperBack, &[Lats, Biceps], MC, W, 150.0, true,
           "Chest on the pad; pull the handles back and pause for a moment."),
        ex("inverted_row", "Inverted Row", UpperBack, &[Lats, Biceps], BWE, B, 15.0, true,
           "Hang under a bar or rings with a straight body and pull your chest up to it."),
        ex("kb_row", "Kettlebell Row", UpperBack, &[Lats, Biceps], KB, W, 60.0, true,
           "Hinge forward and row the bell up toward your ribs."),
        // ---- Traps
        ex("bb_shrug", "Barbell Shrug", Traps, &[Forearms], BB, W, 245.0, false,
           "Lift your shoulders straight up toward your ears and hold briefly."),
        ex("db_shrug", "Dumbbell Shrug", Traps, &[Forearms], DB, W, 80.0, false,
           "Arms long at your sides; shrug up and lower slowly."),
        // ---- Forearms
        ex("wrist_curl", "Dumbbell Wrist Curl", Forearms, &[], DB, W, 30.0, false,
           "Forearms on your thighs, palms up; curl the weight using only your wrists."),
        ex("reverse_curl", "Reverse Barbell Curl", Forearms, &[Biceps], BB, W, 65.0, false,
           "Overhand grip; curl the bar while keeping the wrists straight."),
        // ---- Lower back
        ex("deadlift", "Deadlift", LowerBack, &[Hamstrings, Glutes, UpperBack, Traps], BB, W, 315.0, true,
           "Bar over mid-foot, flat back, push the floor away and stand up tall."),
        ex("back_ext", "Back Extension", LowerBack, &[Glutes, Hamstrings], BWE, B, 20.0, false,
           "Hinge at the hips over the pad and rise until your body forms a straight line."),
        ex("good_morning", "Good Morning", LowerBack, &[Hamstrings, Glutes], BB, W, 135.0, true,
           "Bar on your upper back, soft knees; push the hips back until you feel the hamstrings."),
        // ---- Quads
        ex("squat", "Back Squat", Quads, &[Glutes, Hamstrings, LowerBack], BB, W, 255.0, true,
           "Brace, sit down between your heels until the hips pass the knees, then drive up."),
        ex("front_squat", "Front Squat", Quads, &[Glutes, Abs], BB, W, 195.0, true,
           "Bar on the front of the shoulders, elbows high, torso upright."),
        ex("leg_press", "Leg Press", Quads, &[Glutes, Hamstrings], MC, W, 420.0, true,
           "Feet shoulder-width on the platform; lower until the knees reach about 90 degrees."),
        ex("goblet_squat", "Goblet Squat", Quads, &[Glutes], DB, W, 75.0, true,
           "Hold a dumbbell at your chest and squat between your knees."),
        ex("split_squat", "Bulgarian Split Squat", Quads, &[Glutes, Hamstrings], DB, W, 45.0, true,
           "Rear foot on a bench; drop the back knee straight down."),
        ex("leg_ext", "Leg Extension", Quads, &[], MC, W, 130.0, false,
           "Straighten your legs fully and pause before lowering."),
        ex("lunge", "Walking Lunge", Quads, &[Glutes, Hamstrings], DB, W, 45.0, true,
           "Take long steps and lower the back knee close to the floor each rep."),
        ex("kb_goblet", "Kettlebell Goblet Squat", Quads, &[Glutes], KB, W, 60.0, true,
           "Hold the bell by the horns at chest height and squat deep."),
        ex("air_squat", "Bodyweight Squat", Quads, &[Glutes], BWE, B, 30.0, true,
           "Arms forward for balance; squat to parallel and stand tall."),
        // ---- Hamstrings
        ex("rdl", "Romanian Deadlift", Hamstrings, &[Glutes, LowerBack], BB, W, 215.0, true,
           "Slide the bar down your thighs by pushing the hips back; stop when the hamstrings are stretched."),
        ex("leg_curl", "Lying Leg Curl", Hamstrings, &[], MC, W, 100.0, false,
           "Curl your heels toward your glutes and lower under control."),
        ex("db_rdl", "Dumbbell Romanian Deadlift", Hamstrings, &[Glutes, LowerBack], DB, W, 70.0, true,
           "Keep the dumbbells close to your legs as you hinge."),
        ex("kb_swing", "Kettlebell Swing", Hamstrings, &[Glutes, LowerBack], KB, W, 60.0, true,
           "Hike the bell back, then snap the hips forward to float it to chest height."),
        // ---- Glutes
        ex("hip_thrust", "Barbell Hip Thrust", Glutes, &[Hamstrings], BB, W, 255.0, true,
           "Upper back on a bench, drive through your heels and lock the hips out at the top."),
        ex("glute_bridge", "Glute Bridge", Glutes, &[Hamstrings], BWE, B, 25.0, false,
           "Lying on your back, push through the heels and squeeze the glutes at the top."),
        ex("pull_through", "Cable Pull-Through", Glutes, &[Hamstrings], CB, W, 80.0, false,
           "Facing away from a low cable, hinge back and snap the hips forward."),
        ex("step_up", "Dumbbell Step-Up", Glutes, &[Quads], DB, W, 40.0, true,
           "Step onto a box and drive through the front heel to stand up fully."),
        // ---- Calves
        ex("calf_raise", "Standing Calf Raise", Calves, &[], MC, W, 200.0, false,
           "Rise as high as you can on your toes, pause, then lower into a deep stretch."),
        ex("seated_calf", "Seated Calf Raise", Calves, &[], MC, W, 110.0, false,
           "Knees under the pad; press up through the balls of your feet."),
        ex("db_calf", "Dumbbell Calf Raise", Calves, &[], DB, W, 50.0, false,
           "Hold dumbbells at your sides and raise your heels off a step."),
        ex("single_calf", "Single-Leg Calf Raise", Calves, &[], BWE, B, 20.0, false,
           "One foot on a step edge; rise up slowly and lower all the way down."),
        // ---- Abs
        ex("plank", "Plank", Abs, &[Shoulders], BWE, T, 70.0, false,
           "Forearms on the floor, body in a straight line, squeeze glutes and abs."),
        ex("hanging_leg_raise", "Hanging Leg Raise", Abs, &[Forearms], PB, B, 14.0, false,
           "Hang from a bar and lift your legs without swinging."),
        ex("cable_crunch", "Cable Crunch", Abs, &[], CB, W, 90.0, false,
           "Kneel facing the stack and crunch your ribs toward your hips."),
        ex("crunch", "Crunch", Abs, &[], BWE, B, 30.0, false,
           "Curl your shoulders off the floor and squeeze the abs; don't pull on your neck."),
        ex("ab_wheel", "Ab Wheel Rollout", Abs, &[Shoulders, LowerBack], BWE, B, 12.0, false,
           "From your knees, roll out as far as you can while keeping your back flat."),
        ex("russian_twist", "Russian Twist", Abs, &[], DB, W, 25.0, false,
           "Lean back slightly and rotate the weight from hip to hip."),
    ];
}

pub use table::EXERCISES;
