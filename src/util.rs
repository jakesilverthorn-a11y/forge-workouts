use std::sync::OnceLock;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

pub const DAY: i64 = 86_400;

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];
const WEEKDAYS: [&str; 7] = [
    "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
];

pub fn now_secs() -> i64 {
    now_f() as i64
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_f() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// In the browser `SystemTime::now()` panics, so ask JavaScript's clock.
#[cfg(target_arch = "wasm32")]
pub fn now_f() -> f64 {
    js_sys::Date::now() / 1000.0
}

/// Seconds east of UTC for the local time zone (computed once).
pub fn tz_offset() -> i64 {
    static OFF: OnceLock<i64> = OnceLock::new();
    *OFF.get_or_init(|| chrono::Local::now().offset().local_minus_utc() as i64)
}

/// Local calendar day number (days since 1970-01-01).
pub fn day_index(ts: i64) -> i64 {
    (ts + tz_offset()).div_euclid(DAY)
}

/// A timestamp at local midday of the given day number.
pub fn day_to_ts(day: i64) -> i64 {
    day * DAY - tz_offset() + DAY / 2
}

/// Start of the local day containing `ts`.
pub fn local_midnight(ts: i64) -> i64 {
    day_index(ts) * DAY - tz_offset()
}

/// 0 = Monday ... 6 = Sunday
pub fn weekday_mon(day: i64) -> usize {
    (day + 3).rem_euclid(7) as usize
}

/// Day number of the Monday that starts the week containing `ts`.
pub fn week_start_day(ts: i64) -> i64 {
    let d = day_index(ts);
    d - weekday_mon(d) as i64
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn fmt_long_date(ts: i64) -> String {
    let day = day_index(ts);
    let (_, m, d) = civil_from_days(day);
    format!("{}, {} {}", WEEKDAYS[weekday_mon(day)], MONTHS[(m - 1) as usize], d)
}

pub fn fmt_short_date(ts: i64) -> String {
    let (_, m, d) = civil_from_days(day_index(ts));
    format!("{} {}", &MONTHS[(m - 1) as usize][..3], d)
}

pub fn fmt_day_label(ts: i64) -> String {
    let day = day_index(ts);
    let (_, m, d) = civil_from_days(day);
    format!(
        "{}, {} {}",
        &WEEKDAYS[weekday_mon(day)][..3],
        &MONTHS[(m - 1) as usize][..3],
        d
    )
}

pub fn fmt_clock(secs: i64) -> String {
    let secs = secs.max(0);
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

pub fn fmt_minutes(secs: i64) -> String {
    let m = (secs.max(0) + 30) / 60;
    if m >= 60 {
        format!("{} h {} min", m / 60, m % 60)
    } else {
        format!("{} min", m)
    }
}

pub fn fmt_ago(ts: i64, now: i64) -> String {
    let days = day_index(now) - day_index(ts);
    match days {
        d if d <= 0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        d if d < 14 => format!("{} days ago", d),
        d => format!("{} weeks ago", d / 7),
    }
}

pub fn fmt_thousands(v: f64) -> String {
    let n = v.round() as i64;
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 {
        format!("-{}", out)
    } else {
        out
    }
}

/// Small deterministic random generator (xorshift64*).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng((seed ^ 0x9E37_79B9_7F4A_7C15) | 1)
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// 0.0 .. 1.0
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next_u64() % n
        }
    }
}
