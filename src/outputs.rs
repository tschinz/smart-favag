use defmt::info;
use embassy_rp::Peri;
// other combos for PWM
//use embassy_rp::peripherals::{PIN_0, PWM_SLICE0};
//use embassy_rp::peripherals::{PIN_1, PWM_SLICE6};
//use embassy_rp::peripherals::{PIN_2, PWM_SLICE1};
//use embassy_rp::peripherals::{PIN_10, PWM_SLICE5};
use embassy_rp::peripherals::{PIN_12, PWM_SLICE6};
//use embassy_rp::peripherals::{PIN_14, PWM_SLICE7};
//use embassy_rp::peripherals::{PIN_16, PWM_SLICE0};
//use embassy_rp::peripherals::{PIN_18, PWM_SLICE1};
//use embassy_rp::peripherals::{PIN_20, PWM_SLICE2};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use embassy_time::{Duration, Ticker, Timer};

use crate::helpers::*;

pub struct PwmPin12 {
  pub pin: Peri<'static, PIN_12>,
  pub slice: Peri<'static, PWM_SLICE6>,
}

#[embassy_executor::task(pool_size = 3)]
pub async fn toggle_shared_pin(pin: &'static PinMutexType, delay: Duration) {
  let mut ticker = Ticker::every(delay);
  loop {
    {
      let mut pin_unlocked = pin.lock().await;
      if let Some(pin_ref) = pin_unlocked.as_mut() {
        pin_ref.toggle();
      }
    }
    ticker.next().await;
  }
}

#[embassy_executor::task]
pub async fn task_pwm_pin12(slice: Peri<'static, PWM_SLICE6>, pin: Peri<'static, PIN_12>, freq: f64, duty_cycle: u8) {
  // If we aim for a specific frequency, here is how we can calculate the top value.
  // The top value sets the period of the PWM cycle, so a counter goes from 0 to top and then wraps around to 0.
  // Every such wraparound is one PWM cycle. So here is how we get 2Hz:
  let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
  let divider = 255u8;
  let period = (clock_freq_hz as f64 / (freq * divider as f64)) as u16 - 1;
  info!(
    "PWM Setings Pin 12:\n    freq: {}\n    duty_cycle: {}\n    base_clock: {}\n    divider: {}\n    period: {}",
    freq, duty_cycle, clock_freq_hz, divider, period
  );
  // Duty cycle as a percentage
  let mut c = Config::default();
  c.top = period;
  c.divider = divider.into();

  let mut pwm = Pwm::new_output_a(slice, pin, c.clone());

  loop {
    //pwm.set_duty_cycle_fully_on().unwrap();          // 100% duty cycle, fully on
    pwm.set_duty_cycle_percent(duty_cycle).unwrap(); // as simple percentage.
    //pwm.set_duty_cycle(c.top / 4).unwrap();    // 25% duty cycle. Expressed as 32768/4 = 8192.
    //pwm.set_duty_cycle_fully_off().unwrap();         // 0% duty cycle, fully off.
    Timer::after_secs(1).await;
  }
}
