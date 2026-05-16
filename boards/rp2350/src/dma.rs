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

pub static GLOBAL_DMA: Mutex<RefCell<Option<dma::Channels>>> = Mutex::new(RefCell::new(None));

pub trait InitializeDMA {
    fn initialize(self, buf: u32, adc: u32);
}

impl InitializeDMA for Channels {
    fn initialize(mut self, buf: u32, adc: u32) {
        self.ch0.listen_irq0();

        // -----------------------------
        // Configure DMA Channel 0
        let dma0 = self.ch0.ch();
        dma0.ch_al3_ctrl.modify(|_, w| w.en().set_bit());
        dma0.ch_al3_ctrl.modify(|_, w| w.ring_sel().set_bit());
        dma0.ch_al3_ctrl
            .modify(|_, w| unsafe { w.ring_size().bits(DMA_BUF_SIZE as u8) });
        dma0.ch_al3_ctrl.modify(|_, w| w.data_size().size_byte());
        dma0.ch_al3_ctrl.modify(|_, w| w.treq_sel().adc());
        dma0.ch_al3_ctrl.modify(|_, w| w.incr_write().set_bit());
        dma0.ch_al3_trans_count
            .write(|w| unsafe { w.bits(4) });
        dma0.ch_al3_write_addr.write(|w| unsafe { w.bits(buf) });
        dma0.ch_al3_read_addr_trig
            .write(|w| unsafe { w.bits(adc) });

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

        critical_section::with(|cs| {
            GLOBAL_DMA.borrow(cs).replace(Some(self));
        });

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::DMA_IRQ_0);
        }
    }
}

#[interrupt]
fn DMA_IRQ_0() {
    static mut DMA: Option<dma::Channels> = None;

    debug!("dma triggered interrupt");

    if DMA.is_none() {
        critical_section::with(|cs| {
            *DMA = GLOBAL_DMA.borrow(cs).take();
        });
    }

    if let Some(dma_channels) = DMA {
        let dma0 = &mut dma_channels.ch0;

        if dma0.check_irq0() {
            dma0.ch()
                .ch_al1_trans_count_trig
                .write(|w| unsafe { w.bits(4) });
        }
    }
}
