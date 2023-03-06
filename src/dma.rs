use core::cell::RefCell;

use crate::hal::{
    dma::{self, Channels, SingleChannel},
    gpio::{
        bank0::{Gpio1, Gpio2, Gpio3, Gpio4},
        Output, Pin, PushPull,
    },
    pac::{self, interrupt, ADC},
};
use cortex_m::asm::delay;
use critical_section::Mutex;
use defmt::debug;

use crate::{multiplexer::CD74HC4067, DMA_BUF_SIZE};

pub static GLOBAL_DMA: Mutex<RefCell<Option<(dma::Channels, ADC)>>> =
    Mutex::new(RefCell::new(None));

pub type MUX = CD74HC4067<
    Pin<Gpio4, Output<PushPull>>,
    Pin<Gpio3, Output<PushPull>>,
    Pin<Gpio2, Output<PushPull>>,
    Pin<Gpio1, Output<PushPull>>,
>;
pub static GLOBAL_MUX: Mutex<RefCell<Option<MUX>>> = Mutex::new(RefCell::new(None));

pub trait InitializeDMA {
    fn initialize(self, adc: ADC);
}

impl InitializeDMA for Channels {
    fn initialize(mut self, adc: ADC) {
        self.ch0.listen_irq0();
        self.ch1.listen_irq0();

        // -----------------------------
        // Configure DMA Channel 0
        let dma0 = self.ch0.ch();
        dma0.ch_al3_ctrl.modify(|_, w| w.en().set_bit());
        dma0.ch_al3_ctrl.modify(|_, w| w.ring_sel().set_bit());
        dma0.ch_al3_ctrl
            .modify(|_, w| unsafe { w.ring_size().bits(8) });
        dma0.ch_al3_ctrl
            .modify(|_, w| unsafe { w.chain_to().bits(1) });
        dma0.ch_al3_ctrl.modify(|_, w| w.data_size().size_byte());
        dma0.ch_al3_ctrl.modify(|_, w| w.treq_sel().adc());
        dma0.ch_al3_ctrl.modify(|_, w| w.incr_write().set_bit());
        dma0.ch_al3_trans_count
            .write(|w| unsafe { w.bits(u32::MAX) });

        // dma0.ch_al3_write_addr
        //     .modify(|_, w| unsafe { w.bits(& as *const _ as u32) });

        // -----------------------------
        // Configure DMA Channel 1
        // let dma1 = self.ch1.ch();
        // dma1.ch_al3_ctrl.modify(|_, w| w.en().set_bit());
        // dma1.ch_al3_ctrl
        //     .modify(|_, w| unsafe { w.chain_to().bits(0) });
        // dma1.ch_al3_ctrl.modify(|_, w| w.data_size().size_byte());
        // dma1.ch_al3_ctrl.modify(|_, w| w.treq_sel().adc());
        // dma1.ch_al3_ctrl.modify(|_, w| w.incr_write().set_bit());
        // dma1.ch_al3_trans_count
        //     .write(|w| unsafe { w.bits(DMA_BUF_SIZE as u32) });

        // // dma1.ch_al3_write_addr
        // //     .modify(|_, w| unsafe { w.bits(&BUF1 as *const _ as u32) });
        // dma1.ch_al2_read_addr
        //     .modify(|_, w| unsafe { w.bits(&adc.fifo as *const _ as u32) });

        // -----------------------------
        // Start DMA Channel 0
        dma0.ch_al3_read_addr_trig
            .modify(|_, w| unsafe { w.bits(&adc.fifo as *const _ as u32) });

        // -----------------------------
        // Start ADC
        adc.cs.modify(|_, w| w.start_many().set_bit());

        critical_section::with(|cs| {
            GLOBAL_DMA.borrow(cs).replace(Some((self, adc)));
        });

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::DMA_IRQ_0);
        }
    }
}

// static mut BUF0: [u8; DMA_BUF_SIZE] = [0; DMA_BUF_SIZE];
// static mut BUF1: [u8; DMA_BUF_SIZE] = [0; DMA_BUF_SIZE];

#[interrupt]
fn DMA_IRQ_0() {
    static mut BUF: [[u8; DMA_BUF_SIZE]; 8] = [[0; DMA_BUF_SIZE]; 8];
    static mut DMA: Option<(dma::Channels, ADC)> = None;
    static mut mux: Option<MUX> = None;
    static mut POSITION: u8 = 6;

    debug!("triggered interupt");

    if DMA.is_none() {
        critical_section::with(|cs| {
            *DMA = GLOBAL_DMA.borrow(cs).take();
        });
    }

    if mux.is_none() {
        critical_section::with(|cs| {
            *mux = GLOBAL_MUX.borrow(cs).take();
        });
    }

    if let Some((dma_channels, adc)) = DMA {
        adc.cs.modify(|_, w| w.en().clear_bit());

        let dma0 = &mut dma_channels.ch0;
        let dma1 = &mut dma_channels.ch1;

        let next_position = if *POSITION >= 7 { 0 } else { *POSITION + 1 };

        if let Some(mux) = mux {
            mux.set_output_active(next_position);
            delay(10);

            let _ = adc.fifo.read().bits();
            let _ = adc.fifo.read().bits();
            let _ = adc.fifo.read().bits();
            let _ = adc.fifo.read().bits();

            adc.cs.modify(|_, w| w.en().set_bit());
        }

        if dma0.check_irq0() {
            debug!("dma0 irq");
            dma0.ch()
                .ch_al2_write_addr_trig
                .write(|w| unsafe { w.bits(&BUF[next_position as usize] as *const _ as u32) });
        }

        if dma1.check_irq0() {
            debug!("dma1 irq");
            dma1.ch()
                .ch_al3_write_addr
                .write(|w| unsafe { w.bits(&BUF[next_position as usize] as *const _ as u32) });
        }

        *POSITION = next_position;
        delay(30000000);

        debug!("position: {:?}, duty: {:?}", *POSITION, BUF);
    }
}
