#![no_std]
#![no_main]

// TODO: use embassy

use esp_backtrace as _;
use esp_hal::{clock::ClockControl, delay::Delay, peripherals::Peripherals, prelude::*};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let timer = esp_hal::timer::TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    let _init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timer,
        esp_hal::rng::Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
    // TODO: Use ESP Now
    // https://github.com/esp-rs/esp-wifi/blob/main/esp-wifi/examples/esp_now.rs

    loop {
        log::info!("Hello world!");
        delay.delay(500.millis());
    }
}
