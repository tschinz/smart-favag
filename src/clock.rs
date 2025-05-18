use embassy_futures::select::{Either, select};
use embassy_rp::gpio::Output;
use embassy_time::{Duration, Ticker};

use crate::buttons::{BUTTON_CHANNEL, ButtonEvent};

pub struct ClockPins {
  pub in1: Output<'static>,
  pub in2: Output<'static>,
  pub en: Output<'static>,
}

#[embassy_executor::task]
pub async fn clock_ticks(mut pins: ClockPins, duration: Duration, en_on: Duration, en_off: Duration) {
  let mut tick_duration = duration;
  let mut tick = Ticker::every(tick_duration);
  let mut en_on = Ticker::every(en_on);
  let mut en_off = Ticker::every(en_off);
  let mut go_once = false;
  loop {
    let button_fut = BUTTON_CHANNEL.receive();
    let tick_fut = tick.next();

    match select(button_fut, tick_fut).await {
      Either::First(event) => {
        match event {
          ButtonEvent::Clicked => {
            defmt::info!("Clock: Button Clicked");
            tick_duration = duration;
          }
          ButtonEvent::Held => {
            defmt::info!("Clock: Button Held");
            tick_duration = duration / 60;
          }
          ButtonEvent::LongHeld => {
            defmt::info!("Clock: Button Long Held");
            tick_duration = duration / 120;
          }
          ButtonEvent::Released => {
            defmt::info!("Clock: Button Released");
            tick_duration = duration;
          }
        }
        // reset ticker with new duration
        tick = Ticker::every(tick_duration);
      }
      Either::Second(_) => {
        defmt::info!("Clock: Tick");
      }
    }

    // Now execute the clock pulse
    en_on.reset();
    pins.en.set_high();
    pins.in1.toggle();
    pins.in2.toggle();
    en_on.next().await;
    pins.en.set_low();
    en_off.reset();
    en_off.next().await;
  }
}
