#![no_std]
#![no_main]

use rp2040_hal as hal;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    dma::{DMAExt, SingleChannel},
    entry, pac,
    watchdog::Watchdog,
    Adc, Sio,
};

mod hall;
mod multiplexer;

mod adc;
mod dma;

use crate::{adc::InitializeADC, dma::InitializeDMA};

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

pub const DMA_BUF_SIZE: usize = 4;

#[entry]
fn main() -> ! {
    info!("program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("core initialization finished");

    // Initialize MUX
    let mut mux = multiplexer::CD74HC4067::new(
        pins.mux1_s0.into_push_pull_output(),
        pins.mux1_s1.into_push_pull_output(),
        pins.mux1_s2.into_push_pull_output(),
        pins.mux1_s3.into_push_pull_output(),
    );

    // critical_section::with(|cs| {
    //     dma::GLOBAL_MUX.borrow(cs).replace(Some(mux));
    // });

    // mux.set_output_active(1);

    // Initialize ADC
    let adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let _adc_pin_0 = pins.mux1_com.into_floating_input();
    let _adc_pin_1 = pins.mux2_com.into_floating_input();
    let _adc_pin_2 = pins.mux3_com.into_floating_input();
    let _adc_pin_3 = pins.mux4_com.into_floating_input();
    let mut adc = adc.initialize();

    // Initialize DMA.
    // let dma = hal::dma::Channels::initialize(
    //     pac.DMA.split(&mut pac.RESETS),
    //     adc,
    // );

    if adc.fcs.read().over().bit_is_set() {
        error!("fifo overflow");
        adc.fcs.modify(|_, w| w.over().set_bit())
    }
    if adc.fcs.read().under().bit_is_set() {
        error!("fifo underflow");
        adc.fcs.modify(|_, w| w.under().set_bit())
    }

    debug!("starting main loop");

    delay.delay_ms(1);
    loop {
        mux.set_output_active(0);
        adc.cs.modify(|_, w| w.start_many().set_bit());
        while adc.fcs.read().level().bits() < 3 {
            cortex_m::asm::nop();
        }
        adc.cs.modify(|_, w| w.start_many().clear_bit());

        while adc.cs.read().ready().bit_is_clear() {
            cortex_m::asm::nop();
        }

        let r1 = adc.fifo.read().val().bits();
        let r2 = adc.fifo.read().val().bits();
        let r3 = adc.fifo.read().val().bits();
        let r4 = adc.fifo.read().val().bits();
        debug!("duty: {:?} {:?} {:?} {:?}", r1, r2, r3, r4,);

        debug!("mux input 1");
        mux.set_output_active(7);
        adc.cs.modify(|_, w| w.start_many().set_bit());
        while adc.fcs.read().level().bits() < 3 {
            cortex_m::asm::nop();
        }
        adc.cs.modify(|_, w| w.start_many().clear_bit());

        while adc.cs.read().ready().bit_is_clear() {
            cortex_m::asm::nop();
        }

        let r6 = adc.fifo.read().val().bits() as u8;
        let r7 = adc.fifo.read().val().bits() as u8;
        let r8 = adc.fifo.read().val().bits() as u8;
        let r9 = adc.fifo.read().val().bits() as u8;
        debug!("duty: {:?} {:?} {:?} {:?}", r6, r7, r8, r9);


        delay.delay_ms(1);
    }
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
