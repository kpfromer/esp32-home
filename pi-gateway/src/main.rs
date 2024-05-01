use clap::Parser;
use common::{get_info, Notification, TemperatureReading};
use rppal::uart::{Parity, Uart};
use rumqttc::{AsyncClient, Client, MqttOptions, QoS};
use std::thread;
use std::time::Duration;
use tokio::task;

// const UART: &str = "/dev/ttyS0";
/// From the defailt uart config for esp-hal see /esp-hal-0.17.0/src/uart.rs
const BAUD_RATE: u32 = 115_200;
/// From the defailt uart config for esp-hal see /esp-hal-0.17.0/src/uart.rs
const DATA_BITS: u8 = 8; // This might need to be 8 (2^3 = 8)
/// From the defailt uart config for esp-hal see /esp-hal-0.17.0/src/uart.rs
const STOP_BITS: u8 = 1;
/// From the defailt uart config for esp-hal see /esp-hal-0.17.0/src/uart.rs
const PARITY: Parity = Parity::None;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The uart device path to use to receive message from the esp32 device.
    #[arg(short, long, default_value = "/dev/serial0")]
    uart_device: String,

    #[arg(short, long, default_value_t = 115_200u32)]
    uart_baudrate: u32,

    #[arg(short, long, default_value_t = 8u8)]
    uart_data_bits: u8,

    #[arg(short, long, default_value_t = 1u8)]
    uart_stop_bits: u8,

    #[arg(short, long, default_value_t = Parity::None)]
    uart_parity: Party,

    #[arg(short, long)]
    mqtt_broker_address: String,

    #[arg(short, long, default_value_t = 1883u16)]
    mqtt_broker_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        uart_device,
        mqtt_broker_address,
        mqtt_broker_port,
    } = Args::parse();
    println!("{}", get_info());
    println!("Hello, world!");

    let mut mqttoptions = MqttOptions::new("rumqtt-async", mqtt_broker_address, mqtt_broker_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    // Spawning a thread since this is synchronous work that runs forever
    thread::spawn(move || {
        // TODO: consume data from channel to send via mqtt
        for (i, notification) in connection.iter().enumerate() {
            println!("Notification = {:?}", notification);
        }
    });

    let mut uart = Uart::with_path(uart_device, BAUD_RATE, PARITY, DATA_BITS, STOP_BITS)?;
    let mut len_buffer = [0_u8; 1];
    let mut data_buffer = [0_u8; 256];
    // TODO: update

    uart.set_read_mode(1, Duration::ZERO)?;
    loop {
        let mut i = 0;
        match uart.read(&mut len_buffer) {
        Ok(size) if size > 0 => {
                let data_size = len_buffer[0] as usize;
                println!("Data size {}", data_size);

                while i < data_size {
                match uart.read(&mut len_buffer) {
                    Ok(size) if size > 0 => {
                        data_buffer[i] = len_buffer[0];
                        i += 1;
                    }
                    _ => {
                        println!("NOTHING");
                    }
            }

                }

                        // println!("Size {}", size);
                        println!("buffer {:?}", data_buffer);
                        let notification =
                            postcard::from_bytes::<Notification<TemperatureReading>>(
    &data_buffer[..(data_size as usize)],
                            )
                            .unwrap();
                        println!("Notification {:?}", notification);
            client.publish("test", QoS::AtLeastOnce, false, format!("{:?}", notification)).unwrap();
            }
            _ => {thread::sleep(Duration::from_millis(5))}
            // Ok(size) if size > 0 => {
            //     let data_size = buffer[0];
            //     uart.set_read_mode(data_size, Duration::ZERO)?;

            //     match uart.read(&mut buffer) {
            //         Ok(size) if size > 0 => {
            //             // println!("Size {}", size);
            //             println!("buffer {:?}", buffer);
            //             let notification =
            //                 postcard::from_bytes::<Notification<TemperatureReading>>(
            //                     &buffer[..(data_size as usize)],
            //                 )
            //                 .unwrap();
            //             println!("Notification {:?}", notification);
            //         }
            //         _ => {
            //             println!("NOTHING");
            //         }
            //     }
            // }
            // _ => continue,
        }
    }

    // Listen for UART data and create a mqtt message
    // client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();

    Ok(())
}

// TODO: an mqtt gateway - reads in data from uart and translates it to an mqtt message
