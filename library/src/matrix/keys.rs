//! Global state of the keymap an which keys are pressed.
//!
//! the keymap and keyboard are stored in a static mut and can only be accessed by
//! an unsafe function, to make the access a bit easier the Keymap struct provides
//! an implementation with common actions to take.
//!
//! Despite these methods being "safe" functions, you should still be mindful of how
//! and where you use them. The implementation i use only ever writes to them from
//! one thread, and reads from another. This way you might get through a half updated
//! list. Doing write actions from two cores at once can lead to an inconsistent state.
//!
//! This design was chosen as it does not incure any additional performance cost.

use core::ops::{Deref, DerefMut};

use crate::{layout::NUM_KEY_POSITIONS, lg::debug};
use usbd_human_interface_device::page::Keyboard;

use crate::{
    layout::NUM_SWITCHES,
    matrix::{ScanOrder, SwitchState},
};

/// A single key position in a [`Layer`]
#[derive(Debug, Copy, Clone)]
pub enum Key {
    /// Emit a regular USB HID Keycode like letters or numbers
    ///
    /// See [`usbd_human_interface_device::page::Keyboard`] for full reference
    Keycode(Keyboard),
    /// Emits a USB HID Keycode while also activating shift
    Shift(Keyboard),
    /// Use [`Layer`] `n` while held.
    ///
    /// This position should be a [`Key::Trns`] key on [`Layer`] `n`
    Layer(usize),
    /// [`Layer`] `n` while held, [`keycode`] when tapped.
    ///
    /// [`keycode`]: usbd_human_interface_device::page::Keyboard
    LayerTap(usize, Keyboard),
    /// [`keycode`] `x` while held, [`keycode`] `y` when tapped.
    ///
    /// [`keycode`]: usbd_human_interface_device::page::Keyboard
    KeyTap(Keyboard, Keyboard),
    /// Escape regularly, Grave (`) while shift is pressed
    GrvEsc,
    /// Transparent: Pass this key to the [`Layer`] below
    Trns,
    /// No key action
    Dead,
}

impl Key {
    const fn default() -> Key {
        Key::Dead
    }
}

/// A map of logical key actions to physical key positions
pub type Layer = [Key; NUM_SWITCHES];
pub const NUM_LAYERS: usize = 32;
pub const NUM_KEYCODES: usize = 231 * 2;

/// Represents the state of the keyboard report as send through
/// USB and maps the physical switch states to the logical keymap
pub struct KeyboardState {
    pub matrix: [Keyboard; NUM_KEY_POSITIONS * NUM_LAYERS],
    pub modifiers: [Modifiers; NUM_KEY_POSITIONS],
    pub keymap: Keymap,
}

pub enum KeyboardEvent {
    Keycode(u8, u8, Keyboard),
    SetModifier(u8, Keyboard),
    UnsetModifier(u8, Keyboard),
    Layer(u8),
}

impl KeyboardState {
    pub fn new(keymap: Keymap) -> KeyboardState {
        KeyboardState {
            matrix: [Keyboard::NoEventIndicated; NUM_KEY_POSITIONS * NUM_LAYERS],
            modifiers: [Modifiers(0); NUM_KEY_POSITIONS],
            keymap,
        }
    }

    pub fn update(&mut self, scan: &ScanOrder, mut hook: impl FnMut(KeyboardEvent)) {
        scan.scans
            .iter()
            .flatten()
            .for_each(|s| self.update_keys(s, &mut hook));
    }

    pub fn clear_oneshot(&mut self) {
        for key in self.matrix.iter_mut().take(NUM_KEYCODES).skip(NUM_KEYCODES / 2) {
            *key = Keyboard::NoEventIndicated;
        }
    }

    /// Set keycode or clear keycode
    pub fn set_keycode(
        &mut self,
        index: usize,
        layer: usize,
        state: SwitchState,
        keycode: Keyboard,
        hook: &mut impl FnMut(KeyboardEvent),
    ) {
        let k = &mut self.matrix[index * layer];
        if state.is_pressed() && *k != keycode {
            *k = keycode;
            debug!("key pressed: {}:{} {:?}", index, layer, keycode);
            hook(KeyboardEvent::Keycode(index as u8, layer as u8, keycode));
        } else if !state.is_pressed() && *k != Keyboard::NoEventIndicated {
            *k = Keyboard::NoEventIndicated;
            debug!("key released: {}:{} {:?}", index, layer, keycode);
            hook(KeyboardEvent::Keycode(
                index as u8,
                layer as u8,
                Keyboard::NoEventIndicated,
            ));
        }
    }

