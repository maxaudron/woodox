use defmt::Format;

use super::keys::Keymap;

#[derive(Debug, Format, Copy, Clone)]
pub struct Switch {
    /// Switch position in units of 0.1mm
    pub position: u8,
    /// If the switch is currently pressed
    pub pressed: bool,

    /// Boundry that triggers a press
    pub trig_lower: u8,
    /// Boundry that triggers a release
    pub trig_upper: u8,

    /// individual position correction
    pub comp: u8,
    /// ID of the mux this switch is attached to
    pub mux: u8,
    /// Channel of the mux this switch is attached to
    pub channel: u8,

    /// Index of the switch used to map it
    /// to a location in the keymaps
    pub index: usize,
}

impl Default for Switch {
    fn default() -> Self {
        Self {
            position: u8::MAX,
            comp: 26,
            mux: 0,
            channel: 0,
            pressed: false,
            trig_lower: 20,
            trig_upper: 22,
            index: 0,
        }
    }
}

impl Switch {
    /// Initialize a new switch with default values
    ///
    /// `mux` is the id of the ADC pin the multiplexer is attached to.
    /// `channel` is the channel of the multiplexer this switch is attached to.
    pub fn new(mux: u8, channel: u8) -> Self {
        Self {
            mux,
            channel,
            ..Default::default()
        }
    }

    /// Set the position of the switch based on raw ADC value
    pub fn value(&mut self, value: u8) {
        self.position = super::hall::distance_u8(value) - self.comp
    }

    /// Update the state of the switch and asociated key with the raw ADC value
    pub fn update(&mut self, value: u8) {
        self.value(value);

        if self.pressed {
            if self.position >= self.trig_upper {
                self.pressed = false;
                Keymap::set_key(self.index, false)
            }
        } else if self.position <= self.trig_lower {
            self.pressed = true;
            Keymap::set_key(self.index, true);
        }
    }
}

#[test]
fn switch_update_trigger() {
    let mut switch = Switch::default();

    switch.update(117);
    assert_eq!(switch.pressed, true);
    switch.update(110);
    assert_eq!(switch.pressed, true);
    switch.update(107);
    assert_eq!(switch.pressed, false);
    switch.update(98);
    assert_eq!(switch.pressed, false);
}
