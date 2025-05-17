use defmt::info;
use embassy_rp::Peri;
use embassy_rp::peripherals::{PIN_4, PWM_SLICE2};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use embassy_time::{Duration, Ticker, Timer};

use crate::helpers::*;

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
pub async fn pwm_pin4(slice: Peri<'static, PWM_SLICE2>, pin: Peri<'static, PIN_4>, freq: f64, duty_cycle: u8) {
  // If we aim for a specific frequency, here is how we can calculate the top value.
  // The top value sets the period of the PWM cycle, so a counter goes from 0 to top and then wraps around to 0.
  // Every such wraparound is one PWM cycle. So here is how we get 2Hz:
  let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
  let divider = 255u8;
  let period = (clock_freq_hz as f64 / (freq * divider as f64)) as u16 - 1;
  info!("{}, {}, {}, {}", freq, clock_freq_hz, divider, period);
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
