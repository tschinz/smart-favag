use defmt::info;
use embassy_rp::Peri;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_rp::peripherals::PIN_9;
use embassy_time::{Duration, Instant, Timer, with_deadline};
use {defmt_rtt as _, panic_probe as _};

pub struct Debouncer<'a> {
  input: Input<'a>,
  debounce: Duration,
}

impl<'a> Debouncer<'a> {
  pub fn new(input: Input<'a>, debounce: Duration) -> Self {
    Self { input, debounce }
  }

  pub async fn debounce(&mut self) -> Level {
    loop {
      let l1 = self.input.get_level();

      self.input.wait_for_any_edge().await;

      Timer::after(self.debounce).await;

      let l2 = self.input.get_level();
      if l1 != l2 {
        break l2;
      }
    }
  }
}

#[embassy_executor::task]
pub async fn debounce_pin(pin: Input<'static>, duration: Duration) {
  let mut btn = Debouncer::new(pin, duration);

  loop {
    // button pressed
    btn.debounce().await;
    let start = Instant::now();
    info!("Button Press");

    match with_deadline(start + Duration::from_secs(1), btn.debounce()).await {
      // Button Released < 1s
      Ok(_) => {
        info!("Button pressed for: {}ms", start.elapsed().as_millis());
        continue;
      }
      // button held for > 1s
      Err(_) => {
        info!("Button Held");
      }
    }

    match with_deadline(start + Duration::from_secs(5), btn.debounce()).await {
      // Button released <5s
      Ok(_) => {
        info!("Button pressed for: {}ms", start.elapsed().as_millis());
        continue;
      }
      // button held for > >5s
      Err(_) => {
        info!("Button Long Held");
      }
    }

    // wait for button release before handling another press
    btn.debounce().await;
    info!("Button pressed for: {}ms", start.elapsed().as_millis());
  }
}
