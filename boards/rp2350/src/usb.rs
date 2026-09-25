use defmt::{Format, debug, info};

use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_human_interface_device::prelude::*;

use usbd_human_interface_device::device::DeviceClass;
use usbd_human_interface_device::device::keyboard::{NKROBootKeyboard, NKROBootKeyboardConfig};

use frunk_core::hlist::{HCons, HNil};

use woodox_lib::matrix::KeyboardState;

use crate::uart;
use crate::uart::Uart;
use crate::uart::UartRole;

pub struct Usb<U>
where
    U: UsbBus + Sized + 'static,
{
    hid: UsbHidClass<'static, U, HCons<NKROBootKeyboard<'static, U>, HNil>>,
    dev: UsbDevice<'static, U>,
    pub initialized: UsbState,
    init_ticks: usize,
}

#[derive(Debug, Clone, Format, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum UsbState {
    Initializing,
    Connected,
    Disconnected,
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

        Self {
            hid,
            dev,
            initialized: UsbState::Initializing,
            init_ticks: 0,
        }
    }

    pub fn state(&self) -> UsbDeviceState {
        self.dev.state()
    }

    pub fn tick(&mut self, keys: &mut KeyboardState, uart: &mut Uart) {
        match self.initialized {
            UsbState::Initializing => {
                if self.state() == UsbDeviceState::Configured {
                    info!("usb controller initialized");
                    self.initialized = UsbState::Connected;
                    uart.send(uart::Message::Init(UartRole::Primary));
                } else if self.init_ticks < 2000 {
                    self.init_ticks += 1;
                    if (self.init_ticks % 100) == 0 {
                        info!("usb controller not initialized yet: {:?}", self.state());
                    }
                } else {
                    info!("usb could not connect: {:?}", self.state());
                    self.initialized = UsbState::Disconnected;
                    uart.send(uart::Message::Init(UartRole::Secondary));
                    return;
                }
            }
            UsbState::Connected => (),
            UsbState::Disconnected => return,
        }

        match self.hid.device().write_report(keys.matrix) {
            Err(UsbHidError::WouldBlock) => {
                // info!("usb would block")
            }
            Err(UsbHidError::Duplicate) => {
                // info!("usb duplicate")
            }
            Ok(_) => {
                // info!("usb write ok")
            }
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

        keys.clear_oneshot();

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
