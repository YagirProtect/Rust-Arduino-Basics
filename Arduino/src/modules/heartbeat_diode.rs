use arduino_hal::port::{mode, Pin, PinOps};

pub struct HeartbeatDiode<PW>{
    power_pin: Pin<mode::Output, PW>,
    time: u32,
    blink_time: u32,
    state: bool
}


impl<PW> HeartbeatDiode<PW>
where
    PW: PinOps
{
    pub fn new(power_pin: Pin<mode::Output, PW>, blink_time: u32) -> Self {
        Self{
            power_pin,
            time: 0,
            blink_time,
            state: false,
        }
    }
    
    
    pub fn update(&mut self, time: u32){
        if (time.wrapping_sub(self.time) >= self.blink_time){
            self.time = time;
            self.state = !self.state;
            
            if (self.state == false){
                self.power_pin.set_low();
            }else{
                self.power_pin.set_high();
            }
        }
    }
}