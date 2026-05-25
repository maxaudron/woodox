/// Number of multiplexers connected to ADC
pub const NUM_MUX: usize = 4;
/// Total number of switches
pub const NUM_SWITCHES: usize = 32;
/// Number of scans needed to scan all switches
///
/// This depends on the number of mux and number of switches.
/// If the board has 1 mux connected, we need to scan each switch.
/// If the board has 4 mux connected, we can scan 4 switches at the same time.
///
/// Thus this default is usually correct, this still needs to be copied
/// into every layout as there is no way to conditionally set a const at
/// compile time if it is unset.
pub const NUM_CHANNELS: usize = NUM_SWITCHES / NUM_MUX;

/// About 200ms for this board
pub const HOLD_TIME: u32 = 8000;

pub const SAMPLES: usize = 4;

switches! {
    0;1, 0;0, 0;5, 0;7, 0;6, 0;4,
    1;1, 1;0, 1;5, 1;7, 1;6, 1;4,
    2;1, 2;0, 2;5, 2;7, 2;6, 2;4,
    3;1, 3;0, 3;5, 3;7, 3;6, 3;4, 0;3, 1;3,
    0;2,      1;2, 2;2,    3;2,   3;3, 2;3,
}

keymap! {
    0 = layer! [
        GrvEsc,         Key(Keyboard1), Key(Keyboard2), Key(Keyboard3), Key(Keyboard4), Key(Keyboard5),
        Key(Tab),       Key(Q), Key(W), Key(R), Key(T), Key(Y),
        Key(LeftControl),  Key(A), Key(S), Key(D), Key(F), Key(G),
        Key(LeftShift), Key(Z), Key(X), Key(C), Key(V), Key(B), Key(B), Key(B),
        LayerTap(1, L),     Key(F), Key(G),     Key(LeftShift), Key(Space), Key(DeleteBackspace),
    ];
    1 = layer! [
        GrvEsc,         Key(Keyboard1), Key(Keyboard2), Key(Keyboard3), Key(Keyboard4), Key(Keyboard5),
        Key(Tab),       Key(Q), Key(W), Key(R), Key(T), Key(Y),
        Key(LeftControl),  Key(A), Key(S), Key(D), Key(F), Key(G),
        Key(LeftShift), Key(Z), Key(X), Key(C), Key(V), Key(B), Key(B), Key(B),
        LayerTap(1, L),     Key(F), Key(G),     Key(LeftShift), Key(Space), Key(DeleteBackspace),
    ];
}
