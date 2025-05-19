use crate::irq::Irqs;
use cyw43::JoinOptions;
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_rp::clocks::RoscRng;
use embassy_rp::{
  Peri,
  gpio::{Level, Output},
  peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0},
  pio::Pio,
};
use embassy_time::Timer;
use heapless::Vec;
use rand::RngCore;

use static_cell::StaticCell;

const WIFI_SSID: &str = match option_env!("WIFI_SSID") {
  Some(val) => val,
  None => "your-wifi-ssid",
};

const WIFI_PASS: &str = match option_env!("WIFI_PASS") {
  Some(val) => val,
  None => "your-wifi-password",
};
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
  _stack: Option<Stack<'static>>,
}

impl Wifi {
  pub async fn new(spawner: &Spawner, wifi_pins: WifiPins, ip: Option<Ipv4Address>, cidr: Option<Ipv4Cidr>) -> Self {
    // firmware and clm selection
    let (fw, clm) = Self::load_fw_clm();

    // spi setup
    let pwr = Output::new(wifi_pins.pwr, Level::Low);
    let cs = Output::new(wifi_pins.cs, Level::High);
    let mut pio = Pio::new(wifi_pins.pio0, Irqs);

    let spi = PioSpi::new(&mut pio.common, pio.sm0, DEFAULT_CLOCK_DIVIDER, pio.irq0, cs, wifi_pins.sck, wifi_pins.mosi, wifi_pins.dma_ch0);

    // cyw43 initialization
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    // spawn runner task
    //unwrap!(spawner.spawn(Self::runner_task(runner)));
    unwrap!(spawner.spawn(cyw43_task(runner)));

    // init power management
    control.init(clm).await;
    control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

    // init network stack
    let config;
    if let (Some(ip), Some(cidr)) = (ip, cidr) {
      // Use static IP configuration instead of DHCP
      config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 { address: cidr, dns_servers: Vec::new(), gateway: Some(ip) });
    } else {
      // Use DHCP
      config = Config::dhcpv4(Default::default());
    }

    // Generate random seed
    let mut rng = RoscRng;
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(net_device, config, RESOURCES.init(StackResources::new()), seed);

    // Spawn network task
    unwrap!(spawner.spawn(net_task(runner)));

    info!("Try to connect to {} with {}", WIFI_SSID, WIFI_PASS);
    let mut join = false;
    for _ in 0..3 {
      match control.join(WIFI_SSID, JoinOptions::new(WIFI_PASS.as_bytes())).await {
        Ok(_) => {
          join = true;
          break;
        }
        Err(err) => {
          join = false;
          info!("join failed with status={}", err.status);
          //Timer::after(Duration::from_secs(1)).await; // optional retry delay
        }
      }
    }

    info!("join {}", join);

    if join {
      if ip == None {
        // Wait for DHCP, not necessary when using static IP
        info!("waiting for DHCP...");
        while !stack.is_config_up() {
          Timer::after_millis(100).await;
        }
        info!("DHCP is now up!");
      }

      info!("waiting for link up...");
      while !stack.is_link_up() {
        Timer::after_millis(500).await;
      }
      info!("Link is up!");

      info!("waiting for stack to be up...");
      stack.wait_config_up().await;
      info!("Stack is up!");
      Self { control, _stack: Some(stack) }
    } else {
      info!("Failed to join network");
      Self { control, _stack: None }
    }
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

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
  runner.run().await
}
