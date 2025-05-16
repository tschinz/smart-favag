#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

//use smart_favag::debounce::*;
//use smart_favag::irq::*;
use smart_favag::output::*;
use smart_favag::wifi::*;

static LED: LedType = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  let p = embassy_rp::init(Default::default());

  // extract singleton pins
  //let wifi_pins = WifiPins { pwr_pin: p.PIN_23, cs_pin: p.PIN_25, sck_pin: p.PIN_24, mosi_pin: p.PIN_29, dma_ch0: p.DMA_CH0, pio0: p.PIO0 };

  // WifiChip abstraction
  //let mut wifi = Wifi::new(&spawner, wifi_pins).await;
  let mut wifi = Wifi::new(&spawner, p).await;

  //let led = Output::new(p.PIN_25, Level::High);
  // inner scope is so that once the mutex is written to, the MutexGuard is dropped, thus the
  // Mutex is released
  //{
  //  *(LED.lock().await) = Some(led);
  //}
  //let dt = 1000 * 1_000_000;
  //let k = 1.003;
  //let delay_1 = Duration::from_nanos(dt);
  //let delay_2 = Duration::from_nanos((dt as f64 * k) as u64);
  //unwrap!(spawner.spawn(toggle_led(&LED, delay_1)));
  //unwrap!(spawner.spawn(toggle_led(&LED, delay_2)));

  let delay = embassy_time::Duration::from_millis(500);
  loop {
    info!("LED on!");
    wifi.led_on().await;
    embassy_time::Timer::after(delay).await;

    info!("LED off!");
    wifi.led_off().await;
    embassy_time::Timer::after(delay).await;
  }
}
