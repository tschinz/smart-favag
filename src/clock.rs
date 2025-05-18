use defmt::info;
use embassy_rp::Peri;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIN_2, PIN_3, PIN_4, PIN_10, PIN_11, PIN_12, PWM_SLICE5};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Ticker, Timer};

pub struct ClockPins {
  pub in1: Output<'static>,
  pub in2: Output<'static>,
  pub en: Output<'static>,
}

#[embassy_executor::task]
pub async fn clock_ticks(mut pins: ClockPins, duration: Duration, en_on: Duration, en_off: Duration) {
  let mut tick = Ticker::every(duration);
  let mut en_on = Ticker::every(en_on);
  let mut en_off = Ticker::every(en_off);
  loop {
    en_on.reset();
    // set enable pin high
    pins.en.set_high();
    // toggle in1 and in2
    pins.in1.toggle();
    pins.in2.toggle();
    // wait for end of enable
    en_on.next().await;
    pins.en.set_low();
    en_off.reset();
    en_off.next().await;
    tick.next().await;
  }
}
