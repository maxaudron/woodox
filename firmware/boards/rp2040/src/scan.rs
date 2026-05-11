use core::fmt::Debug;
use defmt::info;

use embedded_hal::digital::v2::OutputPin;
use woodox_lib::{layout::default_switches, matrix::ScanOrder};

use crate::{
    hal::pac::ADC,
    hardware::{self, ReadAdc},
};

#[cfg(feature = "timers")]
use crate::hal::pac::TIMER;
#[cfg(feature = "timers")]
use defmt::debug;

pub fn scan<A, B, C, D>(adc: ADC, mut mux: hardware::CD74HC4067<A, B, C, D>) -> !
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    <A as OutputPin>::Error: Debug,
    <B as OutputPin>::Error: Debug,
    <C as OutputPin>::Error: Debug,
    <D as OutputPin>::Error: Debug,
{
    let mut scan_order = ScanOrder::new(default_switches());

    #[cfg(feature = "timers")]
    let timer = TIMER::ptr();

    info!("starting scan loop");
    loop {
        #[cfg(feature = "timers")]
        let time1 = unsafe { get_counter(&(*timer)) };

        scan_order
            .scans
            .iter_mut()
            .enumerate()
            .for_each(|(i, scan)| {
                mux.set_output_active(i as u8);
                let r = adc.read_all();
                scan.update(r)
            });

        #[cfg(feature = "timers")]
        let time2 = unsafe { get_counter(&(*timer)) };
        #[cfg(feature = "timers")]
        debug!("scan time: {}µs", (time2 - time1));
    }
}

#[cfg(feature = "timers")]
fn get_counter(timer: &crate::pac::timer::RegisterBlock) -> u64 {
    let mut hi0 = timer.timerawh.read().bits();
    let timestamp = loop {
        let low = timer.timerawl.read().bits();
        let hi1 = timer.timerawh.read().bits();
        if hi0 == hi1 {
            break (u64::from(hi0) << 32) | u64::from(low);
        }
        hi0 = hi1;
    };

    timestamp
}
