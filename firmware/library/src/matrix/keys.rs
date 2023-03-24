//! Global state of the keymap and which keys are pressed.
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

use crate::layout::{default_keymap, NUM_SWITCHES};

/// A single key position in a layer
#[derive(Debug, Copy, Clone)]
pub enum Key {
    Keycode(Keyboard),
    Layer(usize),
    LayerTap(usize, Keyboard),
    GrvEsc,
    Trns,
    Dead,
}

impl Key {
    const fn default() -> Key {
        Key::Dead
    }
}

/// Current state of the keyboard, by default all entries are `Keyboard::NoEventIndicated`
/// and these are switched as needed by [`Keymap::set_key()`]
///
/// Contains one field for each possible keycode in the [`Keyboard`] enum. This isn't very
/// memory efficient for us, but more performant than dynamic collections.
///
/// [`Keyboard`]: usbd_human_interface_device::page::Keyboard
static mut KEYBOARD: [Keyboard; NUM_KEYCODES] = [Keyboard::NoEventIndicated; NUM_KEYCODES];
pub const NUM_KEYCODES: usize = 231 * 2;

/// Mapping of switch locations to keys.
///
static mut KEYMAP: Keymap = default_keymap();

pub type Layer = [Key; NUM_SWITCHES];
pub const NUM_LAYERS: usize = 32;

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

    pub fn set_keycode(state: bool, keycode: Keyboard) {
        unsafe {
            if state {
                KEYBOARD[keycode as usize] = keycode;
                debug!("key pressed: {:#X}", keycode as u8)
            } else {
                KEYBOARD[keycode as usize] = Keyboard::NoEventIndicated;
                debug!("key released: {:#X}", keycode as u8)
            }
        }
    }

    pub fn set_oneshot_keycode(keycode: Keyboard) {
        unsafe {
            KEYBOARD[keycode as usize + 231] = keycode;
            debug!("oneshot key pressed: {:#X}", keycode as u8)
        }
    }

    pub fn set_layer(state: bool, layer: usize) {
        unsafe {
            if state {
                KEYMAP.active_layer = layer;
                debug!("layer activated: {:?}", layer)
            } else {
                KEYMAP.active_layer = 0;
                debug!("layer deactivated: {:?}", layer)
            }
        }
    }

    /// Update the key with the position `key` with a new `state`
    pub fn set_key(key: usize, state: bool, held: bool) {
        unsafe {
            for check_layer in (0..(KEYMAP.active_layer + 1)).rev() {
                debug!("layer: {:?}", check_layer);
                match &mut KEYMAP.layers[(check_layer)][key] {
                    Key::Keycode(keycode) => {
                        Keymap::set_keycode(state, *keycode);
                        return;
                    }
                    Key::Layer(layer) => {
                        Keymap::set_layer(state, *layer);
                        return;
                    }
                    Key::LayerTap(_, keycode) => {
                        if !state && !held {
                            Keymap::set_oneshot_keycode(*keycode);
                        }

                        return;
                    }
                    Key::GrvEsc => {
                        if KEYBOARD[Keyboard::LeftShift as usize] == Keyboard::LeftShift
                            || KEYBOARD[Keyboard::RightShift as usize] == Keyboard::RightShift
                        {
                            Keymap::set_keycode(state, Keyboard::Grave);
                        } else {
                            Keymap::set_keycode(state, Keyboard::Escape);
                        }

                        return;
                    }
                    Key::Trns => continue,
                    Key::Dead => return,
                }
            }
        }
    }

    pub fn set_hold(key: usize, state: bool) {
        debug!("hold: {:?} {:?}", key, state);
        unsafe {
            for check_layer in (0..(KEYMAP.active_layer + 1)).rev() {
                match &mut KEYMAP.layers[(check_layer)][key] {
                    Key::LayerTap(layer, _) => Keymap::set_layer(state, *layer),
                    _ => continue,
                }
            }
        }
    }

    /// Get the state of the whole keyboard to pass as report over USB
    pub fn keyboard() -> [Keyboard; NUM_KEYCODES] {
        unsafe { KEYBOARD }
    }

    pub fn clear_oneshot() {
        unsafe {
            for key in KEYBOARD.iter_mut().take(NUM_KEYCODES).skip(231) {
                *key = Keyboard::NoEventIndicated;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! set_key {
        ($key:path) => {
            KEYBOARD[$key as usize] = $key;
        };
    }

    macro_rules! clear_key {
        ($key:path) => {
            KEYBOARD[$key as usize] = Keyboard::NoEventIndicated;
        };
    }

    macro_rules! assert_key {
        ( $map:path, $key:ident $(, $(mod $mod:ident),*)?) => {
            unsafe {
                $($(set_key!(Keyboard::$mod);),*)?
                KEYMAP.layers[0][0] = $map;
                Keymap::set_key(0, true);
                assert!(KEYBOARD[Keyboard::$key as usize] == Keyboard::$key);
                Keymap::set_key(0, false);
                KEYMAP.layers[0][0] = Key::Dead;
                $($(clear_key!(Keyboard::$mod);),*)?
            }
        };
    }

    #[test]
    fn key_grave_escape() {
        assert_key!(Key::GrvEsc, Escape);
        assert_key!(Key::GrvEsc, Grave, mod LeftShift);
        assert_key!(Key::GrvEsc, Grave, mod RightShift);
    }
}
