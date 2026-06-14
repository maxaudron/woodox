#![doc = include_str!("readme.md")]

use defmt::{debug, Format};

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
            .for_each(|(switch, value)| switch.update_raw(value))
    }
}

impl IntoIterator for Scan {
    type Item = Switch;

    type IntoIter = core::array::IntoIter<Switch, NUM_MUX>;

    fn into_iter(self) -> Self::IntoIter {
        self.switches.into_iter()
    }
}

impl IntoIterator for &Scan {
    type Item = Switch;

    type IntoIter = core::array::IntoIter<Switch, NUM_MUX>;

    fn into_iter(self) -> Self::IntoIter {
        self.switches.into_iter()
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

    pub fn debug_position(&self) {
        debug!(
            "{:03} {:03} {:03} {:03} {:03} {:03}",
            self.scans[1].switches[0].position,
            self.scans[0].switches[0].position,
            self.scans[5].switches[0].position,
            self.scans[7].switches[0].position,
            self.scans[6].switches[0].position,
            self.scans[4].switches[0].position,
        );
        debug!(
            "{:03} {:03} {:03} {:03} {:03} {:03}",
            self.scans[1].switches[1].position,
            self.scans[0].switches[1].position,
            self.scans[5].switches[1].position,
            self.scans[7].switches[1].position,
            self.scans[6].switches[1].position,
            self.scans[4].switches[1].position,
        );
        debug!(
            "{:03} {:03} {:03} {:03} {:03} {:03}",
            self.scans[1].switches[2].position,
            self.scans[0].switches[2].position,
            self.scans[5].switches[2].position,
            self.scans[7].switches[2].position,
            self.scans[6].switches[2].position,
            self.scans[4].switches[2].position,
        );
        debug!(
            "{:03} {:03} {:03} {:03} {:03} {:03} {:03} {:03}",
            self.scans[1].switches[3].position,
            self.scans[0].switches[3].position,
            self.scans[5].switches[3].position,
            self.scans[7].switches[3].position,
            self.scans[6].switches[3].position,
            self.scans[4].switches[3].position,
            self.scans[3].switches[0].position,
            self.scans[3].switches[1].position,
        );
        debug!(
            "  {:03}   {:03} {:03}   {:03}   {:03} {:03}",
            self.scans[2].switches[0].position,
            self.scans[2].switches[1].position,
            self.scans[2].switches[2].position,
            self.scans[2].switches[3].position,
            self.scans[3].switches[3].position,
            self.scans[3].switches[2].position,
        );
    }
}

impl IntoIterator for ScanOrder {
    type Item = Scan;

    type IntoIter = core::array::IntoIter<Scan, NUM_CHANNELS>;

    fn into_iter(self) -> Self::IntoIter {
        self.scans.into_iter()
    }
}

impl IntoIterator for &ScanOrder {
    type Item = Scan;

    type IntoIter = core::array::IntoIter<Scan, NUM_CHANNELS>;

    fn into_iter(self) -> Self::IntoIter {
        self.scans.into_iter()
    }
}
