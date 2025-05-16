use embassy_rp::gpio::Output;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Ticker};

pub type LedType = Mutex<ThreadModeRawMutex, Option<Output<'static>>>;

#[embassy_executor::task(pool_size = 2)]
pub async fn toggle_led(led: &'static LedType, delay: Duration) {
  let mut ticker = Ticker::every(delay);
  loop {
    {
      let mut led_unlocked = led.lock().await;
      if let Some(pin_ref) = led_unlocked.as_mut() {
        pin_ref.toggle();
      }
    }
    ticker.next().await;
  }
}
