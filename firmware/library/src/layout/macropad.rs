use super::{keymap, layer, switches};

/// Number of multiplexers connected to ADC
pub const NUM_MUX: usize = 1;
/// Total number of switches
pub const NUM_SWITCHES: usize = 8;
/// Number of scans needed to scan all switches
///
/// This depends on the number of mux and number of switches.
/// If the board has 1 mux connected, we need to scan each switch.
/// If the board has 4 mux connected, we can scan 4 switches at the same time.
///
/// Thus this default is usually correct, this still needs to be copied
/// into every layout as there is no way to conditionally set a const at
/// compile time if it is unset.
pub const NUM_SCANS: usize = NUM_SWITCHES / NUM_MUX;

switches! {
    0,1, 0,3, 0,5, 0,7,
    0,0, 0,2, 0,4, 0,6,
}

keymap! {
    0 = layer! [
          Key(A), Key(B), Key(C), Key(D),
        Layer(1), Key(F), Key(G), Key(H),
    ];
    1 = layer! [
        Key(Z), Key(Z), Key(Z), Key(Z),
          Trns, Key(Z), Key(Z), Key(Z),
    ];
}
