use crate::std::extensions::str_to_unumber::StrToNumberExt;

#[derive(Copy, Clone, Default)]
/// Decimal calendar/time payload.
pub struct DateTime {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub day: u8,         // day of month: 1..31
    pub day_in_week: u8, // weekday: 1..7
    pub month: u8,
    pub year: u8,        // 00..99
}


pub fn build_datetime() -> DateTime {
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