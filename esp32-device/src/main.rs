#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use common::{EspNowMacAddress, Notification, TemperatureReading};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pubsub::PubSubChannel;

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    embassy,
    gpio::IO,
    peripherals::{Peripherals, UART2},
    prelude::*,
    timer::TimerGroup,
    uart::{
        config::{AtCmdConfig, Config, DataBits, Parity, StopBits},
        ClockSource, TxRxPins, Uart, UartRx, UartTx,
    },
    Async,
};
use esp_println::println;
use esp_wifi::esp_now::PeerInfo;
use static_cell::make_static;

// rx_fifo_full_threshold
const READ_BUF_SIZE: usize = 64;
// EOT (CTRL-D)
// const AT_CMD: u8 = 0x04;
const AT_CMD: u8 = b'h';

#[embassy_executor::task]
async fn writer(
    mut tx: UartTx<'static, UART2, Async>,
    // Capacity for 5 items, 1 subscriber and 1 publisher
    data_channel: &'static PubSubChannel<NoopRawMutex, Notification<TemperatureReading>, 5, 1, 1>,
) {
    let mut subscriber = data_channel.subscriber().unwrap();
    let mut buf: [u8; 256] = [0; 256];
    loop {
        println!("UART received new message to write.");
        let notification = subscriber.next_message_pure().await;

        let data_size = postcard::to_slice(&notification, &mut buf[1..]).unwrap().len();
        buf[0] = data_size.try_into().unwrap();

        embedded_io_async::Write::write(&mut tx, &buf[..(1 + data_size)])
            .await
            .unwrap();
        embedded_io_async::Write::flush(&mut tx).await.unwrap();
        println!("Written to UART 2.");
    }
}

#[embassy_executor::task]
async fn esp_now_listener(
    mut esp_now: esp_wifi::esp_now::EspNow<'static>,
    data_channel: &'static PubSubChannel<NoopRawMutex, Notification<TemperatureReading>, 5, 1, 1>,
) {
    let publisher = data_channel.publisher().unwrap();
    loop {
        let message = esp_now.receive_async().await;
        if !esp_now.peer_exists(&message.info.src_address) {
            esp_now
                .add_peer(PeerInfo {
                    peer_address: message.info.src_address,
                    lmk: None,
                    channel: None,
                    encrypt: false,
                })
                .unwrap();
            println!("ESP-NOW added new peer.");
        }
        println!("Received {:?}", message);
        let reading =
            postcard::from_bytes::<TemperatureReading>(&message.data[..(message.len as usize)])
                .unwrap();
        println!("Data: {:?}", reading);
        let notification = Notification::<TemperatureReading> {
            src: EspNowMacAddress(message.info.src_address),
            data: reading,
        };
        publisher.publish_immediate(notification);
    }
}

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, UART2, Async>) {
    const MAX_BUFFER_SIZE: usize = 10 * READ_BUF_SIZE + 16;

    let mut rbuf: [u8; MAX_BUFFER_SIZE] = [0u8; MAX_BUFFER_SIZE];
    loop {
        let _r = embedded_io_async::Read::read(&mut rx, &mut rbuf).await.ok();
        // Do nothing with the data
        // Might want to have the reader alert writer that messaged was received

        // TODO: remove

        // let r = embedded_io_async::Read::read(&mut rx, &mut rbuf).await.ok();
        // if let Some(size) = r {
        //     println!("OUTPUT {:?}", core::str::from_utf8(&rbuf[0..size]));
        // }
    }
}

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
    let esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();
    println!("esp-now version {}", esp_now.get_version().unwrap());
    let mac_address = esp_hal::efuse::Efuse::get_mac_address();
    let other_mac_address: [u8; 6] = [12, 139, 149, 66, 112, 36];
    println!("esp-now mac address: {:?}", mac_address);
    println!("other mac address: {:?}", other_mac_address);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let uart_pins = TxRxPins::new_tx_rx(
        io.pins.gpio16.into_push_pull_output(),
        io.pins.gpio17.into_floating_input(),
    );
    // let mut uart2 =
    //     Uart::new_async_with_config(peripherals.UART2, Config::default(), Some(uart_pins), &clocks);
    let mut uart2 = Uart::new_async_with_config(
        peripherals.UART2,
        Config::default(),
        // Config {
        //     baudrate: 19_200,
        //     data_bits: DataBits::DataBits8,
        //     parity: Parity::ParityNone,
        //     stop_bits: StopBits::STOP1,
        //     clock_source: ClockSource::RefTick,
        // },
        Some(uart_pins),
        &clocks,
    );
    uart2.set_at_cmd(AtCmdConfig::new(None, None, None, AT_CMD, None));
    uart2
        .set_rx_fifo_full_threshold(READ_BUF_SIZE as u16)
        .unwrap();
    let (tx, rx) = uart2.split();

    let signal = &*make_static!(PubSubChannel::new());

    spawner.spawn(reader(rx)).ok();
    spawner.spawn(writer(tx, signal)).ok();
    spawner.spawn(esp_now_listener(esp_now, signal)).ok();

    // esp_now
    //     .add_peer(PeerInfo {
    //         peer_address: other_mac_address,
    //         lmk: None,
    //         channel: None,
    //         encrypt: false,
    //     })
    //     .unwrap();

    // loop {
    //     let status = esp_now.send_async(&other_mac_address, b"Hello Peer").await;
    //     println!("Send hello to peer status: {:?}", status);

    //     Timer::after(Duration::from_millis(1_000)).await;
    // }

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
