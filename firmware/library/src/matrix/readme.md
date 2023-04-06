# Scanning the switches and mapping them to actions

In comparison to a normal mechanical keyboard, hall effect keyboards do not
utilize a matrix. Instead I/O Multiplexers are used to attach the many switches
to few pins, and the arrangement of these muxes and pin assignments are subject
to a PCBs design and practicability of it's physical layout.

A consequence of this is that the order in which switches are scanned does not
correlate with the position of the switch on the keyboard in a clear manner.

On a simple 8-key matrix based keyboard we would e.g. start at the top right and
increment one by one

```text
|---|---|---|---|
| 1 | 2 | 3 | 4 |
|---|---|---|---|
| 5 | 6 | 7 | 8 |
|---|---|---|---|
```

The pin assignments on a keyboard using our might worst case be entirely random:

```text
|---|---|---|---|
| 3 | 7 | 2 | 6 |
|---|---|---|---|
| 8 | 4 | 1 | 4 |
|---|---|---|---|
```

To allow mapping these positions to logical keys we index all keys in an array
and store our logical keys in an array with the same order. The [`switches!`]
and [`keymap!`] macros allow defining these mappings in a more natural manner.

Internally this is represented using the [`ScanOrder`] and [`Scan`] structs
which build an efficient runtime order from a user representation of the switch
arrangement using the [`ScanOrder::new`] function.

[`switches!`]: crate::layout::switches!
[`keymap!`]: crate::layout::keymap!
