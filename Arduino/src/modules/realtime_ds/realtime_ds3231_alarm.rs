use crate::modules::realtime_ds::realtime_ds_registers::common::{
    DAY_MASK,
    HOURS_MASK,
    MINUTES_MASK,
    SECONDS_MASK,
};
use crate::modules::realtime_ds::date_time::DateTime;
use crate::modules::realtime_ds::realtime_ds_registers::ds3231::{
    ALARM_MASK_BIT, 
    CONTROL_A1IE_BIT, 
    CONTROL_A2IE_BIT, 
    CONTROL_INTCN_BIT, 
    DY_DT_BIT, 
    REG_ALARM1_SECONDS, 
    REG_ALARM2_MINUTES, 
    REG_CONTROL, 
    REG_STATUS, 
    STATUS_A1F_BIT, 
    STATUS_A2F_BIT
};
use crate::std::extensions::binary_to_decimal::BcdExt;
use embedded_hal::i2c::I2c;


#[derive(Copy, Clone, Debug)]
pub struct AlarmModule {
    clear_alarm_flags_on_init: bool,
    disable_irq_on_init: bool,
    rtc_addr: u8,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Alarm1DateTime {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub day_or_date: u8, // 1..7 if `day_mode`, otherwise 1..31
    pub day_mode: bool,  // true = day of week, false = date of month
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Alarm2DateTime {
    pub min: u8,
    pub hour: u8,
    pub day_or_date: u8, // 1..7 if `day_mode`, otherwise 1..31
    pub day_mode: bool,  // true = day of week, false = date of month
}

impl AlarmModule {
    pub const fn new(clear_alarm_flags_on_init: bool, disable_irq_on_init: bool) -> Self {
        Self {
            clear_alarm_flags_on_init,
            disable_irq_on_init,
            rtc_addr: 0,
        }
    }

    pub const fn recommended() -> Self {
        Self::new(true, true)
    }

    pub(crate) fn bind_rtc_addr(&mut self, rtc_addr: u8) {
        self.rtc_addr = rtc_addr;
    }

    pub fn init(&mut self, i2c: &mut impl I2c) -> bool {
        let mut is_ok = true;

        if self.disable_irq_on_init {
            is_ok &= self.disable_all_interrupts(i2c);
        }

        if self.clear_alarm_flags_on_init {
            is_ok &= self.clear_flags(i2c);
        }

        is_ok
    }

    pub fn set_alarm1_exact<T>(&mut self, i2c: &mut impl I2c, alarm: T) -> bool
    where
        T: Into<Alarm1DateTime>,
    {
        let alarm = alarm.into();

        let day = (alarm.day_or_date.dec_to_bdc() & DAY_MASK)
            | if alarm.day_mode { DY_DT_BIT } else { 0 };

        let data = [
            REG_ALARM1_SECONDS,
            alarm.sec.dec_to_bdc() & SECONDS_MASK,
            alarm.min.dec_to_bdc() & MINUTES_MASK,
            alarm.hour.dec_to_bdc() & HOURS_MASK,
            day,
        ];

        i2c.write(self.rtc_addr, &data).is_ok()
    }

    pub fn set_alarm2_exact<T>(&mut self, i2c: &mut impl I2c, alarm: T) -> bool
    where
        T: Into<Alarm2DateTime>,
    {
        let alarm = alarm.into();

        let day = (alarm.day_or_date.dec_to_bdc() & DAY_MASK)
            | if alarm.day_mode { DY_DT_BIT } else { 0 };

        let data = [
            REG_ALARM2_MINUTES,
            alarm.min.dec_to_bdc() & MINUTES_MASK,
            alarm.hour.dec_to_bdc() & HOURS_MASK,
            day,
        ];

        i2c.write(self.rtc_addr, &data).is_ok()
    }

    pub(crate) fn set_alarm1_mask_bits(
        &mut self,
        i2c: &mut impl I2c,
        a1m1: bool,
        a1m2: bool,
        a1m3: bool,
        a1m4: bool,
    ) -> bool {
        self.set_alarm_mask_bits(i2c, REG_ALARM1_SECONDS, &[a1m1, a1m2, a1m3, a1m4])
    }

    pub(crate) fn set_alarm2_mask_bits(
        &mut self,
        i2c: &mut impl I2c,
        a2m2: bool,
        a2m3: bool,
        a2m4: bool,
    ) -> bool {
        self.set_alarm_mask_bits(i2c, REG_ALARM2_MINUTES, &[a2m2, a2m3, a2m4])
    }

    pub fn enable_alarm1_interrupt(&mut self, i2c: &mut impl I2c, enabled: bool) -> bool {
        self.update_control_bits(i2c, CONTROL_A1IE_BIT, enabled)
    }

    pub fn enable_alarm2_interrupt(&mut self, i2c: &mut impl I2c, enabled: bool) -> bool {
        self.update_control_bits(i2c, CONTROL_A2IE_BIT, enabled)
    }

    pub fn enable_interrupt_pin(&mut self, i2c: &mut impl I2c, enabled: bool) -> bool {
        self.update_control_bits(i2c, CONTROL_INTCN_BIT, enabled)
    }

    pub fn is_alarm1_triggered(&mut self, i2c: &mut impl I2c) -> Option<bool> {
        self.read_reg(i2c, REG_STATUS)
            .map(|v| (v & STATUS_A1F_BIT) != 0)
    }

    pub fn is_alarm2_triggered(&mut self, i2c: &mut impl I2c) -> Option<bool> {
        self.read_reg(i2c, REG_STATUS)
            .map(|v| (v & STATUS_A2F_BIT) != 0)
    }

    pub(crate) fn clear_alarm1_flag(&mut self, i2c: &mut impl I2c) -> bool {
        self.clear_status_bits(i2c, STATUS_A1F_BIT)
    }

    pub(crate) fn clear_alarm2_flag(&mut self, i2c: &mut impl I2c) -> bool {
        self.clear_status_bits(i2c, STATUS_A2F_BIT)
    }

    pub fn clear_flags(&mut self, i2c: &mut impl I2c) -> bool {
        self.clear_status_bits(i2c, STATUS_A1F_BIT | STATUS_A2F_BIT)
    }

    pub(crate) fn disable_all_interrupts(&mut self, i2c: &mut impl I2c) -> bool {
        self.update_control_bits(i2c, CONTROL_A1IE_BIT | CONTROL_A2IE_BIT, false)
    }

    fn set_alarm_mask_bits(
        &mut self,
        i2c: &mut impl I2c,
        first_reg: u8,
        mask_bits: &[bool],
    ) -> bool {
        for i in 0..mask_bits.len() {
            let reg = first_reg.wrapping_add(i as u8);
            let raw = match self.read_reg(i2c, reg) {
                Some(v) => v,
                None => return false,
            };

            let next = if mask_bits[i] {
                raw | ALARM_MASK_BIT
            } else {
                raw & !ALARM_MASK_BIT
            };

            if !self.write_reg(i2c, reg, next) {
                return false;
            }
        }

        true
    }

    fn update_control_bits(&mut self, i2c: &mut impl I2c, bits: u8, enable: bool) -> bool {
        let control = match self.read_reg(i2c, REG_CONTROL) {
            Some(v) => v,
            None => return false,
        };

        let next = if enable { control | bits } else { control & !bits };
        self.write_reg(i2c, REG_CONTROL, next)
    }

    fn clear_status_bits(&mut self, i2c: &mut impl I2c, bits: u8) -> bool {
        let status = match self.read_reg(i2c, REG_STATUS) {
            Some(v) => v,
            None => return false,
        };

        self.write_reg(i2c, REG_STATUS, status & !bits)
    }

    fn read_reg(&self, i2c: &mut impl I2c, reg: u8) -> Option<u8> {
        let mut b = [0u8; 1];
        if i2c.write_read(self.rtc_addr, &[reg], &mut b).is_err() {
            return None;
        }
        Some(b[0])
    }

    fn write_reg(&self, i2c: &mut impl I2c, reg: u8, value: u8) -> bool {
        i2c.write(self.rtc_addr, &[reg, value]).is_ok()
    }
}

impl From<DateTime> for Alarm1DateTime {
    fn from(value: DateTime) -> Self {
        Self {
            sec: value.sec,
            min: value.min,
            hour: value.hour,
            day_or_date: value.day,
            day_mode: false,
        }
    }
}

impl From<DateTime> for Alarm2DateTime {
    fn from(value: DateTime) -> Self {
        Self {
            min: value.min,
            hour: value.hour,
            day_or_date: value.day,
            day_mode: false,
        }
    }
}
