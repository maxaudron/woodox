# Woodox Firmware

Embedded Rust firmware for an analog hall-effect keyboard, targeting the RP2040 (Cortex-M0+). Runs bare-metal with `no_std`.

## Architecture

The firmware runs on both RP2040 cores simultaneously. Core 1 continuously scans hall-effect switches through analog multiplexers and ADC. Core 0 runs the USB HID stack and sends keyboard reports to the host at 1ms intervals. Switch state is shared between cores via static global memory, avoiding lock overhead.

## Workspace

The project is split into two crates:

### `board`

Hardware-specific code that directly interfaces with RP2040 peripherals.

- **`main.rs`** -- Entry point. Initializes clocks, GPIO pins, ADC, multiplexers, and the USB bus. Spawns the scan loop on core 1 and the USB loop on core 0.
- **`scan.rs`** -- Scan loop running on core 1. Iterates through all multiplexer channels, reads ADC values for each switch, and updates switch state.
- **`usb.rs`** -- USB loop running on core 0. Reads the global keyboard state every 1ms and sends NKRO HID reports to the host.
- **`hardware/mux.rs`** -- Driver for the CD74HC4067 16-channel analog multiplexer. Selects channels by setting four GPIO pins to the binary representation of the channel number.
- **`hardware/adc.rs`** -- Trait and implementation for reading raw values from the RP2040 ADC. Supports reading a single channel or all multiplexer inputs in sequence.
- **`dma.rs`** -- Work-in-progress DMA configuration for ADC transfers. Not currently used in the main loop.
- **`led/mod.rs`** -- Commented-out WS2812 LED driver using PIO. Not currently active.

### `library` (`woodox-lib`)

Platform-independent logic. Can be tested on the host machine.

- **`matrix/switch.rs`** -- Core switch logic. Each switch tracks its position, press/release state, and hold timing. Supports configurable actuation and release points. Implements rapid trigger, which dynamically fires press/release events based on relative travel distance rather than fixed thresholds.
- **`matrix/hall.rs`** -- Lookup table that converts raw 8-bit ADC values to physical travel distance in 0.1mm increments, based on the hall-effect sensor response curve.
- **`matrix/keys.rs`** -- Global keymap and keyboard state. Defines key types: regular keycodes, layer switching, layer-tap (layer on hold / keycode on tap), key-tap (keycode on hold / different keycode on tap), and grave-escape. Resolves key actions by walking down the layer stack.
- **`matrix/mod.rs`** -- Scan and ScanOrder structs that map physical switch positions (mux ID + channel) to their logical index in the keymap.
- **`layout/mod.rs`** -- Macros (`switches!`, `keymap!`, `layer!`, `key!`) for declaratively defining keyboard layouts at compile time.
- **`layout/macropad.rs`** -- Default layout for an 8-key macropad with 1 multiplexer. Defines the physical switch wiring and a two-layer keymap.

## Hardware

- **MCU**: RP2040 (dual Cortex-M0+ at 125 MHz, 12 MHz external crystal)
- **Switches**: Analog hall-effect sensors read via ADC
- **Multiplexers**: CD74HC4067 (16-channel analog mux), selecting switches for ADC reading
- **USB**: Native RP2040 USB with NKRO keyboard HID
- **LEDs**: WS2812 via PIO (currently disabled)

## Features

- `macropad` (default) -- Builds with the 8-key macropad layout.
- `timers` -- Enables microsecond-precision scan loop timing via the hardware timer.

## Building

Requires the `thumbv6m-none-eabi` target:

```sh
rustup target add thumbv6m-none-eabi
cargo build --release -p woodox
```

## Flashing

Uses `probe-run` via cargo runner, or `cargo-embed` with the provided `Embed.toml`:

```sh
cargo embed --release -p woodox
```

Debugger protocol is SWD at 20 MHz. RTT logging is enabled with `defmt` output.
