use crate::helpers::{Chip, TARGET_CHIP};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use embassy_rp::Peripherals;
use embassy_rp::gpio::Output;
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::Pio;

#[embassy_executor::task]
pub async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

pub fn flashing_cyw43_fw() -> (&'static [u8], &'static [u8]) {
    match TARGET_CHIP {
        Chip::RP2040 => flashing_cyw43_rp2040_fw(),
        Chip::RP235x => flashing_cyw43_rp235x_fw(),
    }
}

fn flashing_cyw43_rp2040_fw() -> (&'static [u8], &'static [u8]) {
    todo!()
}

fn flashing_cyw43_rp235x_fw() -> (&'static [u8], &'static [u8]) {
    // Flashing of FW and CLM directly in the program
    //let fw = include_bytes!("../embassy/cyw43-firmware/43439A0.bin");
    //let clm = include_bytes!("../embassy/cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download embassy/cyw43-firmware/43439A0.bin --binary-format bin --chip RP235x --base-address 0x10100000
    //     probe-rs download embassy/cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10140000
    let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };
    (fw, clm)
}

//pub fn setup_cyw43_spi(
//    p: Peripherals,
//    pio: &mut Pio<'_, PIO0>,
//    cs: Output<'static>,
//) -> PioSpi<'static, PIO0, 0, DMA_CH0> {
//    let spi = PioSpi::new(
//        &mut pio.common,
//        pio.sm0,
//        DEFAULT_CLOCK_DIVIDER,
//        pio.irq0,
//        cs,
//        p.PIN_24,
//        p.PIN_29,
//        p.DMA_CH0,
//    );
//    spi
//}
//
