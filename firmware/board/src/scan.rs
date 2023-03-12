use core::fmt::Debug;
use defmt::info;

use embedded_hal::digital::v2::OutputPin;
use woodox_lib::{layout::default_switches, matrix::ScanOrder};

use crate::{
    hal::pac::ADC,
    hardware::{self, ReadAdc},
};

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

    info!("starting scan loop");
    loop {
        scan_order
            .scans
            .iter_mut()
            .enumerate()
            .for_each(|(i, scan)| {
                mux.set_output_active(i as u8);
                let r = adc.read_all();
                scan.update(r)
            });
    }
}
