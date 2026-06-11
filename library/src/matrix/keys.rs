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

use defmt::debug;
use usbd_human_interface_device::page::Keyboard;

use crate::{
    layout::{default_keymap, NUM_SWITCHES},
    matrix::{ScanOrder, SwitchState},
};

/// A single key position in a [`Layer`]
#[derive(Debug, Copy, Clone)]
pub enum Key {
    /// Emit a regular USB HID Keycode like letters or numbers
    ///
    /// See [`usbd_human_interface_device::page::Keyboard`] for full reference
    Keycode(Keyboard),
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
    pub matrix: [Keyboard; NUM_KEYCODES],
    pub keymap: Keymap,
}
impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardState {
    pub fn new() -> KeyboardState {
        KeyboardState {
            matrix: [Keyboard::NoEventIndicated; NUM_KEYCODES],
            keymap: default_keymap(),
        }
    }

    pub fn update(&mut self, scan: &ScanOrder) {
        scan.scans.iter().flatten().for_each(|s| self.update_switch(s));
    }

    pub fn clear_oneshot(&mut self) {
        for key in self.matrix.iter_mut().take(NUM_KEYCODES).skip(NUM_KEYCODES / 2) {
            *key = Keyboard::NoEventIndicated;
        }
    }

    /// Set keycode or clear keycode
    pub fn set_keycode(&mut self, state: SwitchState, keycode: Keyboard) {
        if state.is_pressed() {
            self.matrix[keycode as usize] = keycode;
            debug!("key pressed: {:#X}", keycode as u8)
        } else {
            self.matrix[keycode as usize] = Keyboard::NoEventIndicated;
            debug!("key released: {:#X}", keycode as u8)
        }
    }

    /// Activate a keycode to be active exactly once in the next USB HID Report.
    pub fn set_oneshot_keycode(&mut self, keycode: Keyboard) {
        self.matrix[keycode as usize + 231] = keycode;
        debug!("oneshot key pressed: {:#X}", keycode as u8)
    }

    /// Key actions if they are held.
    ///
    /// [`Keymap::set_key()`] is also run on held keys, so only keys that
    /// have different handling when held rather than tapped are required
    /// to be set here
    pub fn set_hold(&mut self, key: usize, state: bool) {
        debug!("hold: {:?} {:?}", key, state);
        for check_layer in (0..(self.keymap.active_layer + 1)).rev() {
            match self.keymap.layers[check_layer][key] {
                Key::LayerTap(layer, _) => self.keymap.set_layer(SwitchState::Held, layer),
                Key::KeyTap(keycode, _) => self.set_keycode(SwitchState::Held, keycode),
                _ => continue,
            }
        }
    }

    fn update_switch(&mut self, s: super::Switch) {
        for check_layer in (0..(self.keymap.active_layer + 1)).rev() {
            debug!("layer: {:?}", check_layer);
            match self.keymap.layers[check_layer][s.index] {
                Key::Keycode(keycode) => {
                    self.set_keycode(s.state, keycode);
                    return;
                }
                Key::Layer(layer) => {
                    self.keymap.set_layer(s.state, layer);
                    return;
                }
                Key::LayerTap(_, keycode) => {
                    if s.state.is_oneshot() {
                        self.set_oneshot_keycode(keycode);
                    }

                    return;
                }
                Key::KeyTap(_, keycode) => {
                    if s.state.is_oneshot() {
                        self.set_oneshot_keycode(keycode);
                    }

                    return;
                }
                Key::GrvEsc => {
                    if self.matrix[Keyboard::LeftShift as usize] == Keyboard::LeftShift
                        || self.matrix[Keyboard::RightShift as usize] == Keyboard::RightShift
                    {
                        self.set_keycode(SwitchState::Pressed, Keyboard::Grave);
                    } else {
                        self.set_keycode(SwitchState::Pressed, Keyboard::Escape);
                    }

                    return;
                }
                Key::Trns => continue,
                Key::Dead => return,
            }
        }
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
    pub fn set_layer(&mut self, state: SwitchState, layer: usize) {
        if state.is_pressed() {
            self.active_layer = layer;
            debug!("layer activated: {:?}", layer)
        } else {
            self.active_layer = 0;
            debug!("layer deactivated: {:?}", layer)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     macro_rules! set_key {
//         ($key:path) => {
//             KEYBOARD[$key as usize] = $key;
//         };
//     }
//
//     macro_rules! clear_key {
//         ($key:path) => {
//             KEYBOARD[$key as usize] = Keyboard::NoEventIndicated;
//         };
//     }
//
//     macro_rules! assert_key {
//         ( $map:path, $key:ident $(, $(mod $mod:ident),*)?) => {
//             unsafe {
//                 $($(set_key!(Keyboard::$mod);),*)?
//                 KEYMAP.layers[0][0] = $map;
//                 Keymap::set_key(0, true, false);
//                 assert!(KEYBOARD[Keyboard::$key as usize] == Keyboard::$key);
//                 Keymap::set_key(0, false, false);
//                 KEYMAP.layers[0][0] = Key::Dead;
//                 $($(clear_key!(Keyboard::$mod);),*)?
//             }
//         };
//     }
//
//     #[test]
//     fn key_grave_escape() {
//         assert_key!(Key::GrvEsc, Escape);
//         assert_key!(Key::GrvEsc, Grave, mod LeftShift);
//         assert_key!(Key::GrvEsc, Grave, mod RightShift);
//     }
// }
