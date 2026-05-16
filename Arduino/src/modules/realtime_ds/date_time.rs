use crate::std::extensions::str_to_unumber::StrToNumberExt;
use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Sub, SubAssign};

const SECONDS_PER_MINUTE: u32 = 60;
const SECONDS_PER_HOUR: u32 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: u32 = 24 * SECONDS_PER_HOUR;

#[derive(Copy, Clone, Debug)]
/// Decimal calendar/time payload.
pub struct DateTime {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub day: u8,         // day of month: 1..31
    pub day_in_week: u8, // weekday: 1..7 (ISO: 1=Mon .. 7=Sun)
    pub month: u8,
    pub year: u8,        // 00..99 (2000..2099)
}

/// Signed duration for DateTime arithmetic (similar to .NET TimeSpan).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct DateTimeSpan {
    total_seconds: i64,
}

impl Default for DateTime {
    fn default() -> Self {
        Self::from_total_seconds_since_2000(0)
    }
}

impl DateTime {
    pub const fn new(sec: u8, min: u8, hour: u8, day: u8, day_in_week: u8, month: u8, year: u8) -> Self {
        Self {
            sec,
            min,
            hour,
            day,
            day_in_week,
            month,
            year,
        }
    }

    pub const fn is_leap_year(year_00_99: u8) -> bool {
        let y = 2000u16 + year_00_99 as u16;
        (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
    }

    pub const fn days_in_month(month: u8, year_00_99: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year_00_99) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.sec > 59 || self.min > 59 || self.hour > 23 {
            return false;
        }

        if self.year > 99 || self.month < 1 || self.month > 12 {
            return false;
        }

        let dim = Self::days_in_month(self.month, self.year);
        if self.day < 1 || self.day > dim {
            return false;
        }

        if self.day_in_week < 1 || self.day_in_week > 7 {
            return false;
        }

        true
    }

    /// Returns a clamped/normalized date-time in DS3231 range (2000..2099).
    pub fn normalized(&self) -> Self {
        let mut dt = self.normalized_raw();
        dt.day_in_week = Self::iso_weekday_from_total_days(dt.total_days_since_2000());
        dt
    }

    /// Total full days since 2000-01-01.
    pub fn total_days_since_2000(&self) -> u32 {
        let dt = self.normalized_raw();

        let mut days = 0u32;

        let mut y = 0u8;
        while y < dt.year {
            days += if Self::is_leap_year(y) { 366 } else { 365 };
            y += 1;
        }

        let mut m = 1u8;
        while m < dt.month {
            days += Self::days_in_month(m, dt.year) as u32;
            m += 1;
        }

        days + (dt.day as u32 - 1)
    }

    /// Total full hours since 2000-01-01 00:00:00.
    pub fn total_hours_since_2000(&self) -> u32 {
        self.total_days_since_2000() * 24 + self.normalized_raw().hour as u32
    }

    /// Total full minutes since 2000-01-01 00:00:00.
    pub fn total_minutes_since_2000(&self) -> u32 {
        self.total_hours_since_2000() * 60 + self.normalized_raw().min as u32
    }

    /// Total seconds since 2000-01-01 00:00:00.
    pub fn total_seconds_since_2000(&self) -> u32 {
        self.total_minutes_since_2000() * 60 + self.normalized_raw().sec as u32
    }

    pub fn from_total_seconds_since_2000(total_seconds: u32) -> Self {
        let total_days = total_seconds / SECONDS_PER_DAY;
        let mut days_left = total_days;
        let mut sec_of_day = total_seconds % SECONDS_PER_DAY;

        let hour = (sec_of_day / SECONDS_PER_HOUR) as u8;
        sec_of_day %= SECONDS_PER_HOUR;

        let min = (sec_of_day / SECONDS_PER_MINUTE) as u8;
        let sec = (sec_of_day % SECONDS_PER_MINUTE) as u8;

        let mut year = 0u8;
        while year < 99 {
            let dy = if Self::is_leap_year(year) { 366 } else { 365 };
            if days_left < dy {
                break;
            }
            days_left -= dy;
            year += 1;
        }

        let mut month = 1u8;
        while month < 12 {
            let dm = Self::days_in_month(month, year) as u32;
            if days_left < dm {
                break;
            }
            days_left -= dm;
            month += 1;
        }

        let day = (days_left + 1) as u8;

        Self {
            sec,
            min,
            hour,
            day,
            day_in_week: Self::iso_weekday_from_total_days(total_days),
            month,
            year,
        }
    }

    pub fn signed_duration_since(&self, other: DateTime) -> DateTimeSpan {
        let lhs = self.total_seconds_since_2000() as i64;
        let rhs = other.total_seconds_since_2000() as i64;
        DateTimeSpan::from_seconds(lhs - rhs)
    }

    pub fn duration_since(&self, other: DateTime) -> Option<DateTimeSpan> {
        if self >= &other {
            Some(self.signed_duration_since(other))
        } else {
            None
        }
    }

    pub fn checked_add_span(&self, span: DateTimeSpan) -> Option<Self> {
        let total = self.total_seconds_since_2000() as i64 + span.total_seconds();

        if total < 0 || total > Self::max_total_seconds_since_2000() as i64 {
            return None;
        }

        Some(Self::from_total_seconds_since_2000(total as u32))
    }

    pub fn add_span_saturating(&self, span: DateTimeSpan) -> Self {
        match self.checked_add_span(span) {
            Some(v) => v,
            None => {
                if span.total_seconds() < 0 {
                    Self::from_total_seconds_since_2000(0)
                } else {
                    Self::from_total_seconds_since_2000(Self::max_total_seconds_since_2000())
                }
            }
        }
    }

