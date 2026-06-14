use defmt::{debug, error, info, trace, warn};

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
            buf: Some(singleton!(: [u8; BUFFER] = [255; BUFFER]).unwrap()),
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
            debug!("dma ch irq {}", irq);

            let res = buf
                .iter()
                .enumerate()
                .fold([0; NUM_MUX], |mut acc, (i, &s)| {
                    acc[i % NUM_MUX] += s as u16;
                    acc
                })
                .map(|s| (s / SAMPLES as u16) as u8);

            // Update the switch state and runs switch.update() for each switch
            defmt::debug!("{}: {}", self.channel, res);
            self.scan.scans[self.channel as usize].update(res);
            trace!("completed update for channel: {}", self.channel);

            self.dma = Some(ch);
            self.buf = Some(buf);

            self.channel += 1;
        } else {
            warn!("dma completed but no transfer was found");
        }

        if self.channel < (NUM_CHANNELS as u8) {
            self.scan_one();
        } else {
            info!(
                "scan round took: {}μs",
                (self.timer.get_counter() - self.counter).to_micros()
            );
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

        // Reset the starting channel of the ADC so we start back at scanning the first mux
        // without this the position of each mux in the buffer will drift on each subsequent scan
        // because we cannot stop the fifo fast enough for it to not advance again.
        unsafe {
            let adc = crate::hal::pac::ADC::ptr().as_ref_unchecked();
            adc.cs().modify(|_, w| w.ainsel().bits(4)); // channel for mux1_com (GPIO 44)
        }
        self.fifo.resume();
    }
}
