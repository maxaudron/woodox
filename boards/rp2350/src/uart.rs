use fugit::{HertzU32, RateExtU32};
use rp235x_hal::{
    gpio::{
        FunctionUart, Pin, PullDown,
        bank0::{Gpio4, Gpio5},
    },
    pac::{RESETS, UART1},
    uart::{DataBits, Enabled, StopBits, UartConfig, UartPeripheral},
};
use usb_device::device::UsbDeviceState;
use usbd_human_interface_device::page::Keyboard;

use defmt::{Format, debug, error, info};
use woodox_lib::matrix::KeyboardState;

pub struct Uart {
    pub uart: UartPeripheral<
        Enabled,
        UART1,
        (
            Pin<Gpio4, FunctionUart, PullDown>,
            Pin<Gpio5, FunctionUart, PullDown>,
        ),
    >,

    pub role: UartRole,

    pub initialized: bool,
}

impl Uart {
    pub fn new(
        pins: (
            Pin<Gpio4, FunctionUart, PullDown>,
            Pin<Gpio5, FunctionUart, PullDown>,
        ),
        resets: &mut RESETS,
        uart1: UART1,
        freq: HertzU32,
    ) -> Self {
        let mut uart = UartPeripheral::new(uart1, pins, resets)
            .enable(
                // UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
                UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
                freq,
            )
            .unwrap();

        uart.enable_rx_interrupt();
        uart.disable_tx_interrupt();

        Self {
            uart,
            role: UartRole::Setup,
            initialized: false,
        }
    }

    pub fn send(&mut self, msg: Message) {
        info!("sending over uart: {:?}", msg);
        let b = msg.write();
        debug!("sending over uart: {:?}", b);
        self.uart.write_full_blocking(&b)
    }

    pub fn read(&mut self) -> Option<Message> {
        let mut b: [u8; 4] = [0; 4];
        match self.uart.read_raw(&mut b) {
            Ok(n) => {
                debug!("received {} bytes over uart: {:x}", n, b);
                Some(Message::read(&b))
            }
            Err(err) => {
                error!("failed to read uart: WouldBlock");
                None
            }
        }
    }

    pub fn intr(&mut self, keys: &mut KeyboardState) {
        if let Some(msg) = self.read() {
            info!("received msg over uart: {:?}", msg);
            match msg {
                crate::uart::Message::Init(role) => match role {
                    UartRole::Setup => unimplemented!(),
                    UartRole::Primary => {
                        self.send(Message::InitAck(UartRole::Secondary));
                    }
                    UartRole::Secondary => {
                        self.send(Message::InitAck(UartRole::Primary));
                        if self.initialized {
                            self.send(Message::Init(self.role.clone()))
                        }
                    }
                },
                crate::uart::Message::InitAck(role) => match role {
                    UartRole::Setup => todo!(),
                    UartRole::Primary => {
                        self.role = UartRole::Secondary;
                        info!("finish uart initialization with role: {:?}", self.role);
                        self.initialized = true;
                    }
                    UartRole::Secondary => {
                        self.role = UartRole::Primary;
                        info!("finish uart initialization with role: {:?}", self.role);
                        self.initialized = true;
                    }
                },
                crate::uart::Message::Keycode((idx, layer, keycode)) => {
                    keys.matrix[idx as usize * layer as usize * 2] = keycode
                }
                crate::uart::Message::Layer(layer) => keys.keymap.active_layer = layer as usize,
            }
        }
    }
}

#[derive(Debug, Clone, Format, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum UartRole {
    /// Undecided and waiting for setup
    Setup = 0x0,
    /// Has the USB Connection
    Primary = 0x1,
    /// Sends it's buttons to the other half
    Secondary = 0x2,
}

/// Shorthand for (Layer, Switch Position, Keycode)
pub type KeyID = (u8, u8, Keyboard);

#[derive(Debug, Clone, Format)]
#[repr(u8)]
pub enum Message {
    Init(UartRole) = 0x0,
    InitAck(UartRole) = 0x1,

    /// Secondary -> Primary
    Keycode(KeyID) = 0x2,

    /// Primary -> Secondary
    Layer(u8) = 0x4,
}

const MSG_SIZE: usize = core::mem::size_of::<Message>(); // 4

impl Message {
    pub fn write(self) -> [u8; MSG_SIZE] {
        unsafe { core::mem::transmute_copy::<_, [u8; MSG_SIZE]>(&self) }
    }

    pub fn read(msg: &[u8; MSG_SIZE]) -> Self {
        unsafe { core::mem::transmute_copy::<_, Message>(msg) }
    }
}