    pub fn add_seconds(&self, seconds: i64) -> Self {
        self.add_span_saturating(DateTimeSpan::from_seconds(seconds))
    }

    pub fn add_minutes(&self, minutes: i64) -> Self {
        self.add_span_saturating(DateTimeSpan::from_minutes(minutes))
    }

    pub fn add_hours(&self, hours: i64) -> Self {
        self.add_span_saturating(DateTimeSpan::from_hours(hours))
    }

    pub fn add_days(&self, days: i64) -> Self {
        self.add_span_saturating(DateTimeSpan::from_days(days))
    }

    fn normalized_raw(&self) -> Self {
        let year = self.year.min(99);
        let month = self.month.clamp(1, 12);
        let day_max = Self::days_in_month(month, year);

        Self {
            sec: self.sec.min(59),
            min: self.min.min(59),
            hour: self.hour.min(23),
            day: self.day.clamp(1, day_max),
            day_in_week: self.day_in_week.clamp(1, 7),
            month,
            year,
        }
    }

    fn iso_weekday_from_total_days(total_days: u32) -> u8 {
        // 2000-01-01 was Saturday (ISO = 6).
        (((total_days + 5) % 7) + 1) as u8
    }

    fn max_total_seconds_since_2000() -> u32 {
        DateTime {
            sec: 59,
            min: 59,
            hour: 23,
            day: 31,
            day_in_week: 1,
            month: 12,
            year: 99,
        }
        .total_seconds_since_2000()
    }


    pub fn now() -> DateTime {
        DateTime {
            sec: env!("BUILD_SEC").to_u8(),
            min: env!("BUILD_MIN").to_u8(),
            hour: env!("BUILD_HOUR").to_u8(),
            day: env!("BUILD_DAY").to_u8(),
            day_in_week: env!("BUILD_WEEKDAY").to_u8(),
            month: env!("BUILD_MONTH").to_u8(),
            year: (env!("BUILD_YEAR").to_u16() - 2000) as u8,
        }
    }
}

impl DateTimeSpan {
    pub const fn from_seconds(total_seconds: i64) -> Self {
        Self { total_seconds }
    }

    pub const fn from_minutes(total_minutes: i64) -> Self {
        Self {
            total_seconds: total_minutes * 60,
        }
    }

    pub const fn from_hours(total_hours: i64) -> Self {
        Self {
            total_seconds: total_hours * 60 * 60,
        }
    }

    pub const fn from_days(total_days: i64) -> Self {
        Self {
            total_seconds: total_days * 24 * 60 * 60,
        }
    }

    pub const fn total_seconds(self) -> i64 {
        self.total_seconds
    }

    pub const fn total_minutes(self) -> i64 {
        self.total_seconds / 60
    }

    pub const fn total_hours(self) -> i64 {
        self.total_seconds / (60 * 60)
    }

    pub const fn total_days(self) -> i64 {
        self.total_seconds / (24 * 60 * 60)
    }
}

impl PartialEq for DateTime {
    fn eq(&self, other: &Self) -> bool {
        self.total_seconds_since_2000() == other.total_seconds_since_2000()
    }
}

impl Eq for DateTime {}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.total_seconds_since_2000().cmp(&other.total_seconds_since_2000())
    }
}

impl Add<DateTimeSpan> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: DateTimeSpan) -> Self::Output {
        self.add_span_saturating(rhs)
    }
}

impl AddAssign<DateTimeSpan> for DateTime {
    fn add_assign(&mut self, rhs: DateTimeSpan) {
        *self = self.add_span_saturating(rhs);
    }
}

impl Sub<DateTimeSpan> for DateTime {
    type Output = DateTime;

    fn sub(self, rhs: DateTimeSpan) -> Self::Output {
        self.add_span_saturating(DateTimeSpan::from_seconds(-rhs.total_seconds()))
    }
}

impl SubAssign<DateTimeSpan> for DateTime {
    fn sub_assign(&mut self, rhs: DateTimeSpan) {
        *self = *self - rhs;
    }
}

impl Sub<DateTime> for DateTime {
    type Output = DateTimeSpan;

    fn sub(self, rhs: DateTime) -> Self::Output {
        self.signed_duration_since(rhs)
    }
}



#[cfg(test)]
mod tests {
    use super::{DateTime, DateTimeSpan};

    #[test]
    fn compare_and_sub_works() {
        let a = DateTime::new(0, 0, 12, 10, 3, 5, 26);
        let b = DateTime::new(0, 30, 11, 10, 3, 5, 26);

        assert!(a > b);
        assert_eq!((a - b).total_minutes(), 30);
    }

    #[test]
    fn add_minute_rollover_day_month_year() {
        let t = DateTime::new(30, 59, 23, 31, 7, 12, 25);
        let n = t + DateTimeSpan::from_minutes(1);

        assert_eq!(n.sec, 30);
        assert_eq!(n.min, 0);
        assert_eq!(n.hour, 0);
        assert_eq!(n.day, 1);
        assert_eq!(n.month, 1);
        assert_eq!(n.year, 26);
    }

    #[test]
    fn totals_roundtrip() {
        let src = DateTime::new(45, 13, 21, 17, 6, 8, 24);
        let total = src.total_seconds_since_2000();
        let dst = DateTime::from_total_seconds_since_2000(total);

        assert_eq!(src.sec, dst.sec);
        assert_eq!(src.min, dst.min);
        assert_eq!(src.hour, dst.hour);
        assert_eq!(src.day, dst.day);
        assert_eq!(src.month, dst.month);
        assert_eq!(src.year, dst.year);
    }
}
