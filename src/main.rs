#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use embassy_sync::mutex::Mutex;
use smart_favag::buttons::*;
use smart_favag::clock::*;
use smart_favag::helpers::*;
use smart_favag::outputs::*;
use smart_favag::watchdog::*;
use smart_favag::wifi::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  info!("Init Clock");
  let p = embassy_rp::init(Default::default());

  // create watchdog (2sec)
  let delay_watchdog = Duration::from_millis(2_000);
  let watchdog = RpWatchdog::new(p.WATCHDOG, delay_watchdog);

  // extract singleton pins for Wifi
  let wifi_pins = WifiPins { pwr: p.PIN_23, cs: p.PIN_25, sck: p.PIN_24, mosi: p.PIN_29, dma_ch0: p.DMA_CH0, pio0: p.PIO0 };
  // WifiChip abstraction
  let mut wifi = Wifi::new(&spawner, wifi_pins, None, None).await;
  //let mut wifi = Wifi::new(&spawner, wifi_pins, Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24), Ipv4Address::new(192, 168, 69, 1)).await;

  // extract singleton pins for clock
  let clock_pins = ClockPins { in1: Output::new(p.PIN_2, Level::High), in2: Output::new(p.PIN_3, Level::Low), en: Output::new(p.PIN_4, Level::Low) };
  let delay_1m = Duration::from_millis(1000 * 60);
  let delay_1s = Duration::from_millis(1000);
  //let delay_500ms = Duration::from_millis(500);
  let delay_250ms = Duration::from_millis(250);
  let delay_en_on = Duration::from_millis(500);

  // PWM pin
  let pwm_freq: f64 = 1_000.0; // 1kHz
  let pwm_duty_cycle: u8 = 10; // 50% duty cycle
  let pwm_pin12 = PwmPin12 { pin: p.PIN_12, slice: p.PWM_SLICE6 };

  // Shared Leds
  static LED_GREEN: PinMutexType = Mutex::new(None);
  static LED_ORANGE: PinMutexType = Mutex::new(None);
  // inner scope is so that once the mutex is written to, the MutexGuard is dropped, thus the Mutex is released
  {
    *(LED_GREEN.lock().await) = Some(Output::new(p.PIN_10, Level::Low));
    *(LED_ORANGE.lock().await) = Some(Output::new(p.PIN_11, Level::Low));
  }

  // Button debounce
  let pin_btn_1 = Input::new(p.PIN_9, Pull::Up);
  let delay_debounce = Duration::from_millis(20);

  // Start Tasks
  info!("Start Tasks");
  // start clock task
  info!("Start clock task");
  unwrap!(spawner.spawn(clock_ticks(clock_pins, delay_1m, delay_en_on)));

  // start output tasks
  info!("Start led tasks");
  unwrap!(spawner.spawn(toggle_shared_pin(&LED_GREEN, delay_1s)));
  unwrap!(spawner.spawn(toggle_shared_pin(&LED_ORANGE, delay_250ms)));

  // start button tasks
  info!("Start button debounce task");
  unwrap!(spawner.spawn(debounce_pin(pin_btn_1, delay_debounce)));

  // start pwm task
  spawner.spawn(task_pwm_pin12(pwm_pin12.slice, pwm_pin12.pin, pwm_freq, pwm_duty_cycle)).unwrap();

  // start watchdog task
  info!("Start watchdog");
  spawner.spawn(feeder(watchdog)).unwrap();
  let delay_1s = Duration::from_millis(1000);

  // main loop
  loop {
    wifi.led_on().await;
    Timer::after(delay_1s).await;
    wifi.led_off().await;
    Timer::after(delay_1s).await;
  }
}
