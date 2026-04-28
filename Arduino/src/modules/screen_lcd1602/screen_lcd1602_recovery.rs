pub const LCD_CMD_LONG_DELAY_MS: u32 = 2;
pub const LCD_ASYNC_STEP_DELAY_MS: u32 = 1;
pub const LCD_RECOVERY_FAIL_STREAK: u8 = 8;
pub const LCD_STALE_TIMEOUT_MS: u32 = 15_000;
pub const LCD_RECOVERY_COOLDOWN_MS: u32 = 5_000;

#[derive(Copy, Clone, Debug)]
pub struct RecoverModule {
    stale_timeout_ms: u32,
    recovery_cooldown_ms: u32,
    fail_streak_threshold: u8,

    now_ms: u32,
    last_i2c_ok_ms: u32,
    last_recovery_ms: u32,
    has_i2c_ok: bool,
    in_recovery: bool,
    fail_streak: u8,
}

impl RecoverModule {
    pub const fn new(stale_timeout_ms: u32, recovery_cooldown_ms: u32, fail_streak_threshold: u8) -> Self {
        Self {
            stale_timeout_ms,
            recovery_cooldown_ms,
            fail_streak_threshold,
            now_ms: 0,
            last_i2c_ok_ms: 0,
            last_recovery_ms: 0,
            has_i2c_ok: false,
            in_recovery: false,
            fail_streak: 0,
        }
    }

    pub const fn recommended() -> Self {
        Self::new(
            LCD_STALE_TIMEOUT_MS,
            LCD_RECOVERY_COOLDOWN_MS,
            LCD_RECOVERY_FAIL_STREAK,
        )
    }

    pub fn set_now(&mut self, now_ms: u32) {
        self.now_ms = now_ms;
    }

    pub fn mark_i2c_ok(&mut self) {
        self.fail_streak = 0;
        self.has_i2c_ok = true;
        self.last_i2c_ok_ms = self.now_ms;
    }

    pub fn mark_i2c_error_and_need_recover(&mut self) -> bool {
        self.fail_streak = self.fail_streak.saturating_add(1);
        !self.in_recovery && self.fail_streak >= self.fail_streak_threshold
    }

    pub fn begin_recovery(&mut self) {
        self.in_recovery = true;
        self.last_recovery_ms = self.now_ms;
        self.fail_streak = 0;
    }

    pub fn finish_recovery(&mut self) {
        self.in_recovery = false;
    }

    pub fn need_recovery(&self, display_on: bool) -> bool {
        if !display_on || self.in_recovery || !self.has_i2c_ok {
            return false;
        }

        if self.now_ms.wrapping_sub(self.last_i2c_ok_ms) < self.stale_timeout_ms {
            return false;
        }

        if self.now_ms.wrapping_sub(self.last_recovery_ms) < self.recovery_cooldown_ms {
            return false;
        }

        true
    }
}