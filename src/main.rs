#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use smart_favag::buttons::*;
use smart_favag::outputs::*;
use smart_favag::watchdog::*;
use smart_favag::wifi::*;

static PIN_IN1: PinMutexType = Mutex::new(None);
static PIN_IN2: PinMutexType = Mutex::new(None);
static PIN_EN: PinMutexType = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  info!("Init Clock");
  let p = embassy_rp::init(Default::default());

  // create watchdog (2sec)
  let delay_watchdog = Duration::from_millis(2_000);
  let watchdog = RpWatchdog::new(p.WATCHDOG, delay_watchdog);

  // extract singleton pins for Wifi
  let wifi_pins = WifiPins { pwr_pin: p.PIN_23, cs_pin: p.PIN_25, sck_pin: p.PIN_24, mosi_pin: p.PIN_29, dma_ch0: p.DMA_CH0, pio0: p.PIO0 };

  // WifiChip abstraction
  let mut wifi = Wifi::new(&spawner, wifi_pins).await;

  // Clock output
  let delay_clock = Duration::from_millis(500);
  let delay_en_on = Duration::from_millis(350);
  let pin_in1 = Output::new(p.PIN_10, Level::High);
  let pin_in2 = Output::new(p.PIN_11, Level::Low);
  let pin_en = Output::new(p.PIN_12, Level::High);
  // inner scope is so that once the mutex is written to, the MutexGuard is dropped, thus the Mutex is released
  {
    *(PIN_IN1.lock().await) = Some(pin_in1);
    *(PIN_IN2.lock().await) = Some(pin_in2);
    *(PIN_EN.lock().await) = Some(pin_en);
  }

  // Button debounce
  let pin_btn_1 = Input::new(p.PIN_9, Pull::Up);
  let delay_debounce = Duration::from_millis(20);

  // start output tasks
  info!("Start in1, in2 and en tasks");
  unwrap!(spawner.spawn(toggle_shared_pin(&PIN_IN1, delay_clock)));
  unwrap!(spawner.spawn(toggle_shared_pin(&PIN_IN2, delay_clock)));
  unwrap!(spawner.spawn(toggle_shared_pin(&PIN_EN, delay_en_on)));

  // start button tasks
  unwrap!(spawner.spawn(debounce_pin(pin_btn_1, delay_debounce)));

  let delay_blink = Duration::from_millis(1000);

  spawner.spawn(pwm_set_dutycycle(p.PWM_SLICE2, p.PIN_4)).unwrap();

  // start watchdog task
  spawner.spawn(feeder(watchdog)).unwrap();

  // main loop
  loop {
    wifi.led_on().await;
    Timer::after(delay_blink).await;
    wifi.led_off().await;
    Timer::after(delay_blink).await;
  }
}
