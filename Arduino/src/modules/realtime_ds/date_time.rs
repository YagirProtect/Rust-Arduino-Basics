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