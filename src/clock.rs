use defmt::info;
use embassy_rp::Peri;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIN_2, PIN_3, PIN_4, PIN_10, PIN_11, PIN_12, PWM_SLICE5};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Ticker, Timer};

pub struct ClockPins {
  pub in1: Peri<'static, PIN_2>,
  pub in2: Peri<'static, PIN_3>,
  pub en: Peri<'static, PIN_4>,
}

#[embassy_executor::task(pool_size = 3)]
pub async fn clock_ticks(pins: ClockPins) {
  let delay_1min = Duration::from_millis(1000 * 60);
  let delay_1s = Duration::from_millis(1000);
  let delay_500ms = Duration::from_millis(500);
  let delay_250ms = Duration::from_millis(250);
  let delay_en_on = Duration::from_millis(350);
  let delay_en_off = Duration::from_millis(150);
  let en_freq: f64 = 0.5; // 2Hz
  let en_duty_cycle: u8 = 70; // 70% duty cycle

  let mut pin_in1 = Output::new(pins.in1, Level::High);
  let mut pin_in2 = Output::new(pins.in2, Level::Low);
  let mut pin_en = Output::new(pins.en, Level::High);

  let mut tick = Ticker::every(delay_1s);
  let mut en_on = Ticker::every(delay_en_on);
  let mut en_off = Ticker::every(delay_en_off);
  loop {
    en_on.reset();
    pin_en.set_high();
    en_on.next().await;
    pin_in1.toggle();
    pin_in2.toggle();
    en_off.reset();
    pin_en.set_low();
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
