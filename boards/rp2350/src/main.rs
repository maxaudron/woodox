#![no_std]
#![no_main]

use hal::multicore::Multicore;
use hal::multicore::Stack;
use rp235x_hal::dma::DMAExt;
use rp235x_hal as hal;

use usb_device::class_prelude::*;

use defmt::*;
use defmt_rtt as _;

#[cfg(all(target_arch = "arm", target_os = "none"))]
use panic_probe as _;

use hal::{Adc, adc::AdcPin, clocks, entry, pac, watchdog};

mod hardware;
mod scan;
mod usb;

use woodox_lib::layout;

use crate::scan::ScanState;

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

    let pins = Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    info!("core initialization finished");

    // ------------------------------------
    // Setup Mux and ADC for switch scanning

    // Initialize MUX
    let mux = hardware::mux::CD74HC4051::new(
        pins.mux1_s0.into_push_pull_output(),
        pins.mux1_s1.into_push_pull_output(),
        pins.mux1_s2.into_push_pull_output(),
    );

    let dma = pac.DMA.split(&mut pac.RESETS);

    // Initialize ADC
    let adc = {
        let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
        let mut adc_pin_0 = AdcPin::new(pins.mux1_com.into_floating_input()).unwrap();
        let adc_pin_1 = AdcPin::new(pins.mux2_com.into_floating_input()).unwrap();
        let adc_pin_2 = AdcPin::new(pins.mux3_com.into_floating_input()).unwrap();
        let adc_pin_3 = AdcPin::new(pins.mux4_com.into_floating_input()).unwrap();

        let fifo = adc
            .build_fifo()
            .round_robin((&adc_pin_0, &adc_pin_1, &adc_pin_2, &adc_pin_3))
            .set_channel(&mut adc_pin_0)
            .enable_dma()
            .start_paused();

        info!("adc initialization finished");

        fifo
    };
    
    let scan = ScanState::new(mux, dma, adc);

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
    Gpio31 { name: mux1_s0 },
    Gpio30 { name: mux1_s1 },
    Gpio29 { name: mux1_s2 },
    Gpio33 { name: led },
    Gpio44 { name: mux1_com },
    Gpio43 { name: mux2_com },
    Gpio42 { name: mux3_com },
    Gpio41 { name: mux4_com },
    Gpio34 { name: d_cs },
    Gpio35 { name: d_dc },
    Gpio36 { name: d_rst },
    Gpio37 { name: d_sda },
    Gpio38 { name: d_scl },
    Gpio4 { name: i2c_sda },
    Gpio5 { name: i2c_sdl },
    Gpio6 { name: i2c_sda_acc },
    Gpio7 { name: i2c_sdl_acc },
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
