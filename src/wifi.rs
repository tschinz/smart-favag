use crate::irq::Irqs;
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_rp::{
  Peri,
  gpio::{Level, Output},
  peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0},
  pio::Pio,
};

use static_cell::StaticCell;

pub struct WifiPins {
  pub pwr: Peri<'static, PIN_23>,
  pub cs: Peri<'static, PIN_25>,
  pub sck: Peri<'static, PIN_24>,
  pub mosi: Peri<'static, PIN_29>,
  pub dma_ch0: Peri<'static, DMA_CH0>,
  pub pio0: Peri<'static, PIO0>,
}

pub struct Wifi {
  control: cyw43::Control<'static>,
}

impl Wifi {
  pub async fn new(spawner: &Spawner, wifi_pins: WifiPins) -> Self {
    // Firmware and CLM selection
    let (fw, clm) = Self::load_fw_clm();

    // SPI setup
    let pwr = Output::new(wifi_pins.pwr, Level::Low);
    let cs = Output::new(wifi_pins.cs, Level::High);
    let mut pio = Pio::new(wifi_pins.pio0, Irqs);

    let spi = PioSpi::new(&mut pio.common, pio.sm0, DEFAULT_CLOCK_DIVIDER, pio.irq0, cs, wifi_pins.sck, wifi_pins.mosi, wifi_pins.dma_ch0);

    // CYW43 initialization
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    // Spawn runner task
    //unwrap!(spawner.spawn(Self::runner_task(runner)));
    unwrap!(spawner.spawn(cyw43_task(runner)));

    // Init power management
    control.init(clm).await;
    control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

    Self { control }
  }

  pub async fn led_on(&mut self) {
    self.control.gpio_set(0, true).await;
  }

  pub async fn led_off(&mut self) {
    self.control.gpio_set(0, false).await;
  }

  fn load_fw_clm() -> (&'static [u8], &'static [u8]) {
    // Flashing of FW and CLM directly in the program
    //let fw = include_bytes!("../embassy/cyw43-firmware/43439A0.bin");
    //let clm = include_bytes!("../embassy/cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    // rp2040
    //     probe-rs download embassay/cyw43-firmware/43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download embassay/cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    // rp235x
    //     probe-rs download embassy/cyw43-firmware/43439A0.bin --binary-format bin --chip RP235x --base-address 0x10100000
    //     probe-rs download embassy/cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10140000
    let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    (fw, clm)
  }
}

#[embassy_executor::task]
pub async fn cyw43_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
  runner.run().await
}
