//! Module containing different layouts and exposing them generically

#[macro_export]
macro_rules! switches {
    {$($mux:expr;$channel:expr),+$(,)+} => {
        use $crate::{matrix::Switch, layout::NUM_SWITCHES};

        pub const fn switches() -> [Switch; NUM_SWITCHES] {
            [ $(Switch::new($mux,$channel)),+ ]
        }
    };
}

#[macro_export]
macro_rules! keymap {
    {$($n:literal=$layer:expr;)+} => {
        use usbd_human_interface_device::page::Keyboard;
        use $crate::matrix::{Keymap, Key};

        pub const fn keymap() -> Keymap {
            let mut keymap = Keymap::default();

            $(keymap.layers[$n] = $layer;)+

            keymap
        }
    };
}

#[macro_export]
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

#[macro_export]
macro_rules! key {
    (Key($key:tt)) => {
        Key::Keycode(Keyboard::$key)
    };
    (Shift($key:tt)) => {
        Key::Shift(Keyboard::$key)
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

pub use {keymap, layer, switches};

#[cfg(feature = "macropad")]
mod macropad;
#[cfg(feature = "macropad")]
pub use macropad::*;

#[cfg(feature = "woodox")]
mod woodox;
#[cfg(feature = "woodox")]
pub use woodox::*;
