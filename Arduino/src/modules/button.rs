use arduino_hal::port::mode::{Input, Output, PullUp};
use arduino_hal::port::{mode, Pin, PinOps};

pub struct Button<IN>{
    input_pin: Pin<mode::Input<PullUp>, IN>
}


impl<IN> Button<IN>
where
    IN: PinOps
{
    pub fn new(input: Pin<mode::Input<PullUp>, IN>) -> Self{
        Self {
            input_pin: input,
        }
    }

    pub fn is_pressed(&self) -> bool{
        self.input_pin.is_low()
    }
}