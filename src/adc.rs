use defmt::debug;
use rp2040_hal::{Adc, pac::ADC};

pub trait InitializeADC {
    fn initialize(self) -> ADC;
}

impl InitializeADC for Adc {
    fn initialize(self) -> ADC {
        let adc = self.free();
        adc.cs.modify(|_, w| unsafe { w.rrobin().bits(0b01111) }); adc.fcs.modify(|_, w| w.en().set_bit());
        adc.fcs.modify(|_, w| w.shift().set_bit());
        // adc.fcs.modify(|_, w| w.dreq_en().set_bit());
        // adc.fcs.modify(|_, w| unsafe { w.thresh().bits(0x01) });
        // Wait for adc ready
        while !adc.cs.read().ready().bit_is_set() {
            cortex_m::asm::nop();
        }

        debug!("adc ready");
        adc.cs.modify(|_, w| unsafe { w.ainsel().bits(0) });
        adc.cs.modify(|_, w| w.start_many().clear_bit());
        adc.cs.modify(|_, w| w.start_once().clear_bit());

        let r1 = adc.fifo.read().val().bits() as u8;
        let r2 = adc.fifo.read().val().bits() as u8;
        let r3 = adc.fifo.read().val().bits() as u8;
        let r4 = adc.fifo.read().val().bits() as u8;
        debug!("initial fifo cleared: {:?}", (r1, r2, r3, r4));

        return adc;
    }
}
