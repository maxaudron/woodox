use defmt::Format;

use crate::layout::HOLD_TIME;

use super::keys::Keymap;

/// A single switch in our keyboard
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

    pub hold_counter: u32,
    pub held: bool,
}

impl Default for Switch {
    fn default() -> Self {
        Self {
            position: u8::MAX,
            comp: 26,
            mux: 0,
            channel: 0,
            trig_lower: 20,
            trig_upper: 22,
            index: 0,

            pressed: false,
            hold_counter: 0,
            held: false,
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
        self.position = super::hall::distance(value) - self.comp
    }

    /// Update the state of the switch and asociated key with the raw ADC value
    pub fn update(&mut self, value: u8) {
        self.value(value);

        if self.pressed {
            if self.position >= self.trig_upper {
                self.pressed = false;
                Keymap::set_key(self.index, false, self.held);

                if self.held {
                    self.held = false;
                    Keymap::set_hold(self.index, false);
                }

            } else {
                if !self.held && self.hold_counter >= HOLD_TIME {
                    self.held = true;
                    self.hold_counter = 0;
                    Keymap::set_hold(self.index, true);
                } else {
                    self.hold_counter += 1;
                }
            }
        } else if self.position <= self.trig_lower {
            self.pressed = true;

            Keymap::set_key(self.index, true, false);
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
