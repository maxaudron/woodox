#![no_std]
#![no_main]

use rp235x_hal as hal;

#[cfg(all(target_arch = "arm", target_os = "none"))]
use panic_probe as _;
use defmt_rtt as _;

mod hardware;
mod scan;
mod usb;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rtic::app(device = crate::hal::pac)]
mod app {
    use crate::{
        Pins, hal::{
            self, Adc,
            adc::AdcPin,
            dma::DMAExt,
            timer::{Alarm, Alarm0, CopyableTimer0},
            usb::UsbBus,
        }, hardware::mux::Mux, scan::ScanState, usb::Usb
    };
    use defmt::info;
    use fugit::MicrosDurationU32;
    use usb_device::bus::UsbBusAllocator;
    use woodox_lib::matrix::KeyboardState;

    #[shared]
    struct Shared {
        #[lock_free]
        keys: KeyboardState,
        #[lock_free]
        scan: ScanState<'static>,
        #[lock_free]
        usb: Usb<UsbBus>,
        #[lock_free]
        alarm: Alarm0<CopyableTimer0>,
    }

    #[local]
    struct Local {}

    #[init(local = [
        adc: Option<hal::Adc> = None, 
        usb: Option<UsbBusAllocator<UsbBus>> = None,
    ])]
    fn init(c: init::Context) -> (Shared, Local) {
        info!("program start");

        unsafe {
            hal::sio::spinlock_reset();
        }

        // ------------------------------------
        // Setup core hardware
        
        let mut resets = c.device.RESETS;
        let mut watchdog = hal::watchdog::Watchdog::new(c.device.WATCHDOG);
        let sio = hal::Sio::new(c.device.SIO);

        // External high-speed crystal on the pico board is 12Mhz
        let clocks = hal::clocks::init_clocks_and_plls(
            crate::XTAL_FREQ_HZ,
            c.device.XOSC,
            c.device.CLOCKS,
            c.device.PLL_SYS,
            c.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let mut timer = hal::Timer::new_timer0(c.device.TIMER0, &mut resets, &clocks);

        let pins = Pins::new(c.device.IO_BANK0, c.device.PADS_BANK0, sio.gpio_bank0, &mut resets);

        info!("core initialization finished");

        // ------------------------------------
        // Setup Mux and ADC for switch scanning

        // Initialize MUX

        let mut mux = crate::hardware::mux::CD74HC4051::new(
            pins.mux_enable.into_push_pull_output(),
            pins.mux_s0.into_push_pull_output(),
            pins.mux_s1.into_push_pull_output(),
            pins.mux_s2.into_push_pull_output(),
        );

        let dma = c.device.DMA.split(&mut resets);
        *c.local.adc = Some(Adc::new(c.device.ADC, &mut resets));
        let adc = c.local.adc.as_mut().unwrap();

        let mut adc_pin_0 = AdcPin::new(pins.mux1_com.into_floating_input()).unwrap();
        let mut adc_pin_1 = AdcPin::new(pins.mux2_com.into_floating_input()).unwrap();
        let mut adc_pin_2 = AdcPin::new(pins.mux3_com.into_floating_input()).unwrap();
        let mut adc_pin_3 = AdcPin::new(pins.mux4_com.into_floating_input()).unwrap();

        let fifo = adc
            .build_fifo()
            .round_robin((&adc_pin_0, &adc_pin_1, &adc_pin_2, &adc_pin_3))
            .set_channel(&mut adc_pin_0)
            .shift_8bit()
            .enable_dma()
            .start_paused();
        let scan = ScanState::new(mux, dma.ch0, fifo, timer);

        info!("adc initialization finished");

        *c.local.usb = Some(UsbBusAllocator::new(hal::usb::UsbBus::new(
            c.device.USB,
            c.device.USB_DPRAM,
            clocks.usb_clock,
            true,
            &mut resets,
        )));

        let usb_bus = c.local.usb.as_mut().unwrap();
        let usb = Usb::new(usb_bus);

        let mut alarm = timer.alarm_0().unwrap();
        alarm.enable_interrupt();
        alarm.schedule(MicrosDurationU32::Hz(1000)).unwrap();

        let keys = KeyboardState::new();

        (Shared { keys, scan, usb, alarm }, Local {})
    }

    #[task(binds = TIMER0_IRQ_0, shared = [keys, scan, usb, alarm])]
    fn usb_timer_alarm(cx: usb_timer_alarm::Context) {
        // Schedule next USB interrupt instantly
        cx.shared.alarm.clear_interrupt();
        cx.shared.alarm.schedule(MicrosDurationU32::Hz(1000)).unwrap();
        
        // Do our matrix scan & usb report
        // this should be fixed timing smaller than the USB timer period
        // so complete before the next IRQ
        cx.shared.scan.scan();
        cx.shared.usb.tick(cx.shared.keys);
        
        // cx.shared.alarm.schedule(MicrosDurationU32::Hz(1)).unwrap();

    }

    #[task(binds = DMA_IRQ_0, shared = [keys, scan])]
    fn scan_dma_completion(cx: scan_dma_completion::Context) {
        let scan = cx.shared.scan;
        scan.dma_completion(cx.shared.keys);
    }
}

hal::bsp_pins!(
    Gpio32 { name: mux_enable },
    Gpio31 { name: mux_s0 },
    Gpio30 { name: mux_s1 },
    Gpio29 { name: mux_s2 },
    Gpio33 { name: led },
    Gpio44 { name: mux1_com },
    Gpio43 { name: mux2_com },
    Gpio42 { name: mux3_com },
    Gpio41 { name: mux4_com },
    Gpio34 { name: d_cs },
    Gpio35 { name: d_dc },
    Gpio36 { name: d_rst },
    Gpio37 { name: d_sda },
    Gpio38 { name: d_scl },
    Gpio4 { name: i2c_sda },
    Gpio5 { name: i2c_sdl },
    Gpio6 { name: i2c_sda_acc },
    Gpio7 { name: i2c_sdl_acc },
);

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Hall Effect Firmware"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
