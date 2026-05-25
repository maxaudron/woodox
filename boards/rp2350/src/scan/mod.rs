use defmt::{debug, info};

use woodox_lib::{layout::default_switches, matrix::ScanOrder};

use crate::{
    hal::{
        adc::{AdcFifo, DmaReadTarget},
        dma::{
            CH0, Channel, SingleChannel,
            single_buffer::{self, Transfer},
        },
        singleton,
    },
    hardware::mux::{CD74HC4051, Mux, MuxPin1, MuxPin2, MuxPin3},
};

#[cfg(feature = "timers")]
use crate::hal::pac::TIMER;
#[cfg(feature = "timers")]
use defmt::debug;

use woodox_lib::layout::*;

const BUFFER: usize = NUM_MUX * SAMPLES;

pub struct ScanState<'a> {
    mux: CD74HC4051<MuxPin1, MuxPin2, MuxPin3>,
    dma: Option<Channel<CH0>>,
    fifo: AdcFifo<'a, u8>,

    channel: u8,
    scan: ScanOrder,
    buf: Option<&'static mut [u8; BUFFER]>,
    transfer: Option<Transfer<Channel<CH0>, DmaReadTarget<u8>, &'static mut [u8; BUFFER]>>,
}

impl<'a> ScanState<'a> {
    pub fn new(
        mux: CD74HC4051<MuxPin1, MuxPin2, MuxPin3>,
        mut dma: Channel<CH0>,
        fifo: AdcFifo<'a, u8>,
    ) -> Self {
        dma.enable_irq0();

        Self {
            mux,
            dma: Some(dma),
            fifo,

            channel: 0,
            buf: Some(singleton!(: [u8; BUFFER] = [0; BUFFER]).unwrap()),
            scan: ScanOrder::new(default_switches()),
            transfer: None,
        }
    }

    pub fn scan(&mut self) {
        info!("initiated new full matrix scan");
        self.channel = 0;
        self.dma_completion();
    }

    /// Triggered by DMA_IRQ_0 interrupt
    pub fn dma_completion(&mut self) {
        debug!("completed dma transfer for channel: {}", self.channel);
        self.fifo.pause();
        let transfer = self.transfer.take().unwrap();

        let (ch, _target, buf) = transfer.wait();

        let res = buf
            .iter()
            .enumerate()
            .fold([0; NUM_MUX], |mut acc, (i, &s)| {
                acc[i % NUM_MUX] += s;
                acc
            })
            .map(|s| s / NUM_MUX as u8);

        self.scan.scans[self.channel as usize].update(res);
        debug!("completed update for channel: {}", self.channel);

        self.dma = Some(ch);
        self.buf = Some(buf);

        if self.channel < NUM_CHANNELS as u8 {
            self.channel += 1;
            self.scan_one();
        }
    }

    /// Switch the active mux input and initiate the adc dma transfer
    fn scan_one(&mut self) {
        self.mux.set_output_active(self.channel);
        self.fifo.clear();

        let buf = self.buf.take().unwrap();
        let dma = self.dma.take().unwrap();
        self.transfer = Some(single_buffer::Config::new(dma, self.fifo.dma_read_target(), buf).start());
        self.fifo.resume();
        debug!("resumed adc dma transfer for channel: {}", self.channel);
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

// pub fn scan(adc: Adc, mut mux: impl Mux) -> ! {
//     let mut scan_order = ScanOrder::new(default_switches());
//
//     #[cfg(feature = "timers")]
//     let timer = TIMER::ptr();
//
//     info!("starting scan loop");
//     loop {
//         #[cfg(feature = "timers")]
//         let time1 = unsafe { get_counter(&(*timer)) };
//
//         scan_order.scans.iter_mut().enumerate().for_each(|(i, scan)| {
//             mux.set_output_active(i as u8);
//             let r = adc.read_all();
//             scan.update(r)
//         });
//
//         #[cfg(feature = "timers")]
//         let time2 = unsafe { get_counter(&(*timer)) };
//         #[cfg(feature = "timers")]
//         debug!("scan time: {}µs", (time2 - time1));
//     }
// }
//
// #[cfg(feature = "timers")]
// fn get_counter(timer: &crate::pac::timer::RegisterBlock) -> u64 {
//     let mut hi0 = timer.timerawh.read().bits();
//     let timestamp = loop {
//         let low = timer.timerawl.read().bits();
//         let hi1 = timer.timerawh.read().bits();
//         if hi0 == hi1 {
//             break (u64::from(hi0) << 32) | u64::from(low);
//         }
//         hi0 = hi1;
//     };
//
//     timestamp
// }
