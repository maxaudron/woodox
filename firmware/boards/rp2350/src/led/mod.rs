use ws2812_pio::Ws2812;

use hal::{
    pio::PIOExt,
    sio::Sio,
};

    // let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    // let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    // let mut ws = Ws2812::new(
    //     pins.led.into_mode(),
    //     &mut pio,
    //     sm0,
    //     clocks.peripheral_clock.freq(),
    //     timer.count_down(),
    // );

    // use smart_leds::{SmartLedsWrite, RGB8};
    // let color = (255, 35, 0);
    // let color: RGB8 = color.into();
    // ws.write(
    //     [color, color, color, color, color, color, color, color]
    //         .iter()
    //         .copied(),
    // )
    // .unwrap();

    // loop {
    //     Rainbow vomit
    //     if color.0 > 0 && color.2 == 0 {
    //         color.0 -= 1;
    //         color.1 += 1;
    //     }

    //     if color.1 > 0 && color.0 == 0 {
    //         color.1 -= 1;
    //         color.2 += 1;
    //     }

    //     if color.2 > 0 && color.1 == 0 {
    //         color.2 -= 1;
    //         color.0 += 1;
    //     }

    //     delay.delay_ms(50);
    // }