    /// Activate a keycode to be active exactly once in the next USB HID Report.
    pub fn set_oneshot_keycode(&mut self, index: usize, keycode: Keyboard) {
        self.matrix[index] = keycode;
        debug!("oneshot key pressed: {:#X}", keycode as u8)
    }

    pub fn set_modifier(
        &mut self,
        index: usize,
        state: SwitchState,
        keycode: Keyboard,
        hook: &mut impl FnMut(KeyboardEvent),
    ) {
        let m = &mut self.modifiers[index];
        if state.is_pressed() && !m.is_set(keycode) {
            debug!("set modifier {} for {}", keycode, index);
            hook(KeyboardEvent::SetModifier(index as u8, keycode));
            m.set(keycode);
        } else if !state.is_pressed() && m.is_set(keycode) {
            debug!("unset modifier {} for {}", keycode, index);
            hook(KeyboardEvent::UnsetModifier(index as u8, keycode));
            m.unset(keycode);
        }
    }

    // /// Key actions if they are held.
    // ///
    // /// [`Keymap::set_key()`] is also run on held keys, so only keys that
    // /// have different handling when held rather than tapped are required
    // /// to be set here
    // pub fn set_hold(&mut self, key: usize, state: bool) {
    //     debug!("hold: {:?} {:?}", key, state);
    //     for check_layer in (0..(self.keymap.active_layer + 1)).rev() {
    //         match self.keymap.layers[check_layer][key] {
    //             Key::LayerTap(layer, _) => self.keymap.set_layer(SwitchState::Held, layer),
    //             Key::KeyTap(keycode, _) => self.set_keycode(SwitchState::Held, keycode),
    //             _ => continue,
    //         }
    //     }
    // }

    fn update_keys(&mut self, s: super::Switch, hook: &mut impl FnMut(KeyboardEvent)) {
        for check_layer in (0..(self.keymap.active_layer + 1)).rev() {
            let index = s.index * (check_layer + 1);
            let layer = check_layer + 1;
            match self.keymap.layers[check_layer][s.index] {
                Key::Keycode(keycode) => {
                    self.set_keycode(index, layer, s.state, keycode, hook);
                    return;
                }
                Key::Shift(keycode) => {
                    self.set_modifier(index, s.state, Keyboard::LeftShift, hook);
                    self.set_keycode(index, layer, s.state, keycode, hook);
                    return;
                }
                Key::Layer(set_layer) => {
                    self.keymap.set_layer(s.state, set_layer, hook);
                    return;
                }
                Key::LayerTap(_, keycode) => {
                    if s.state.is_oneshot() {
                        self.set_oneshot_keycode(index, keycode);
                    }

                    return;
                }
                Key::KeyTap(_, keycode) => {
                    if s.state.is_oneshot() {
                        self.set_oneshot_keycode(index, keycode);
                    }

                    return;
                }
                Key::GrvEsc => {
                    if self.matrix.contains(&Keyboard::LeftShift)
                        || self.matrix.contains(&Keyboard::RightShift)
                    {
                        self.set_keycode(index, layer, s.state, Keyboard::Grave, hook);
                    } else {
                        self.set_keycode(index, layer, s.state, Keyboard::Escape, hook);
                    }

                    return;
                }
                Key::Trns => continue,
                Key::Dead => return,
            }
        }
    }

