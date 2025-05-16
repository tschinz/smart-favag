use defmt::info;
use embassy_rp::Peri;
use embassy_rp::peripherals::WATCHDOG;
use embassy_rp::watchdog::Watchdog;
use embassy_time::{Duration, Timer};

pub struct RpWatchdog {
  pub handle: Watchdog,
  pub delay: Duration,
}

impl RpWatchdog {
  pub fn new(wd: Peri<'static, WATCHDOG>, delay: Duration) -> Self {
    let handle = Watchdog::new(wd);
    info!("Watchdog initialized");
    Self { handle, delay }
  }

  pub fn start(&mut self) {
    self.handle.start(self.delay);
    info!("Watchdog started");
  }
}

#[embassy_executor::task]
pub async fn feeder(mut wd: RpWatchdog) {
  wd.start();
  loop {
    Timer::after(wd.delay / 2).await;
    // watchdog feed
    wd.handle.feed();
  }
}
