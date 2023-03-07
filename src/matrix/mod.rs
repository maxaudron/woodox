use defmt::Format;

#[derive(Debug, Format, Copy, Clone)]
pub struct Switch {
    /// Last read value
    value: u8,
    /// individual position correction
    comp: u8,

    /// ID of the mux this switch is attached to
    mux: u8,
    /// Channel of the mux this switch is attached to
    channel: u8,
}

impl Default for Switch {
    fn default() -> Self {
        Self {
            value: u8::MAX,
            comp: 26,
            mux: 0,
            channel: 0,
        }
    }
}

impl Switch {
    pub fn new(mux: u8, channel: u8) -> Self {
        Self {
            mux,
            channel,
            ..Default::default()
        }
    }

    pub fn value(&mut self, value: u8) {
        self.value = crate::hall::distance_u8(value) - self.comp
    }
}

/// Number of multiplexers connected to ADC
const NUM_MUX: usize = 4;
const NUM_SWITCHES: usize = 8;
const NUM_SCANS: usize = 8;

/// A Single scan action to be taken on multiple switches
#[derive(Default, Debug, Format, Copy, Clone)]
pub struct Scan {
    pub switches: [Switch; NUM_MUX],
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

        switches.into_iter().for_each(|s| {
            scans[s.channel as usize].switches[s.mux as usize] = s
        });

        ScanOrder { scans }
    }
}
