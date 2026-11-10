//! A validated civil calendar date — the pacing clock's
//! value type.
//!
//! The protocol core never reads a wall clock (design doc
//! section 8: pure functions over explicit inputs), so the
//! shell hands dates in as values. A `CalendarDate` can
//! only be constructed through [`CalendarDate::new`], which
//! validates the day against the real proleptic Gregorian
//! calendar — February 30th is a typed refusal, never a
//! silent rollover. Ordering is lexicographic on
//! (year, month, day), which is chronological order.
//!
//! Representation choice: a civil (year, month, day)
//! triple rather than a days-since-epoch count, because
//! the save file is human-inspectable and the pacing rule
//! is stated in calendar terms ("the next day unlocks
//! when the local date passes"). Epoch arithmetic exists
//! only inside [`CalendarDate::next_day`].

use serde::{Deserialize, Deserializer, Serialize};

use crate::protocol::FallError;

/// Inclusive year bound: the proleptic Gregorian calendar
/// from year 1. Saves have no business outside it.
pub const YEAR_MIN: i32 = 1;
/// Inclusive year bound (four-digit years).
pub const YEAR_MAX: i32 = 9999;

/// One civil calendar date, always a real date.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct CalendarDate {
    year: i32,
    month: u32,
    day: u32,
}

/// Raw mirror for deserialization: a date read from a save
/// crosses the same constructor as a date built in code,
/// so a stored February 30th is refused at load.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDate {
    year: i32,
    month: u32,
    day: u32,
}

impl<'de> Deserialize<'de> for CalendarDate {
    fn deserialize<D>(deserializer: D) -> Result<CalendarDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawDate::deserialize(deserializer)?;
        CalendarDate::new(raw.year, raw.month, raw.day)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

/// Gregorian leap rule (proleptic): divisible by 4,
/// except centuries, unless divisible by 400.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days in `month` of `year`; 0 for a month outside 1..=12.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Days from the civil epoch 1970-01-01 (Howard Hinnant's
/// algorithm); exact over the whole supported year range.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let mut y = i64::from(year);
    let m = i64::from(month);
    let d = i64::from(day);
    if m <= 2 {
        y -= 1;
    }
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Inverse of [`days_from_civil`].
fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    (
        i32::try_from(y).unwrap_or(YEAR_MAX),
        u32::try_from(m).unwrap_or(1),
        u32::try_from(d).unwrap_or(1),
    )
}

impl CalendarDate {
    /// Construct a date, validating it against the real
    /// calendar. Invalid input (month 13, February 30th, a
    /// year outside 1..=9999) is a typed refusal.
    pub fn new(year: i32, month: u32, day: u32) -> Result<CalendarDate, FallError> {
        let valid = (YEAR_MIN..=YEAR_MAX).contains(&year)
            && (1..=12).contains(&month)
            && day >= 1
            && day <= days_in_month(year, month);
        if !valid {
            return Err(FallError::InvalidDate { year, month, day });
        }
        Ok(CalendarDate { year, month, day })
    }

    /// The year (1..=9999).
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month (1..=12).
    pub fn month(self) -> u32 {
        self.month
    }

    /// The day of the month.
    pub fn day(self) -> u32 {
        self.day
    }

    /// The following calendar date, or `None` only at the
    /// top of the supported range (9999-12-31).
    pub fn next_day(self) -> Option<CalendarDate> {
        let (year, month, day) =
            civil_from_days(days_from_civil(self.year, self.month, self.day) + 1);
        CalendarDate::new(year, month, day).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_years_follow_the_gregorian_rule() {
        assert!(CalendarDate::new(2028, 2, 29).is_ok());
        assert!(CalendarDate::new(2027, 2, 29).is_err());
        assert!(CalendarDate::new(2000, 2, 29).is_ok());
        assert!(CalendarDate::new(1900, 2, 29).is_err());
    }

    #[test]
    fn next_day_rolls_months_and_years() {
        let jan31 = CalendarDate::new(2027, 1, 31).expect("valid");
        assert_eq!(jan31.next_day(), CalendarDate::new(2027, 2, 1).ok());
        let dec31 = CalendarDate::new(2027, 12, 31).expect("valid");
        assert_eq!(dec31.next_day(), CalendarDate::new(2028, 1, 1).ok());
        let leap = CalendarDate::new(2028, 2, 28).expect("valid");
        assert_eq!(leap.next_day(), CalendarDate::new(2028, 2, 29).ok());
    }

    #[test]
    fn ordering_is_chronological() {
        let a = CalendarDate::new(2027, 3, 1).expect("valid");
        let b = CalendarDate::new(2027, 3, 2).expect("valid");
        let c = CalendarDate::new(2028, 1, 1).expect("valid");
        assert!(a < b && b < c);
    }
}
