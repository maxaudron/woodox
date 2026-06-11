use defmt::{error, info, trace, warn};

use woodox_lib::{
    layout::default_switches,
    matrix::{KeyboardState, ScanOrder},
};

use crate::{
    hal::{
        Timer,
        adc::{AdcFifo, DmaReadTarget},
        dma::{
            CH0, Channel, SingleChannel,
            single_buffer::{self, Transfer},
        },
        singleton,
        timer::{CopyableTimer0, Instant},
    },
    hardware::mux::{CD74HC4051, Mux, MuxEnable, MuxPin1, MuxPin2, MuxPin3},
};

#[cfg(feature = "timers")]
use crate::hal::pac::TIMER;

use woodox_lib::layout::*;

const BUFFER: usize = NUM_MUX * SAMPLES;

pub struct ScanState<'a> {
    mux: CD74HC4051<MuxEnable, MuxPin1, MuxPin2, MuxPin3>,
    dma: Option<Channel<CH0>>,
    fifo: AdcFifo<'a, u8>,

    channel: u8,
    scan: ScanOrder,
    buf: Option<&'static mut [u8; BUFFER]>,
    transfer: Option<Transfer<Channel<CH0>, DmaReadTarget<u8>, &'static mut [u8; BUFFER]>>,

    timer: Timer<CopyableTimer0>,
    counter: Instant,
}

impl<'a> ScanState<'a> {
    pub fn new(
        mux: CD74HC4051<MuxEnable, MuxPin1, MuxPin2, MuxPin3>,
        mut dma: Channel<CH0>,
        fifo: AdcFifo<'a, u8>,
        timer: Timer<CopyableTimer0>,
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
            timer,

            counter: Instant::from_ticks(0),
        }
    }

    pub fn scan(&mut self) {
        if self.transfer.is_some() {
            error!("transfer already in progress. aborting!");
        } else {
            trace!("initiated new full matrix scan");
            self.channel = 0;
            self.counter = self.timer.get_counter();
            self.scan_one();
        }
    }

    /// Triggered by DMA_IRQ_0 interrupt
    pub fn dma_completion(&mut self, keys: &mut KeyboardState) {
        trace!("completed dma transfer for channel: {}", self.channel);
        self.fifo.pause();
        if let Some(transfer) = self.transfer.take() {
            let (mut ch, _target, buf) = transfer.wait();
            let irq = ch.check_irq0();

            let res = buf
                .iter()
                .enumerate()
                .fold([0; NUM_MUX], |mut acc, (i, &s)| {
                    acc[i % NUM_MUX] += s as u16;
                    acc
                })
                .map(|s| (s / SAMPLES as u16) as u8);

            // Update the switch state and runs switch.update() for each switch
            self.scan.scans[self.channel as usize].update(res);
            trace!("completed update for channel: {}", self.channel);

            self.dma = Some(ch);
            self.buf = Some(buf);

            self.channel += 1;
        } else {
            warn!("dma completed but no transfer was found");
        }

        if self.channel <= (NUM_CHANNELS as u8) - 1 {
            self.scan_one();
        } else {
            info!("scan round took: {}μs", (self.timer.get_counter() - self.counter).to_micros());
            trace!("stopping scan round on channel {}", self.channel);
            keys.update(&self.scan);
        }
    }

    /// Switch the active mux input and initiate the adc dma transfer
    fn scan_one(&mut self) {
        trace!("initiated scan for channel {}", self.channel);
        self.mux.set_output_active(self.channel);
        self.fifo.clear();

        let buf = self.buf.take().unwrap();
        let dma = self.dma.take().unwrap();
        self.transfer = Some(single_buffer::Config::new(dma, self.fifo.dma_read_target(), buf).start());
        self.fifo.resume();
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
//         trace!("scan time: {}µs", (time2 - time1));
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
