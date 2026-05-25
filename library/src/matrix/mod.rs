#![doc = include_str!("readme.md")]

use defmt::Format;

use crate::layout::*;

mod hall;
mod keys;
mod switch;

pub use hall::*;
pub use keys::*;
pub use switch::*;

/// A Single scan action to be taken on multiple switches
///
/// The rp2040 ADC for example has an internal 4 pin mux that allows it
/// to read multiple values from the ADC in quick succession.
#[derive(Default, Debug, Format, Copy, Clone)]
pub struct Scan {
    pub switches: [Switch; NUM_MUX],
}

impl Scan {
    pub const fn new() -> Self {
        Scan {
            switches: [Switch::new(0, 0); NUM_MUX],
        }
    }

    pub fn update(&mut self, values: [u8; NUM_MUX]) {
        self.switches
            .iter_mut()
            .zip(values)
            .for_each(|(switch, value)| switch.update(value))
    }
}

/// The order we will execute all the scans in
///
/// Technically the order does not matter, but we want to keep it
/// consistent so we can map the index of the scan and switch to
/// an exact position in our [`Layer`]
///
/// [`Layer`]: crate::matrix::Layer
#[derive(Default, Debug, Format, Copy, Clone)]
pub struct ScanOrder {
    pub scans: [Scan; NUM_CHANNELS],
}

impl ScanOrder {
    /// Order the user representation of the keyboard layout for runtime.
    pub fn new(switches: [Switch; NUM_SWITCHES]) -> Self {
        let mut scans = [Scan::new(); NUM_CHANNELS];

        switches.into_iter().enumerate().for_each(|(i, mut switch)| {
            switch.index = i;
            scans[switch.channel as usize].switches[switch.mux as usize] = switch
        });

        ScanOrder { scans }
    }
}
