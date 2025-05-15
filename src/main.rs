#![no_std]
#![no_main]

use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::info;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::pio::Pio;
use embassy_time::{Duration, Instant, Timer, with_deadline};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use smart_favag::debounce::*;
use smart_favag::irq::*;
use smart_favag::wifi::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Wifi Chip Configuration
    let (fw, clm) = flashing_cyw43_fw();

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Button
    let mut btn = Debouncer::new(Input::new(p.PIN_9, Pull::Up), Duration::from_millis(20));

    // Led duration
    let delay = Duration::from_millis(500);

    loop {
        // // button pressed
        // btn.debounce().await;
        // let start = Instant::now();
        // info!("Button Press");

        // match with_deadline(start + Duration::from_secs(1), btn.debounce()).await {
        //     // Button Released < 1s
        //     Ok(_) => {
        //         info!("Button pressed for: {}ms", start.elapsed().as_millis());
        //         continue;
        //     }
        //     // button held for > 1s
        //     Err(_) => {
        //         info!("Button Held");
        //     }
        // }

        // match with_deadline(start + Duration::from_secs(1), btn.debounce()).await {
        //     // Button Released < 1s
        //     Ok(_) => {
        //         info!("Button pressed for: {}ms", start.elapsed().as_millis());
        //         continue;
        //     }
        //     // button held for > 1s
        //     Err(_) => {
        //         info!("Button Held");
        //     }
        // }

        // // wait for button release before handling another press
        // btn.debounce().await;
        // info!("Button pressed for: {}ms", start.elapsed().as_millis());

        // Led control
        info!("led on!");
        control.gpio_set(0, true).await;
        Timer::after(delay).await;

        info!("led off!");
        control.gpio_set(0, false).await;
        Timer::after(delay).await;
    }
}
