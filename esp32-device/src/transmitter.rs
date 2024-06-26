#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod dht20;

use dht20::Dht20;

// TODO: use embassy
use common::TemperatureReading;
use embassy_executor::Spawner;
// TODO: don't use NOOP since it's dual core (move to dual core usage)
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    embassy,
    gpio::{GpioPin, Output, PushPull, IO},
    i2c::I2C,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
};
use esp_println::println;
use esp_wifi::esp_now::PeerInfo;
use static_cell::make_static;

#[embassy_executor::task]
async fn led(
    mut led_pin: GpioPin<Output<PushPull>, 18>,
    signal: &'static Signal<NoopRawMutex, bool>,
) {
    loop {
        signal.wait().await;
        led_pin.set_high();
        Timer::after(Duration::from_millis(1_000)).await;
        led_pin.set_low();
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

/// Embassy notes:
/// Blocked interupts: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/esp-hal/src/system.rs#L213
/// thread source code: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/esp-hal/src/embassy/executor/thread.rs
/// Solid embassy esp32 examples: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/examples/src/bin/embassy_multicore.rs
/// Solid embassy esp32 with esp-now example: https://github.com/esp-rs/esp-wifi/blob/main/esp-wifi/examples/embassy_esp_now.rs
/// TODO: deep sleep
/// good deep sleep article: https://randomnerdtutorials.com/esp32-external-wake-up-deep-sleep/
#[main]
async fn main(spawner: Spawner) {
    // Main with espnow
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    // TODO: don't use max speed, use `boot_defaults` (lowest speed) instead
    let clocks = ClockControl::max(system.clock_control).freeze();

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
    // println!("esp-now version {}", esp_now.get_version().unwrap());
    // let mac_address = esp_hal::efuse::Efuse::get_mac_address();
    let other_mac_address: [u8; 6] = [12, 139, 149, 66, 112, 36];
    // println!("esp-now mac address: {:?}", mac_address);
    // println!("other mac address: {:?}", other_mac_address);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c0 = I2C::new_async(
        peripherals.I2C0,
        // TODO: check if good
        io.pins.gpio26,
        io.pins.gpio27,
        400.kHz(),
        &clocks,
    );
    let mut temp_sensor = Dht20::new(i2c0, 0x38);

    let led_signal = &*make_static!(Signal::new());
    let led_pin = io.pins.gpio18.into_push_pull_output();
    spawner.spawn(led(led_pin, led_signal)).unwrap();

    esp_now
        .add_peer(PeerInfo {
            peer_address: other_mac_address,
            lmk: None,
            channel: None,
            encrypt: false,
        })
        .unwrap();

    let mut buf = [0u8; 32];
    loop {
        let result = temp_sensor.read().await.unwrap();
        println!("Temp: {} °C, Hum: {} %", result.temp, result.hum);

        let reading = TemperatureReading {
            temperature_celsius: result.temp,
            humidity_percentage: result.hum,
        };
        let data = postcard::to_slice(&reading, &mut buf).unwrap();
        let status = esp_now.send_async(&other_mac_address, data).await;
        println!("Sent data to peer status: {:?}", status);
        led_signal.signal(true);

        Timer::after(Duration::from_millis(3_000)).await;
    }
}
