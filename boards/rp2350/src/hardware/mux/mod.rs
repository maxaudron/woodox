#![allow(unused)]

mod cd74hc4051;
mod cd74hc4067;

pub use cd74hc4051::*;
pub use cd74hc4067::*;

pub trait Mux {
    /// Enable output `n`.
    /// If a SelectPinError occurs, the select is left in a possibly unwanted state, but it is disabled here.
    fn set_output_active(&mut self, output: u8);
}
