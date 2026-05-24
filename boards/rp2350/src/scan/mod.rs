use defmt::info;

use embedded_hal::digital::OutputPin;
use woodox_lib::{layout::default_switches, matrix::ScanOrder};

use crate::{
    hal::{Adc, adc::AdcFifo, dma::Channels},
    hardware::mux::Mux,
};

#[cfg(feature = "timers")]
use crate::hal::pac::TIMER;
#[cfg(feature = "timers")]
use defmt::debug;

use woodox_lib::layout::*;

pub struct ScanState {
    mux: impl Mux,
    dma: Channels,
    adc: AdcFifo<'_, u16>,

    channel: u8,
    scan: ScanOrder,
}

impl ScanState {
    pub fn new(mux: impl Mux, dma: Channels, adc: AdcFifo<'_, u16>) -> Self {
        Self {
            mux,
            dma,
            adc,

            channel: 0,
            scan: ScanOrder::new(default_switches()),
        }
    }

    pub fn init(&mut self) {

    }
}


// fn timer_alarm_isr():
//     state.channel = 0
//     set_mux(0)
//     settle()
//     drain_fifo()
//     arm_dma(&state.buf, N*4)
//     start_adc()
//
// fn dma_completion_isr():
//     stop_adc()
//     average_and_update(state.channel, &state.buf)
//
//     state.channel += 1
//     if state.channel < 8:
//         set_mux(state.channel)
//         settle()
//         drain_fifo()
//         arm_dma(&state.buf, N*4)
//         start_adc()
//     // else: scan complete, nothing to do until next timer alarm

pub fn scan(adc: Adc, mut mux: impl Mux) -> ! {
    let mut scan_order = ScanOrder::new(default_switches());

    #[cfg(feature = "timers")]
    let timer = TIMER::ptr();

    info!("starting scan loop");
    loop {
        #[cfg(feature = "timers")]
        let time1 = unsafe { get_counter(&(*timer)) };

        scan_order.scans.iter_mut().enumerate().for_each(|(i, scan)| {
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
