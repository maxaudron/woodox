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
pub const NUM_KEYCODES: usize = 231;

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

    /// Update the key with the position `key` with a new `state`
    pub fn set_key(key: usize, state: bool) {
        unsafe {
            match KEYMAP.layers[KEYMAP.active_layer][key] {
                Key::Keycode(action) => {
                    if state {
                        KEYBOARD[action as usize] = action;
                        debug!("key pressed: {:?}", action as u8)
                    } else {
                        KEYBOARD[action as usize] = Keyboard::NoEventIndicated;
                        debug!("key released: {:?}", action as u8)
                    }
                }
                Key::Layer(layer) => {
                    if state {
                        KEYMAP.active_layer = layer;
                        debug!("layer activated: {:?}", layer)
                    } else {
                        KEYMAP.active_layer = 0;
                        debug!("layer deactivated: {:?}", layer)
                    }
                }
                Key::Trns => {}
                Key::Dead => todo!(),
            }
        }
    }

    /// Get the state of the whole keyboard to pass as report over USB
    pub fn keyboard() -> [Keyboard; NUM_KEYCODES] {
        unsafe { KEYBOARD }
    }
}