    #[allow(unused)]
    pub fn build_report(&self) -> [Keyboard; NUM_KEY_POSITIONS * NUM_LAYERS + 8] {
        let mut modifiers = Modifiers(0);
        for m in self.modifiers {
            modifiers.0 |= m.0
        }

        let mut report = [Keyboard::NoEventIndicated; NUM_KEY_POSITIONS * NUM_LAYERS + 8];

        report[0..8].copy_from_slice(&modifiers.to_report());
        report[8..].copy_from_slice(&self.matrix);

        report
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Modifiers(u16);

impl Modifiers {
    const OFFSET: u16 = 0xE0;

    pub fn set(&mut self, keycode: Keyboard) {
        match keycode {
            Keyboard::LeftControl
            | Keyboard::RightControl
            | Keyboard::LeftAlt
            | Keyboard::RightAlt
            | Keyboard::LeftShift
            | Keyboard::RightShift
            | Keyboard::LeftGUI
            | Keyboard::RightGUI => {
                let keycode = keycode as u16 - Self::OFFSET;
                self.0 |= 1 << keycode
            }
            _ => unimplemented!(),
        }
    }

    pub fn unset(&mut self, keycode: Keyboard) {
        match keycode {
            Keyboard::LeftControl
            | Keyboard::RightControl
            | Keyboard::LeftAlt
            | Keyboard::RightAlt
            | Keyboard::LeftShift
            | Keyboard::RightShift
            | Keyboard::LeftGUI
            | Keyboard::RightGUI => {
                let keycode = keycode as u16 - Self::OFFSET;
                self.0 &= !(1 << keycode)
            }
            _ => unimplemented!(),
        }
    }

    pub fn is_set(&self, keycode: Keyboard) -> bool {
        let keycode = keycode as u16 - Self::OFFSET;
        (self.0 & (1 << keycode)) >= 1
    }

    pub fn to_report(&self) -> [Keyboard; 8] {
        let mut report = [Keyboard::NoEventIndicated; 8];

        if self.is_set(Keyboard::LeftShift) {
            report[0] = Keyboard::LeftShift;
        }
        if self.is_set(Keyboard::LeftControl) {
            report[1] = Keyboard::LeftControl;
        }
        if self.is_set(Keyboard::LeftAlt) {
            report[2] = Keyboard::LeftAlt;
        }
        if self.is_set(Keyboard::LeftGUI) {
            report[3] = Keyboard::LeftGUI;
        }
        if self.is_set(Keyboard::RightShift) {
            report[4] = Keyboard::RightShift;
        }
        if self.is_set(Keyboard::RightControl) {
            report[5] = Keyboard::RightControl;
        }
        if self.is_set(Keyboard::RightAlt) {
            report[6] = Keyboard::RightAlt;
        }
        if self.is_set(Keyboard::RightGUI) {
            report[7] = Keyboard::RightGUI;
        }

        report
    }
}

impl DerefMut for Modifiers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Modifiers {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Keymap {
    pub layers: [Layer; NUM_LAYERS],
    pub active_layer: usize,
}

impl Keymap {
    pub const fn default() -> Keymap {
        Keymap {
            layers: [[Key::default(); NUM_SWITCHES]; NUM_LAYERS],
            active_layer: 0,
        }
    }

    /// Set `layer` active or inactive based on `state`
    pub fn set_layer(&mut self, state: SwitchState, layer: usize, hook: &mut impl FnMut(KeyboardEvent)) {
        if state.is_pressed() && self.active_layer != layer {
            self.active_layer = layer;
            debug!("layer activated: {:?}", layer);
            hook(KeyboardEvent::Layer(layer as u8))
        } else if !state.is_pressed() && self.active_layer == layer {
            self.active_layer = 0;
            debug!("layer deactivated: {:?}", layer);
            hook(KeyboardEvent::Layer(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_keycode_key_held() {
        let mut state = KeyboardState::new(Keymap::default());
        let kcn = Keyboard::NoEventIndicated;
        let kc = Keyboard::A;
        let kcu = 0;

        assert_eq!(state.matrix[kcu], kcn);

        state.set_keycode(kcu, 0, SwitchState::Pressed, kc, &mut |_| {});
        assert_eq!(state.matrix[kcu], kc);

        state.set_keycode(kcu, 0, SwitchState::Unpressed, kc, &mut |_| {});
        assert_eq!(state.matrix[kcu], kcn);
    }

    #[test]
    fn set_modifier() {
        let mut modifier = Modifiers(0);

        modifier.set(Keyboard::LeftShift);
        assert_eq!(modifier.0, 0b0010);

        modifier.set(Keyboard::LeftGUI);
        assert_eq!(modifier.0, 0b1010);

        modifier.set(Keyboard::RightGUI);
        assert_eq!(modifier.0, 0b10001010);

        modifier.unset(Keyboard::LeftShift);
        assert_eq!(modifier.0, 0b10001000);

        assert!(modifier.is_set(Keyboard::LeftGUI));
        assert!(!modifier.is_set(Keyboard::LeftShift));
    }
}
