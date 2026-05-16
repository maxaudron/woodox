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

    /// If repid trigger is enabled for this switch,
    pub rapid_enabled: bool,
    /// If the switch is currently pressed
    pub rapid_pressed: bool,
    /// Last position that triggered a press or release
    pub rapid_position: u8,
    /// Boundry that triggers a press
    pub rapid_lower: u8,
    /// Boundry that triggers a release
    pub rapid_upper: u8,

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
            pressed: false,

            trig_lower: 20,
            trig_upper: 22,

            rapid_enabled: false,
            rapid_pressed: false,
            rapid_position: u8::MAX,
            rapid_lower: 4,
            rapid_upper: 4,

            comp: 26,

            mux: 0,
            channel: 0,
            index: 0,

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
    #[inline(always)]
    pub fn value(&mut self, value: u8) -> u8 {
        super::hall::distance(value) - self.comp
    }

    #[inline(always)]
    pub fn pressed(&mut self, rapid: bool) {
        if rapid {
            self.rapid_pressed = true;
        } else {
            self.pressed = true;
        }

        self.rapid_position = self.position;
        Keymap::set_key(self.index, true, false);
    }

    #[inline(always)]
    pub fn released(&mut self, rapid: bool) {
        if rapid {
            self.rapid_pressed = false;
        } else {
            self.rapid_pressed = false;
            self.pressed = false;
        }

        self.rapid_position = self.position;
        Keymap::set_key(self.index, false, self.held);

        if self.held {
            self.held = false;
            Keymap::set_hold(self.index, false);
        }
    }

    #[inline(always)]
    pub fn held(&mut self, _rapid: bool) {
        if !self.held && self.hold_counter >= HOLD_TIME {
            self.held = true;
            self.hold_counter = 0;
            Keymap::set_hold(self.index, true);
        } else {
            self.hold_counter += 1;
        }
    }

    #[inline(always)]
    pub fn update_rapid(&mut self) {
        if self.rapid_pressed {
            if self.position >= (self.rapid_position + self.rapid_upper) {
                self.rapid_position = self.position;
                self.released(true)
            } else {
                self.held(true);
            }
        } else if self.position <= (self.rapid_position - self.rapid_lower) {
            self.rapid_position = self.position;
            self.pressed(true)
        }
    }

    /// Update the state of the switch and asociated key with the raw ADC value
    pub fn update(&mut self, value: u8) {
        self.position = self.value(value);

        if self.pressed {
            if self.position >= self.trig_upper {
                self.released(false)
            } else {
                self.held(false);
            }
        } else if self.position <= self.trig_lower {
            self.pressed(false)
        }

        if self.rapid_enabled {
            self.update_rapid()
        }
    }
}

#[test]
fn switch_update_trigger() {
    let mut switch = Switch::default();

    switch.update(117);
    assert!(switch.pressed);
    switch.update(110);
    assert!(switch.pressed);
    switch.update(107);
    assert!(!switch.pressed);
    switch.update(98);
    assert!(!switch.pressed);
}

#[test]
fn switch_update_rapid_trigger() {
    let mut switch = Switch {
        rapid_enabled: true,
        ..Default::default()
    };

    switch.update(117);
    assert!(switch.pressed);
    switch.update(147);
    assert!(switch.rapid_pressed);
    switch.update(128);
    assert!(!switch.rapid_pressed);
    switch.update(147);
    assert!(switch.rapid_pressed);
    switch.update(98);
    assert!(!switch.pressed);
}
