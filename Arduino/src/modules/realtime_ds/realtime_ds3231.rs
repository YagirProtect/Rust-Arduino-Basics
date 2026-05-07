use crate::modules::realtime_ds::realtime_ds_registers::common::{
    DAY_MASK,
    DAY_WEEK_MASK,
    HOURS_MASK,
    MINUTES_MASK,
    MONTH_MASK,
    SECONDS_MASK,
    YEAR_MASK,
};
use crate::modules::realtime_ds::realtime_ds_registers::ds3231::{
    REG_SECONDS,
    REG_STATUS,
};
use crate::modules::realtime_ds::date_time::DateTime;
use crate::modules::realtime_ds::realtime_ds3231_alarm::AlarmModule;
use crate::std::extensions::binary_to_decimal::BcdExt;
use embedded_hal::i2c::I2c;

pub struct RealTimeDS3231 {
    addr: u8,
    alarms: Option<AlarmModule>,
}

impl RealTimeDS3231 {
    pub fn new(addr: u8, alarms: Option<AlarmModule>) -> Self {
        let mut rtc = Self { addr, alarms };

        if let Some(alarms) = rtc.alarms.as_mut() {
            alarms.bind_rtc_addr(addr);
        }

        rtc
    }

    pub fn alarms_mut(&mut self) -> Option<&mut AlarmModule> {
        self.alarms.as_mut()
    }

    pub fn set_time(&mut self, i2c: &mut impl I2c, dt: DateTime) -> bool {
        let data = [
            REG_SECONDS,
            dt.sec.dec_to_bdc() & SECONDS_MASK,
            dt.min.dec_to_bdc() & MINUTES_MASK,
            dt.hour.dec_to_bdc() & HOURS_MASK,
            dt.day_in_week.dec_to_bdc() & DAY_WEEK_MASK,
            dt.day.dec_to_bdc() & DAY_MASK,
            dt.month.dec_to_bdc() & MONTH_MASK,
            dt.year.dec_to_bdc() & YEAR_MASK,
        ];

        if i2c.write(self.addr, &data).is_err() {
            return false;
        }

        if let Some(alarms) = self.alarms.as_mut() {
            let _ = alarms.init(i2c);
        }

        true
    }

    pub fn read_time(&mut self, i2c: &mut impl I2c) -> Option<DateTime> {
        let mut b = [0u8; 7];

        if i2c.write_read(self.addr, &[REG_SECONDS], &mut b).is_err() {
            return None;
        }

        Some(DateTime {
            sec: (b[0] & SECONDS_MASK).bdc_to_dec(),
            min: (b[1] & MINUTES_MASK).bdc_to_dec(),
            hour: (b[2] & HOURS_MASK).bdc_to_dec(),
            day_in_week: (b[3] & DAY_WEEK_MASK).bdc_to_dec(),
            day: (b[4] & DAY_MASK).bdc_to_dec(),
            month: (b[5] & MONTH_MASK).bdc_to_dec(),
            year: (b[6] & YEAR_MASK).bdc_to_dec(),
        })
    }

    pub fn is_alive(&self, i2c: &mut impl I2c) -> bool {
        let mut b = [0u8; 1];
        i2c.write_read(self.addr, &[REG_STATUS], &mut b).is_ok()
    }
}
