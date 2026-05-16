#![no_std]
#![no_main]

use rp235x_hal as hal;
use hal::multicore::Multicore;
use hal::multicore::Stack;

use usb_device::class_prelude::*;

use defmt::*;
use defmt_rtt as _;

#[cfg(all(target_arch = "arm", target_os = "none"))]
use panic_probe as _;

use hal::{Adc, clocks, entry, pac, watchdog};

mod hardware;
mod scan;
mod usb;

use woodox_lib::layout;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

pub const DMA_BUF_SIZE: usize = 4;

static CORE1_STACK: Stack<4096> = Stack::new();

#[entry]
fn main() -> ! {
    info!("program start");

    // ------------------------------------
    // Setup core hardware
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
    let mut sio = hal::Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let clocks = clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let test = "whatever";
    let timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("core initialization finished");

    // ------------------------------------
    // Setup Mux and ADC for switch scanning

    // Initialize MUX
    let mux = hardware::CD74HC4067::new(
        pins.mux1_s0.into_push_pull_output(),
        pins.mux1_s1.into_push_pull_output(),
        pins.mux1_s2.into_push_pull_output(),
        pins.mux1_s3.into_push_pull_output(),
    );

    // Initialize ADC
    let adc = {
        let adc = Adc::new(pac.ADC, &mut pac.RESETS);
        let _adc_pin_0 = pins.mux1_com.into_floating_input();
        let _adc_pin_1 = pins.mux2_com.into_floating_input();
        let _adc_pin_2 = pins.mux3_com.into_floating_input();
        let _adc_pin_3 = pins.mux4_com.into_floating_input();

        let adc = adc.free();
        adc.cs().modify(|_, w| unsafe { w.rrobin().bits(0b01111) });

        adc.cs().modify(|_, w| unsafe { w.ainsel().bits(0) });
        adc.cs().modify(|_, w| w.en().set_bit());
        while !adc.cs().read().ready().bit_is_set() {
            cortex_m::asm::nop();
        }

        info!("adc initialization finished");

        adc
    };

    // ------------------------------------
    // Setup USB BUS
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USB,
        pac.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // ------------------------------------
    // Start switch scan loop on core 1
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _scan = core1.spawn(CORE1_STACK.take().unwrap(), || scan::scan(adc, mux));

    // ------------------------------------
    // Start usb controlelr loop on core 0
    usb::usb::<hal::usb::UsbBus>(timer, usb_bus)
}

hal::bsp_pins!(
    Gpio0 { name: pull_up },
    Gpio1 { name: mux1_s3 },
    Gpio2 { name: mux1_s2 },
    Gpio3 { name: mux1_s1 },
    Gpio4 { name: mux1_s0 },
    Gpio5 { name: led },
    Gpio26 { name: mux1_com },
    Gpio27 { name: mux2_com },
    Gpio28 { name: mux3_com },
    Gpio29 { name: mux4_com },
    // Gpio9 { name: d_cs },
    Gpio10 { name: d_dc },
    Gpio11 { name: d_rst },
    Gpio12 { name: d_sda },
    Gpio13 { name: d_scl },
);

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Hall Effect Firmware"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
