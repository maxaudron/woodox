use defmt::Format;

use crate::layout::*;

mod hall;
mod keys;
mod switch;

pub use keys::*;
pub use switch::*;

/// A Single scan action to be taken on multiple switches
#[derive(Default, Debug, Format, Copy, Clone)]
pub struct Scan {
    pub switches: [Switch; NUM_MUX],
}

impl Scan {
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
/// consistent during one run at least.
#[derive(Default, Debug, Format, Copy, Clone)]
pub struct ScanOrder {
    pub scans: [Scan; NUM_SCANS],
}

impl ScanOrder {
    pub fn new(switches: [Switch; NUM_SWITCHES]) -> Self {
        let mut scans = [Scan::default(); NUM_SCANS];

        switches
            .into_iter()
            .enumerate()
            .for_each(|(i, mut switch)| {
                switch.index = i;
                scans[switch.channel as usize].switches[switch.mux as usize] = switch
            });

        ScanOrder { scans }
    }
}
