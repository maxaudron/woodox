pub mod left {
    use woodox_lib::{keymap, layer, switches, key};

    switches! {
        0;1, 0;0, 0;5, 0;7, 0;6, 0;4,
        3;1, 3;0, 3;5, 3;7, 3;6, 3;4,
        2;1, 2;0, 2;5, 2;7, 2;6, 2;4,
        1;1, 1;0, 1;5, 1;7, 1;6, 1;4, 0;3, 3;3,
        0;2,      3;2, 2;2,    1;2,   1;3, 2;3,
    }

    keymap! {
        0 = layer! [
            GrvEsc,             Key(Keyboard1), Key(Keyboard2), Key(Keyboard3), Key(Keyboard4), Key(Keyboard5),
            Key(Tab),           Key(Q),         Key(W),         Key(R),         Key(T),         Key(Y),
            Key(LeftControl),   Key(A),         Key(S),         Key(D),         Key(F),         Key(G),
            Key(LeftShift),     Key(Z),         Key(X),         Key(C),         Key(V),         Key(B), Key(B), Key(B),
            LayerTap(1, L),            Key(F),           Key(G),         Key(LeftShift),                Key(Space), Key(DeleteBackspace),
        ];
        1 = layer! [
            GrvEsc,         Key(Keyboard1), Key(Keyboard2), Key(Keyboard3), Key(Keyboard4), Key(Keyboard5),
            Key(Tab),       Key(Q), Key(W), Key(R), Key(T), Key(Y),
            Key(LeftControl),  Key(A), Key(S), Key(D), Key(F), Key(G),
            Key(LeftShift), Key(Z), Key(X), Key(C), Key(V), Key(B), Key(B), Key(B),
            LayerTap(1, L),     Key(F), Key(G),     Key(LeftShift), Key(Space), Key(DeleteBackspace),
        ];
    }
}

pub mod right {
    use woodox_lib::{keymap, layer, switches, key};

    switches! {
        0;1, 0;0, 0;5, 0;7, 0;6, 0;4,
        3;1, 3;0, 3;5, 3;7, 3;6, 3;4,
        2;1, 2;0, 2;5, 2;7, 2;6, 2;4,
        1;1, 1;0, 1;5, 1;7, 1;6, 1;4, 0;3, 3;3,
        0;2,      3;2, 2;2,    1;2,   1;3, 2;3,
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
}
