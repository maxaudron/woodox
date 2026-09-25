use fugit::{HertzU32, RateExtU32};

use crate::hal::{
    gpio::{
        FunctionI2C, Pin, PullUp,
        bank0::{Gpio4, Gpio5},
    },
    pac::{I2C0, RESETS},
};

pub struct I2C {
    i2c: crate::hal::I2C<I2C0, (Pin<Gpio4, FunctionI2C, PullUp>, Pin<Gpio5, FunctionI2C, PullUp>)>,
}

impl I2C {
    pub fn new(dev: I2C0, pins: (Pin<Gpio4, FunctionI2C, PullUp>, Pin<Gpio5, FunctionI2C, PullUp>), resets: &mut RESETS) -> Self {
        let i2c = crate::hal::I2C::i2c0(dev, pins.0, pins.1, 400.kHz(), resets, 125_000_000.Hz());

        Self { i2c }
    }
}
