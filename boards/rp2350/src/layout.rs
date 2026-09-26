const BASE: usize = 0;
const SYMB: usize = 1;
const NAV: usize = 2;

pub mod left {
    use woodox_lib::{key, keymap, layer, switches};

    use crate::layout::SYMB;

    switches! {
        0;1, 0;0, 0;5, 0;7, 0;6, 0;4,
        3;1, 3;0, 3;5, 3;7, 3;6, 3;4,
        2;1, 2;0, 2;5, 2;7, 2;6, 2;4,
        1;1, 1;0, 1;5, 1;7, 1;6, 1;4, 0;3, 3;3,
           0;2,   3;2, 2;2,    1;2,   1;3, 2;3,
    }

    keymap! {
        0 = layer! [ // BASE
            GrvEsc,             Key(Keyboard1), Key(Keyboard2), Key(Keyboard3), Key(Keyboard4), Key(Keyboard5),
            Key(Tab),           Key(Q),         Key(W),         Key(R),         Key(T),         Key(Y),
            Key(LeftControl),   Key(A),         Key(S),         Key(D),         Key(F),         Key(G),
            Key(LeftShift),     Key(Z),         Key(X),         Key(C),         Key(V),         Key(B), Key(LeftAlt), Key(LeftGUI),
            LayerTap(1, L),         Key(LeftArrow),       Key(RightArrow),         Layer(SYMB),          Key(Space),   Key(DeleteBackspace),
        ];
        1 = layer! [ // SYMB
            Trns,   Key(F1),    Key(F2),    Key(F3),         Key(F4),           Key(F5),
            Trns,   Trns,       Trns,     Shift(LeftBrace),Shift(RightBrace), Trns,
            Trns,   Trns,       Trns,     Key(LeftBrace),  Key(RightBrace),   Trns,
            Trns,   Trns,       Trns,     Shift(Keyboard9),Shift(Keyboard0),  Trns,     Trns, Trns,
            Trns,          Trns,       Trns,                        Trns,               Trns,   Trns,
        ];
        1 = layer! [ // NAV
            Trns,   Key(F1),        Key(F2),        Key(F3),        Key(F4),        Key(F5),
            Trns,   Key(Home),      Key(UpArrow),   Key(End),       Key(PageUp),    Trns,
            Trns,   Key(LeftArrow), Key(DownArrow), Key(RightArrow),Key(PageDown),  Trns,
            Trns,   Trns,           Trns,           Trns,           Trns,           Trns,   Trns,   Trns,
            Trns,   Trns,       Trns,         Trns,                                         Trns,   Trns,
        ];
    }
}

pub mod right {
    use woodox_lib::{key, keymap, layer, switches};

    use crate::layout::NAV;

    switches! {
                  0;4, 0;6, 0;7, 0;5, 0;0, 0;1,
                  3;4, 3;6, 3;7, 3;5, 3;0, 3;1,
                  2;4, 2;6, 2;7, 2;5, 2;0, 2;1,
        3;3, 0;3, 1;4, 1;6, 1;7, 1;5, 1;0, 1;1,
        2;3, 1;3,    1;2,   2;2, 3;2,    0;2,
    }

    keymap! {
        0 = layer! [
                                                Key(Keyboard6), Key(Keyboard7), Key(Keyboard8), Key(Keyboard9), Key(Keyboard0),   Key(Minus),
                                                Key(Y),         Key(U),         Key(I),         Key(O),         Key(P),           Key(Backslash),
                                                Key(H),         Key(J),         Key(K),         Key(L),         Key(Semicolon),   Key(Apostrophe),
            Key(RightAlt),Key(RightControl),    Key(N),         Key(M),         Key(Comma),     Key(Dot),       Key(ForwardSlash),Key(RightShift),
            Key(DeleteForward),Key(ReturnEnter),       Layer(NAV),              Key(LeftArrow), Key(RightArrow),            Layer(NAV),
        ];
        1 = layer! [
                          Key(F6),              Key(F7),        Key(F8),      Key(F9),      Key(F10),           Key(F11),
                          Key(KeypadDivide),    Key(Keypad7),   Key(Keypad8), Key(Keypad9), Key(KeypadSubtract),Key(F12),
                          Key(KeypadMultiply),  Key(Keypad4),   Key(Keypad5), Key(Keypad6), Key(KeypadAdd),     Key(Apostrophe),
            Trns,   Trns, Key(KeypadEqualSign), Key(Keypad1),   Key(Keypad2), Key(Keypad3), Key(KeypadEnter),   Key(KeypadEqual),
            Trns,   Trns,          Trns,                 Key(Keypad0), Key(KeypadDot),             Layer(NAV),
        ];
        2 = layer! [
                          Key(F6),          Key(F7),        Key(F8),        Key(F9),         Key(F10),  Key(F11),
                          Trns,             Trns,           Trns,           Trns,            Trns,      Key(F12),
                          Key(LeftArrow),   Key(DownArrow), Key(UpArrow),   Key(RightArrow), Trns,      Trns,
            Trns,   Trns, Trns,             Trns,           Trns,           Trns,            Trns,      Trns,
            Trns,   Trns,          Trns,                    Trns,           Trns,                  Trns,
        ];
    }
}
