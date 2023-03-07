#![no_std]
#![no_main]

use rp2040_hal as hal;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use hal::{clocks, entry, pac, watchdog, Adc, Clock, Timer};

mod hall;

mod hardware;
mod matrix;

use crate::{
    hardware::ReadAdc,
    matrix::{ScanOrder, Switch},
};

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
    let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
    let sio = hal::Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = clocks::init_clocks_and_plls(
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

    #[allow(unused_variables)]
    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("core initialization finished");

    // Initialize MUX
    let mut mux = hardware::CD74HC4067::new(
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
        adc.cs.modify(|_, w| unsafe { w.rrobin().bits(0b01111) });

        adc.cs.modify(|_, w| unsafe { w.ainsel().bits(0) });
        adc.cs.modify(|_, w| w.en().set_bit());
        while !adc.cs.read().ready().bit_is_set() {
            cortex_m::asm::nop();
        }

        info!("adc initialization finished");

        adc
    };

    let switches = [
        Switch::new(0, 0),
        Switch::new(0, 1),
        Switch::new(0, 2),
        Switch::new(0, 3),
        Switch::new(0, 4),
        Switch::new(0, 5),
        Switch::new(0, 6),
        Switch::new(0, 7),
    ];

    let mut scan_order = ScanOrder::new(switches);

    #[cfg(feature = "timers")]
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    debug!("starting main loop");
    loop {
        #[cfg(feature = "timers")]
        let time = timer.get_counter();

        scan_order
            .scans
            .iter_mut()
            .enumerate()
            .for_each(|(i, scan)| {
                mux.set_output_active(i as u8);
                let r = adc.read_all();

                scan.switches[0].value(r.0);
                scan.switches[1].value(r.1);
                scan.switches[2].value(r.2);
                scan.switches[3].value(r.3);
            });

        #[cfg(feature = "timers")]
        {
            let time2 = timer.get_counter();
            debug!("time: {:?}", (time2 - time).to_nanos())
        }
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
