use defmt::Format;

use crate::layout::HOLD_TIME;

#[derive(Debug, Default, Format, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SwitchState {
    Pressed,
    #[default]
    Unpressed,
    RapidPressed,
    RapidUnpressed,
    Held,
    OneShot,
}

impl SwitchState {
    pub const fn new() -> Self {
        Self::Unpressed
    }

    pub fn is_oneshot(&self) -> bool {
        *self == Self::OneShot
    }

    pub fn is_pressed(&self) -> bool {
        *self == Self::Pressed || *self == Self::RapidPressed || *self == Self::Held
    }

    pub fn is_held(&self) -> bool {
        *self == Self::Held
    }
}

/// A single switch in our keyboard
#[derive(Debug, Format, Copy, Clone)]
pub struct Switch {
    /// Switch position in units of 0.1mm
    pub position: u8,
    /// Whether the switch is pressed, held, etc.
    pub state: SwitchState,
    /// How long the switch has been held for in cycles
    pub hold_counter: u32,

    /// Boundry that triggers a press
    pub trig_press: u8,
    /// Boundry that triggers a release
    pub trig_release: u8,

    /// If repid trigger is enabled for this switch,
    pub rapid_enabled: bool,
    /// Last position that triggered a press or release
    pub rapid_last_position: u8,
    /// Boundry that triggers a press
    pub rapid_lower: u8,
    /// Boundry that triggers a release
    pub rapid_upper: u8,

    /// individual position correction
    pub offset: u8,

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
            position: 0,

            trig_press: 20,
            trig_release: 18,

            rapid_enabled: false,
            rapid_last_position: 0,
            rapid_lower: 4,
            rapid_upper: 4,

            offset: 26,

            mux: 0,
            channel: 0,
            index: 0,

            hold_counter: 0,

            state: SwitchState::Unpressed,
        }
    }
}

impl Switch {
    /// Initialize a new switch with default values
    ///
    /// `mux` is the id of the ADC pin the multiplexer is attached to.
    /// `channel` is the channel of the multiplexer this switch is attached to.
    pub const fn new(mux: u8, channel: u8) -> Self {
        Self {
            mux,
            channel,
            position: 0,
            trig_press: 20,
            trig_release: 18,
            rapid_enabled: false,
            rapid_last_position: 0,
            rapid_lower: 4,
            rapid_upper: 4,
            offset: 0,
            index: 0,
            hold_counter: 0,

            state: SwitchState::new(),
        }
    }

    /// Calculate the mm travel distance based on raw ADC input and
    /// switch defined offset
    #[inline(always)]
    pub fn value(&mut self, value: u8) -> u8 {
        super::hall::distance_travel(value)
    }

    #[inline(always)]
    pub fn pressed(&mut self, rapid: bool) {
        if rapid {
            self.state = SwitchState::RapidPressed
        } else {
            self.state = SwitchState::Pressed
        }

        self.rapid_last_position = self.position;
    }

    #[inline(always)]
    pub fn released(&mut self, rapid: bool) {
        if rapid && self.position > self.trig_press {
            self.state = SwitchState::RapidUnpressed
        } else {
            self.state = SwitchState::Unpressed
        }

        self.hold_counter = 0;
        self.rapid_last_position = self.position;
    }

    #[inline(always)]
    pub fn held(&mut self, _rapid: bool) {
        if !self.state.is_held() && self.hold_counter >= HOLD_TIME {
            // Switch has been in pressed state long enough so we switch to held
            self.hold_counter = 0;
            self.state = SwitchState::Held;
        } else {
            // We keep incrementing the hold counter every cycle
            self.hold_counter += 1;
        }
    }

    #[inline(always)]
    pub fn update_rapid(&mut self) {
        if self.state == SwitchState::RapidPressed {
            if self.position <= (self.rapid_last_position.saturating_sub(self.rapid_upper)) {
                self.rapid_last_position = self.position;
                self.released(true)
            } else {
                if self.position > self.rapid_last_position {
                    self.rapid_last_position = self.position;
                }
                self.held(true);
            }
        } else if (self.state == SwitchState::RapidUnpressed
            && self.position >= (self.rapid_last_position.saturating_add(self.rapid_lower)))
            || self.position >= self.trig_press
        {
            self.rapid_last_position = self.position;
            self.pressed(true)
        }
    }

