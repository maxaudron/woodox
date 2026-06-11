use crate::hal::gpio::{
    FunctionSio, Pin, PullDown, SioOutput,
    bank0::{Gpio29, Gpio30, Gpio31, Gpio32},
};
use embedded_hal::digital::OutputPin;

use crate::hardware::mux::Mux;

pub type MuxEnable = Pin<Gpio32, FunctionSio<SioOutput>, PullDown>;
pub type MuxPin1 = Pin<Gpio31, FunctionSio<SioOutput>, PullDown>;
pub type MuxPin2 = Pin<Gpio30, FunctionSio<SioOutput>, PullDown>;
pub type MuxPin3 = Pin<Gpio29, FunctionSio<SioOutput>, PullDown>;

pub struct CD74HC4051<A, B, C, D> {
    enable: A,
    pin_0: B,
    pin_1: C,
    pin_2: D,
}

impl<A, B, C, D> CD74HC4051<A, B, C, D>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
{
    pub fn new(mut enable: A, mut pin_0: B, mut pin_1: C, mut pin_2: D) -> Self {
        // Set to output 0
        pin_0.set_low().unwrap();
        pin_1.set_low().unwrap();
        pin_2.set_low().unwrap();

        enable.set_low().unwrap();

        Self {
            enable,
            pin_0,
            pin_1,
            pin_2,
        }
    }
}

impl<A, B, C, D> Mux for CD74HC4051<A, B, C, D>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
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

        // Wait one cpu cycle (7.5ns rp2040) (6.7 rp2350) for mux to settle
        // FIXME TODO try without this
        cortex_m::asm::delay(28); // 120ns
    }
}
