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

#[embassy_executor::task(pool_size = 3)]
pub async fn clock_ticks(mut pins: ClockPins, duration: Duration, en_on: Duration, en_off: Duration) {
  let mut tick = Ticker::every(duration);
  let mut en_on = Ticker::every(en_on);
  let mut en_off = Ticker::every(en_off);
  loop {
    en_on.reset();
    pins.en.set_high();
    en_on.next().await;
    pins.in1.toggle();
    pins.in2.toggle();
    en_off.reset();
    pins.en.set_low();
    en_off.next().await;

    tick.next().await;
  }
}

#[embassy_executor::task]
pub async fn pwm_pin11(slice: Peri<'static, PWM_SLICE5>, pin: Peri<'static, PIN_10>, freq: f64, duty_cycle: u8) {
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