    /// Update the state of the switch and asociated key with the travel in mm
    pub fn update(&mut self, travel: u8) {
        self.position = travel;

        if self.rapid_enabled {
            self.update_rapid()
        } else {
            if self.state == SwitchState::Pressed || self.state == SwitchState::Held {
                if self.position <= self.trig_release {
                    self.released(false)
                } else {
                    self.held(false);
                }
            } else if self.position >= self.trig_press {
                self.pressed(false)
            }
        }
    }

    /// Update the state of the switch and asociated key with the raw ADC value
    pub fn update_raw(&mut self, value: u8) {
        let travel = self.value(value);
        self.update(travel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_switch() -> Switch {
        Switch::default()
    }

    fn rapid_switch() -> Switch {
        Switch {
            rapid_enabled: true,
            ..Default::default()
        }
    }

    // ── SwitchState ──────────────────────────────────────────────

    #[test]
    fn state_new_is_unpressed() {
        assert_eq!(SwitchState::new(), SwitchState::Unpressed);
    }

    #[test]
    fn state_default_is_unpressed() {
        assert_eq!(SwitchState::default(), SwitchState::Unpressed);
    }

    #[test]
    fn state_is_pressed_variants() {
        assert!(!SwitchState::Unpressed.is_pressed());
        assert!(SwitchState::Pressed.is_pressed());
        assert!(SwitchState::RapidPressed.is_pressed());
        assert!(SwitchState::Held.is_pressed());
        assert!(!SwitchState::OneShot.is_pressed());
    }

    #[test]
    fn state_is_held() {
        assert!(!SwitchState::Unpressed.is_held());
        assert!(!SwitchState::Pressed.is_held());
        assert!(!SwitchState::RapidPressed.is_held());
        assert!(SwitchState::Held.is_held());
        assert!(!SwitchState::OneShot.is_held());
    }

    #[test]
    fn state_is_oneshot() {
        assert!(!SwitchState::Unpressed.is_oneshot());
        assert!(!SwitchState::Pressed.is_oneshot());
        assert!(!SwitchState::RapidPressed.is_oneshot());
        assert!(!SwitchState::Held.is_oneshot());
        assert!(SwitchState::OneShot.is_oneshot());
    }

    // ── Switch::new vs Default ───────────────────────────────────

    #[test]
    fn new_sets_mux_and_channel() {
        let s = Switch::new(3, 7);
        assert_eq!(s.mux, 3);
        assert_eq!(s.channel, 7);
        assert_eq!(s.state, SwitchState::Unpressed);
    }

    #[test]
    fn default_has_expected_thresholds() {
        let s = default_switch();
        assert_eq!(s.trig_press, 20);
        assert_eq!(s.trig_release, 18);
        assert_eq!(s.rapid_lower, 4);
        assert_eq!(s.rapid_upper, 4);
        assert_eq!(s.offset, 26);
        assert_eq!(s.position, 0);
        assert_eq!(s.rapid_last_position, 0);
    }

    // ── Switch::pressed() / released() direct ────────────────────

    #[test]
    fn pressed_normal_sets_state_and_rapid_position() {
        let mut s = default_switch();
        s.position = 10;
        s.pressed(false);

        assert_eq!(s.state, SwitchState::Pressed);
        assert_eq!(s.rapid_last_position, 10);
    }

    #[test]
    fn pressed_rapid_sets_state_and_rapid_position() {
        let mut s = default_switch();
        s.position = 10;
        s.pressed(true);

        assert_eq!(s.state, SwitchState::RapidPressed);
        assert_eq!(s.rapid_last_position, 10);
    }

    #[test]
    fn released_normal_clears_pressed() {
        let mut s = default_switch();
        s.state = SwitchState::Pressed;
        s.position = 50;
        s.released(false);

        assert_eq!(s.state, SwitchState::Unpressed);
        assert_eq!(s.rapid_last_position, 50);
    }

    #[test]
    fn released_normal_clears_held() {
        let mut s = default_switch();
        s.state = SwitchState::Held;
        s.position = 50;
        s.released(false);

        assert_eq!(s.state, SwitchState::Unpressed);
        assert_eq!(s.rapid_last_position, 50);
    }

    #[test]
    fn released_normal_rapid_pressed() {
        let mut s = default_switch();
        s.state = SwitchState::RapidPressed;
        s.position = 50;
        s.released(false);

        assert_eq!(s.state, SwitchState::Unpressed);
        assert_eq!(s.rapid_last_position, 50);
    }

    #[test]
    fn released_rapid_always_clears() {
        let mut s = default_switch();
        s.state = SwitchState::RapidPressed;
        s.position = 50;
        s.released(true);

        assert_eq!(s.state, SwitchState::RapidUnpressed);
        assert_eq!(s.rapid_last_position, 50);
    }

    #[test]
    fn released_resets_hold_counter() {
        let mut s = default_switch();
        s.state = SwitchState::Held;
        s.hold_counter = 500;
        s.position = 50;

        s.released(false);

        assert_eq!(s.state, SwitchState::Unpressed);
        assert_eq!(s.hold_counter, 0);
    }

    // ── Switch::held() ───────────────────────────────────────────

    #[test]
    fn held_increments_counter_when_not_yet_held() {
        let mut s = default_switch();
        s.state = SwitchState::Pressed;

        s.held(false);

        assert_eq!(s.hold_counter, 1);
        assert_eq!(s.state, SwitchState::Pressed);
    }

    #[test]
    fn held_triggers_at_hold_time() {
        let mut s = default_switch();
        s.state = SwitchState::Pressed;
        s.hold_counter = HOLD_TIME;

        s.held(false);

        assert_eq!(s.hold_counter, 0);
        assert_eq!(s.state, SwitchState::Held);
    }

    #[test]
    fn held_does_not_trigger_below_hold_time() {
        let mut s = default_switch();
        s.state = SwitchState::Pressed;
        s.hold_counter = HOLD_TIME - 1;

        s.held(false);

        assert!(!s.state.is_held());
        assert_eq!(s.hold_counter, HOLD_TIME);
    }

    #[test]
    fn held_keeps_incrementing_once_held() {
        let mut s = default_switch();
        s.state = SwitchState::Held;
        s.hold_counter = HOLD_TIME;

        s.held(false);

        assert_eq!(s.hold_counter, HOLD_TIME + 1);
        assert_eq!(s.state, SwitchState::Held);
    }

    #[test]
    fn held_can_retrigger_after_release() {
        let mut s = default_switch();

        s.state = SwitchState::Pressed;
        s.hold_counter = HOLD_TIME;
        s.held(false);
        assert_eq!(s.state, SwitchState::Held);

        s.released(false);
        assert_eq!(s.state, SwitchState::Unpressed);

        s.state = SwitchState::Pressed;
        s.hold_counter = HOLD_TIME;
        s.held(false);
        assert_eq!(s.state, SwitchState::Held);
    }

    // ── Switch::update() — normal trigger ────────────────────────

    #[test]
    fn update_press_at_threshold() {
        let mut s = default_switch();
        s.update(20);
        assert_eq!(s.state, SwitchState::Pressed);
    }

    #[test]
    fn update_no_press_above_threshold() {
        let mut s = default_switch();
        // trig_lower = 20, 20 <= 20 -> pressed
        s.update(20);
        assert!(s.state.is_pressed());

        let mut s2 = default_switch();
        // 18 <= 20 is false -> not pressed
        s2.update(18);
        assert!(!s2.state.is_pressed());
    }

    #[test]
    fn update_release_at_upper_threshold() {
        let mut s = default_switch();
        s.update(20);
        assert!(s.state.is_pressed());

        // trig_upper = 18, 18 >= 18 -> released
        s.update(18);
        assert_eq!(s.state, SwitchState::Unpressed);
    }

    #[test]
    fn update_stays_pressed_between_thresholds() {
        let mut s = default_switch();
        s.update(20);
        assert!(s.state.is_pressed());

        // distance(110) = 47, comp = 26, position = 21
        // trig_upper = 18, 21 >= 18 is false -> stays pressed
        s.update(21);
        assert!(s.state.is_pressed());
    }

    #[test]
    fn update_hysteresis_prevents_bounce() {
        let mut s = default_switch();

        s.update(20); // pos = 20, <= 20 -> press
        assert!(s.state.is_pressed());

        s.update(19); // pos = 19, < 18 -> still pressed
        assert!(s.state.is_pressed());

        s.update(18); // pos = 18, >= 18 -> release
        assert!(!s.state.is_pressed());

        s.update(19); // pos = 21, > 20 -> stays unpressed
        assert!(!s.state.is_pressed());

        s.update(20); // pos = 20, <= 20 -> press again
        assert!(s.state.is_pressed());
    }

    #[test]
    fn update_does_not_overwrite_rapid_pressed() {
        let mut s = rapid_switch();
        s.state = SwitchState::RapidPressed;
        s.position = 15;
        s.rapid_last_position = 15;

        // pos = 15 is below trig_lower(20), but update() should
        // not fire pressed(false) when already in RapidPressed
        s.update(15); // pos = 15
        assert_eq!(s.state, SwitchState::RapidPressed);
    }

    // ── Switch::update() — hold via update ───────────────────────

    #[test]
    fn update_hold_accumulates_while_pressed() {
        let mut s = default_switch();
        s.update(20);
        assert!(s.state.is_pressed());
        assert_eq!(s.hold_counter, 0);

        for i in 1..=5 {
            s.update(20);
            assert_eq!(s.hold_counter, i);
        }
    }

    #[test]
    fn update_hold_triggers_after_hold_time_plus_one_updates() {
        let mut s = default_switch();
        s.update(20); // initial press, hold_counter = 0

        for _ in 0..HOLD_TIME {
            s.update(20);
        }
        assert!(!s.state.is_held());
        assert_eq!(s.hold_counter, HOLD_TIME);

        s.update(20);
        assert_eq!(s.state, SwitchState::Held);
    }

    #[test]
    fn update_held_key_can_still_be_released() {
        let mut s = default_switch();
        s.update(20);

        for _ in 0..=HOLD_TIME {
            s.update(20);
        }
        assert_eq!(s.state, SwitchState::Held);

        s.update(18); // pos = 18, >= trig_upper(18) -> released
        assert_eq!(s.state, SwitchState::Unpressed);
    }

    #[test]
    fn update_hold_counter_resets_on_release() {
        let mut s = default_switch();

        s.update(20);
        for _ in 0..100 {
            s.update(20);
        }
        assert_eq!(s.hold_counter, 100);

        s.update(18);
        assert_eq!(s.state, SwitchState::Unpressed);
        assert_eq!(s.hold_counter, 0);

        // Second press starts counting from 0
        s.update(20);
        assert_eq!(s.hold_counter, 0);
        s.update(20);
        assert_eq!(s.hold_counter, 1);
    }

    // ── Switch::update_rapid() ───────────────────────────────────

    #[test]
    fn update_rapid_full_cycle() {
        let mut s = rapid_switch();

        // Normal press
        s.update(20);
        assert_eq!(s.state, SwitchState::RapidPressed);

        // Rapid press
        s.update(24);
        assert_eq!(s.state, SwitchState::RapidPressed);

        // Rapid release
        s.update(20);
        assert_eq!(s.state, SwitchState::Unpressed);

        // Rapid re-press
        s.update(26);
        assert_eq!(s.state, SwitchState::RapidPressed);

        // Rapid release again
        s.update(22);
        assert_eq!(s.state, SwitchState::RapidUnpressed);
    }

    #[test]
    fn update_rapid_release_above_treshold() {
        let mut s = rapid_switch();

        s.update(20);
        assert_eq!(s.state, SwitchState::RapidPressed);
        s.update(18);
        assert_eq!(s.rapid_last_position, 20);

        s.update(16);
        assert_eq!(s.state, SwitchState::Unpressed);
    }

    #[test]
    fn update_rapid_gradual_release() {
        let mut s = rapid_switch();

        s.update(26);
        assert_eq!(s.state, SwitchState::RapidPressed);
        s.update(25);
        s.update(24);
        s.update(23);
        s.update(22);
        assert_eq!(s.rapid_last_position, 22);
        assert_eq!(s.state, SwitchState::RapidUnpressed);
    }

    #[test]
    fn update_rapid_gradual_release_above_thresh() {
        let mut s = rapid_switch();

        s.update(20);
        assert_eq!(s.state, SwitchState::RapidPressed);
        s.update(19);
        s.update(18);
        s.update(17);
        s.update(16);
        s.update(15);
        assert_eq!(s.rapid_last_position, 16);
        assert_eq!(s.state, SwitchState::Unpressed);
    }

    #[test]
    fn update_rapid_gradual_press() {
        let mut s = rapid_switch();

        s.update(16);
        assert_eq!(s.state, SwitchState::Unpressed);
        s.update(17);
        s.update(18);
        s.update(19);
        s.update(20);
        assert_eq!(s.rapid_last_position, 20);
        assert_eq!(s.state, SwitchState::RapidPressed);
    }

    #[test]
    fn update_rapid_only_after_normal_press() {
        let mut s = rapid_switch();

        assert_eq!(s.state, SwitchState::Unpressed);
        s.update(14);
        assert_eq!(s.state, SwitchState::Unpressed);
        s.update(15);
        s.update(16);
        s.update(17);
        assert_eq!(s.rapid_last_position, 0);
        assert_eq!(s.state, SwitchState::Unpressed);

        s.update(20);
        assert_eq!(s.state, SwitchState::RapidPressed);
    }

    // ── update_rapid arithmetic edge cases ───────────────────────

    #[test]
    fn update_rapid_saturates_on_underflow() {
        let mut s = rapid_switch();
        s.state = SwitchState::RapidPressed;
        s.rapid_last_position = 2; // less than rapid_lower (4)
        s.position = 1;

        // Should not panic — saturating_sub clamps to 0
        // 1 <= 0 is false, so no spurious rapid press
        s.update_rapid();
        assert_eq!(s.state, SwitchState::RapidPressed);
    }

    #[test]
    fn update_rapid_saturates_on_overflow() {
        let mut s = rapid_switch();
        s.state = SwitchState::RapidPressed;
        s.rapid_last_position = 253; // 253 + 4 = 257 would overflow
        s.position = 254;

        // Should not panic — saturating_add clamps to 255
        // 254 >= 255 is false, so no spurious rapid release
        s.update_rapid();
        assert_eq!(s.state, SwitchState::RapidPressed);
    }

    // ── State transitions through update ─────────────────────────

    #[test]
    fn state_transitions_press_release() {
        let mut s = default_switch();
        assert_eq!(s.state, SwitchState::Unpressed);

        s.update(20);
        assert_eq!(s.state, SwitchState::Pressed);

        s.update(18);
        assert_eq!(s.state, SwitchState::Unpressed);
    }

    #[test]
    fn state_transitions_press_hold_release() {
        let mut s = default_switch();

        s.update(20);
        assert_eq!(s.state, SwitchState::Pressed);

        for _ in 0..=HOLD_TIME {
            s.update(20);
        }
        assert_eq!(s.state, SwitchState::Held);

        s.update(18);
        assert_eq!(s.state, SwitchState::Unpressed);
    }

    #[test]
    fn state_transitions_disabled_rapid_does_not_interfere_with_normal() {
        let mut s = default_switch();
        assert!(!s.rapid_enabled);

        s.update(20);
        assert_eq!(s.state, SwitchState::Pressed);

        s.update(25); // deeper, but rapid disabled
        assert_eq!(s.state, SwitchState::Pressed);
    }

    // ── value() ──────────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn value_underflows_when_distance_less_than_comp() {
        let mut s = default_switch();
        assert_eq!(s.value(248), 0); // distance(248) = 26, comp = 26 -> 0
        s.offset = 27;
        let _ = s.value(248); // 26 - 27 underflows
    }
}
