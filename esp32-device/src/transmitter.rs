#![no_std]
#![no_main]

// TODO: use embassy
use common::get_info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, embassy, peripherals::Peripherals, prelude::*, timer::TimerGroup,
};
use esp_println::println;
use esp_wifi::{
    current_millis,
    esp_now::{PeerInfo, BROADCAST_ADDRESS},
};
use heapless::String;

// #[entry]
// fn main() -> ! {
//     let peripherals = Peripherals::take();
//     let system = peripherals.SYSTEM.split();

//     let clocks = ClockControl::max(system.clock_control).freeze();
//     let delay = Delay::new(&clocks);

//     esp_println::logger::init_logger_from_env();

//     let timer = esp_hal::timer::TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
//     let _init = esp_wifi::initialize(
//         esp_wifi::EspWifiInitFor::Wifi,
//         timer,
//         esp_hal::rng::Rng::new(peripherals.RNG),
//         system.radio_clock_control,
//         &clocks,
//     )
//     .unwrap();
//     // TODO: Use ESP Now
//     // https://github.com/esp-rs/esp-wifi/blob/main/esp-wifi/examples/esp_now.rs

//     loop {
//         log::info!("Hello world!");
//         delay.delay(500.millis());
//     }
// }

#[embassy_executor::task]
async fn run() {
    loop {
        println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

// #[embassy_executor::task]
// async fn run() {
//     loop {
// // todo: LED
//         Timer::after(Duration::from_millis(1_000)).await;
//     }
// }

/// Embassy notes:
/// Blocked interupts: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/esp-hal/src/system.rs#L213
/// thread source code: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/esp-hal/src/embassy/executor/thread.rs
/// Solid embassy esp32 examples: https://github.com/esp-rs/esp-hal/blob/3dfea214d45562ef8eefe410003d083ae2c14f98/examples/src/bin/embassy_multicore.rs
/// Solid embassy esp32 with esp-now example: https://github.com/esp-rs/esp-wifi/blob/main/esp-wifi/examples/embassy_esp_now.rs
#[main]
async fn main(spawner: Spawner) {
    // Main with espnow
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

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
    println!("esp-now version {}", esp_now.get_version().unwrap());
    let mac_address = esp_hal::efuse::Efuse::get_mac_address();
    let other_mac_address: [u8; 6] = [12, 139, 149, 66, 112, 36];
    println!("esp-now mac address: {:?}", mac_address);
    println!("other mac address: {:?}", other_mac_address);

    println!("Running sub task");
    spawner.spawn(run()).ok();

    esp_now
        .add_peer(PeerInfo {
            peer_address: other_mac_address,
            lmk: None,
            channel: None,
            encrypt: false,
        })
        .unwrap();

    loop {
        let status = esp_now.send_async(&other_mac_address, b"Hello Peer").await;
        println!("Send hello to peer status: {:?}", status);

        Timer::after(Duration::from_millis(1_000)).await;
    }

    // println!("Running network task");
    // let mut next_send_time = current_millis() + 5 * 1000;
    // loop {
    //     let r = esp_now.receive();
    //     if let Some(r) = r {
    //         println!("Received {:?}", r);

    //         if r.info.dst_address == BROADCAST_ADDRESS {
    //             if !esp_now.peer_exists(&r.info.src_address) {
    //                 esp_now
    //                     .add_peer(PeerInfo {
    //                         peer_address: r.info.src_address,
    //                         lmk: None,
    //                         channel: None,
    //                         encrypt: false,
    //                     })
    //                     .unwrap();
    //             }
    //             let status = esp_now
    //                 .send(&r.info.src_address, b"Hello Peer")
    //                 .unwrap()
    //                 .wait();
    //             println!("Send hello to peer status: {:?}", status);
    //         }
    //     }

    //     if current_millis() >= next_send_time {
    //         next_send_time = current_millis() + 5 * 1000;
    //         println!("Send");
    //         println!("{}", get_info());
    //         let status = esp_now
    //             .send(&BROADCAST_ADDRESS, b"0123456789")
    //             .unwrap()
    //             .wait();
    //         println!("Send broadcast status: {:?}", status)
    //     }
    // }
}
