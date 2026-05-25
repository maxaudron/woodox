use defmt::{debug, info};

use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_human_interface_device::prelude::*;

use usbd_human_interface_device::device::DeviceClass;
use usbd_human_interface_device::device::keyboard::{NKROBootKeyboard, NKROBootKeyboardConfig};

use frunk_core::hlist::{HCons, HNil};

use woodox_lib::matrix;

pub struct Usb<U>
where
    U: UsbBus + Sized + 'static,
{
    hid: UsbHidClass<'static, U, HCons<NKROBootKeyboard<'static, U>, HNil>>,
    dev: UsbDevice<'static, U>,
}

impl<U> Usb<U>
where
    U: UsbBus + Sized + 'static,
{
    pub fn new(usb_bus: &'static UsbBusAllocator<U>) -> Self {
        let hid = UsbHidClassBuilder::new()
            .add_device(NKROBootKeyboardConfig::default())
            .build(usb_bus);

        let dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1209, 0x0001))
            .strings(&[StringDescriptors::default()
                .manufacturer("usbd-human-interface-device")
                .product("NKRO Keyboard")
                .serial_number("TEST")])
            .unwrap()
            .build();

        info!("usb initialized");

        Self { hid, dev }
    }

    pub fn tick(&mut self) {
        let keys = matrix::Keymap::keyboard();

        match self.hid.device().write_report(keys) {
            Err(UsbHidError::WouldBlock) => {}
            Err(UsbHidError::Duplicate) => {}
            Ok(_) => {}
            Err(e) => {
                core::panic!("Failed to write keyboard report: {:?}", e)
            }
        };

        match self.hid.device().tick() {
            Err(UsbHidError::WouldBlock) => {}
            Ok(_) => {}
            Err(e) => {
                core::panic!("Failed to process keyboard tick: {:?}", e)
            }
        };

        matrix::Keymap::clear_oneshot();

        if self.dev.poll(&mut [&mut self.hid]) {
            match self.hid.device().read_report() {
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
