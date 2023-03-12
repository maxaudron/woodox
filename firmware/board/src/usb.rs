use defmt::{debug, info};
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_human_interface_device::prelude::*;

use embedded_hal::prelude::*;
use fugit::ExtU32;

use crate::hal::Timer;
use woodox_lib::matrix;

pub fn usb<U>(timer: Timer, usb_bus: UsbBusAllocator<U>) -> !
where
    U: UsbBus + Sized,
{
    let mut keyboard = UsbHidClassBuilder::new()
        .add_interface(
            usbd_human_interface_device::device::keyboard::NKROBootKeyboardInterface::default_config(),
        )
        .build(&usb_bus);

    //https://pid.codes
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .manufacturer("usbd-human-interface-device")
        .product("NKRO Keyboard")
        .serial_number("TEST")
        .max_packet_size_0(8)
        .build();

    let mut tick_count_down = timer.count_down();
    tick_count_down.start(1.millis());

    info!("starting usb loop");
    loop {
        if tick_count_down.wait().is_ok() {
            let keys = matrix::Keymap::keyboard();

            match keyboard.interface().write_report(keys) {
                Err(UsbHidError::WouldBlock) => {}
                Err(UsbHidError::Duplicate) => {}
                Ok(_) => {}
                Err(e) => {
                    core::panic!("Failed to write keyboard report: {:?}", e)
                }
            };

            match keyboard.interface().tick() {
                Err(UsbHidError::WouldBlock) => {}
                Ok(_) => {}
                Err(e) => {
                    core::panic!("Failed to process keyboard tick: {:?}", e)
                }
            };
        }

        if usb_dev.poll(&mut [&mut keyboard]) {
            match keyboard.interface().read_report() {
                Err(UsbError::WouldBlock) => {
                    //do nothing
                }
                Err(e) => {
                    core::panic!("Failed to read keyboard report: {:?}", e)
                }
                Ok(leds) => {
                    debug!(
                        "got leds: {} {} {} {} {}",
                        leds.num_lock, leds.caps_lock, leds.scroll_lock, leds.compose, leds.kana
                    )
                }
            }
        }
    }
}
