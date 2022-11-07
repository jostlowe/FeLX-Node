//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

pub mod dmx;

extern crate rp_pico;

use crate::dmx::{Dmx, DmxPIO};
use bsp::entry;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    pio::PIOExt,
    sio::Sio,
    watchdog::Watchdog,
};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;
use rp_pico as bsp;

#[entry]
fn main() -> ! {
    // Get Core and Peripherals, watchdog and SIO
    info!("Fetching peripherals");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // Setup internal clocks and plls
    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Create a Delay instance we might use later
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    pac.DMA.ch[0]
        .ch_ctrl_trig
        .write(|w| w.en().set_bit().data_size().size_byte());

    // Get the pins so we can map them for whatever
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

    /*
    Initialize a PIO State machine with the DMX-output program
    */
    let (mut pio, sm0, _sm1, _sm2, _sm3) = pac.PIO0.split(&mut pac.RESETS);

    let mut a = Dmx::new(DmxPIO::new(&mut pio, sm0, &clocks.system_clock).unwrap());

    a.send_universe(&[1, 2, 3, 4]);

    loop {
        info!("on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        info!("off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}

// End of file
