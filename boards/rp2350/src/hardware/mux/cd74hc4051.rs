use embedded_hal::digital::OutputPin;

use crate::hardware::mux::Mux;

pub struct CD74HC4051<A, B, C> {
    pin_0: A,
    pin_1: B,
    pin_2: C,
}

impl<A, B, C> CD74HC4051<A, B, C>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
{
    pub fn new(mut pin_0: A, mut pin_1: B, mut pin_2: C) -> Self {
        // Set to output 0
        pin_0.set_low().unwrap();
        pin_1.set_low().unwrap();
        pin_2.set_low().unwrap();

        Self { pin_0, pin_1, pin_2 }
    }
}

impl<A, B, C> Mux for CD74HC4051<A, B, C>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
{
    /// Enable output `n`. `n` must be between 0 and 7 inclusive.
    /// If a SelectPinError occurs, the select is left in a possibly unwanted state, but it is disabled here.
    fn set_output_active(&mut self, output: u8) {
        let is_bit_set = |b: u8| -> bool { output & (1 << b) != 0 };

        if is_bit_set(0) {
            self.pin_0.set_high().unwrap();
        } else {
            self.pin_0.set_low().unwrap();
        }
        if is_bit_set(1) {
            self.pin_1.set_high().unwrap();
        } else {
            self.pin_1.set_low().unwrap();
        }
        if is_bit_set(2) {
            self.pin_2.set_high().unwrap();
        } else {
            self.pin_2.set_low().unwrap();
        }

        // Wait one cpu cycle (8ns) for mux to settle
        // FIXME TODO try without this
        cortex_m::asm::delay(1);
    }
}
