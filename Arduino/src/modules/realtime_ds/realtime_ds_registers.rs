pub(crate) mod common {
    pub(crate) const SECONDS_MASK: u8 = 0x7F;
    pub(crate) const MINUTES_MASK: u8 = 0x7F;
    pub(crate) const HOURS_MASK: u8 = 0x3F; // 24-hour mode
    pub(crate) const DAY_MASK: u8 = 0x3F;
    pub(crate) const DAY_WEEK_MASK: u8 = 0x07;
    pub(crate) const MONTH_MASK: u8 = 0x1F;
    pub(crate) const YEAR_MASK: u8 = 0xFF;
}

pub(crate) mod ds3231 {
    pub(crate) const REG_SECONDS: u8 = 0x00;
    pub(crate) const REG_MINUTES: u8 = 0x01;
    pub(crate) const REG_HOURS: u8 = 0x02;
    pub(crate) const REG_DAY_WEEK: u8 = 0x03;
    pub(crate) const REG_DAY: u8 = 0x04;
    pub(crate) const REG_MONTH: u8 = 0x05;
    pub(crate) const REG_YEAR: u8 = 0x06;

    pub(crate) const REG_ALARM1_SECONDS: u8 = 0x07;
    pub(crate) const REG_ALARM2_MINUTES: u8 = 0x0B;
    pub(crate) const REG_CONTROL: u8 = 0x0E;
    pub(crate) const REG_STATUS: u8 = 0x0F;


    pub(crate) const ALARM_MASK_BIT: u8 = 1 << 7;
    pub(crate) const DY_DT_BIT: u8 = 1 << 6;
    
    pub(crate) const CONTROL_INTCN_BIT: u8 = 1 << 2;
    pub(crate) const CONTROL_A2IE_BIT: u8 = 1 << 1;
    pub(crate) const CONTROL_A1IE_BIT: u8 = 1 << 0;
    
    pub(crate) const STATUS_A2F_BIT: u8 = 1 << 1;
    pub(crate) const STATUS_A1F_BIT: u8 = 1 << 0;
}

pub(crate) mod ds1302 {
    pub(crate) const REG_SECONDS: u8 = 0x80;
    pub(crate) const REG_MINUTES: u8 = 0x82;
    pub(crate) const REG_HOURS: u8 = 0x84;
    pub(crate) const REG_DAY: u8 = 0x86; // day of month
    pub(crate) const REG_MONTH: u8 = 0x88;
    pub(crate) const REG_DAY_WEEK: u8 = 0x8A; // day of week
    pub(crate) const REG_YEAR: u8 = 0x8C;
    pub(crate) const REG_CONTROL: u8 = 0x8E;

    pub(crate) const CONTROL_WP_BIT: u8 = 0x80;
}
