#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod dht20;

use dht20::Dht20;

// TODO: use embassy
use common::TemperatureReading;
use embassy_executor::Spawner;
// TODO: don't use NOOP since it's dual core (move to dual core usage)
use core::time::Duration as CoreDuration;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    embassy,
    gpio::{GpioPin, Output, PushPull, IO},
    i2c::I2C,
    peripherals::Peripherals,
    prelude::*,
    rtc_cntl::{get_reset_reason, get_wakeup_cause, sleep::TimerWakeupSource, Rtc, SocResetReason},
    timer::TimerGroup,
    Cpu,
};
use esp_println::println;
use esp_wifi::esp_now::PeerInfo;

const INTERVAL: CoreDuration = CoreDuration::from_secs(60 * 5);
const ESP_GATEWAY: [u8; 6] = [12, 139, 149, 66, 112, 36];

/// Embassy notes:
/// Blocked interupts: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/esp-hal/src/system.rs#L213
/// thread source code: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/esp-hal/src/embassy/executor/thread.rs
/// Solid embassy esp32 examples: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/examples/src/bin/embassy_multicore.rs
/// Solid embassy esp32 with esp-now example: https://github.com/esp-rs/esp-wifi/blob/main/esp-wifi/examples/embassy_esp_now.rs
/// TODO: deep sleep
/// good deep sleep article: https://randomnerdtutorials.com/esp32-external-wake-up-deep-sleep/
#[main]
async fn main(_spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    // Lowest speed clocks are used from `boot_defaults`
    // However, the wifi driver requires we use the highest clock speeds
    let clocks = ClockControl::max(system.clock_control).freeze();

    let mut delay = Delay::new(&clocks);
    let mut rtc = Rtc::new(peripherals.LPWR, None);

    #[cfg(not(feature = "no-print"))]
    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let timer = TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    let init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timer,
        esp_hal::rng::Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
    let wifi = peripherals.WIFI;
    let mut esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();
    esp_now
        .add_peer(PeerInfo { peer_address: ESP_GATEWAY, lmk: None, channel: None, encrypt: false })
        .unwrap();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c0 = I2C::new_async(
        peripherals.I2C0,
        // SDA
        io.pins.gpio22,
        // SCL
        io.pins.gpio23,
        400.kHz(),
        &clocks,
    );
    let mut temp_sensor = Dht20::new(i2c0, 0x38);

    #[cfg(not(feature = "no-print"))]
    println!("up and runnning!");
    let reason = get_reset_reason(Cpu::ProCpu).unwrap_or(SocResetReason::ChipPowerOn);
    #[cfg(not(feature = "no-print"))]
    println!("reset reason: {:?}", reason);
    let wake_reason = get_wakeup_cause();
    #[cfg(not(feature = "no-print"))]
    println!("wake reason: {:?}", wake_reason);

    let result = temp_sensor.read().await.unwrap();
    #[cfg(not(feature = "no-print"))]
    println!("Temp: {} °C, Hum: {} %", result.temp, result.hum);

    let mut buf = [0u8; 32];
    let reading =
        TemperatureReading { temperature_celsius: result.temp, humidity_percentage: result.hum };
    let data = postcard::to_slice(&reading, &mut buf).unwrap();
    let _status = esp_now.send_async(&ESP_GATEWAY, data).await;

    let timer = TimerWakeupSource::new(INTERVAL);
    rtc.sleep_deep(&[&timer], &mut delay);
}
