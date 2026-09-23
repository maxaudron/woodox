#![cfg_attr(not(test), no_std)]

pub mod layout;
pub mod matrix;

#[cfg(not(test))]
#[allow(unused)]
mod lg {
    pub use defmt::{debug, error, info, trace, warn, Format};
}

#[cfg(test)]
#[allow(unused)]
mod lg {
    pub use log::{debug, error, info, trace, warn};
}
