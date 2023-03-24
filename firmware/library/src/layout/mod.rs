//! Module containing different layouts and exposing them generically

macro_rules! switches {
    {$($x:expr,$y:expr),+$(,)+} => {
        use crate::matrix::Switch;

        pub fn default_switches() -> [Switch; NUM_SWITCHES] {
            [ $(Switch::new($x,$y)),+ ]
        }
    };
}

macro_rules! keymap {
    {$($n:literal=$layer:expr;)+} => {
        use usbd_human_interface_device::page::Keyboard;
        use crate::matrix::{Keymap, Key};

        pub const fn default_keymap() -> Keymap {
            let mut keymap = Keymap::default();

            $(keymap.layers[$n] = $layer;)+

            keymap
        }
    };
}

macro_rules! layer {
    [$(
        $k:tt
            $((
                $key:tt$(, $code:tt)*
            ))*
    ),+$(,)+
    ] => {
        [ $( key!($k$(($key$(, $code)*))*) ),+ ]
    };
}

macro_rules! key {
    (Key($key:tt)) => {
        Key::Keycode(Keyboard::$key)
    };
    ($k:tt($layer:literal, $key:tt)) => {
        Key::$k($layer, Keyboard::$key)
    };
    ($k:tt($key:tt)) => {
        Key::$k($key)
    };
    ($k:tt) => {
        Key::$k
    };
}

pub(crate) use {keymap, layer, switches};

#[cfg(feature = "macropad")]
mod macropad;
#[cfg(feature = "macropad")]
pub use macropad::*;
