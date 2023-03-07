use rp2040_hal::pac::ADC;

pub trait ReadAdc {
    fn read(&self, pin: u8) -> u8;
    fn read_all(&self) -> (u8, u8, u8, u8);
}

impl ReadAdc for ADC {
    fn read(&self, channel: u8) -> u8 {
        self.cs
            .modify(|_, w| unsafe { w.ainsel().bits(channel).start_once().set_bit() });

        while !self.cs.read().ready().bit_is_set() {
            cortex_m::asm::nop();
        }

        (self.result.read().result().bits() >> 4) as u8
    }

    fn read_all(&self) -> (u8, u8, u8, u8) {
        (self.read(0), self.read(1), self.read(2), self.read(3))
    }
}
