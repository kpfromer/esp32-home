use clap::Parser;
use common::{get_info, EspNowMacAddress, Notification, TemperatureReading};
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

    // #[arg(short, long, default_value_t = Parity::None)]
    // uart_parity: Parity,
    #[arg(short, long)]
    mqtt_broker_address: String,

    #[arg(short, long, default_value_t = 1883u16)]
    mqtt_broker_port: u16,
}

fn read_uart_data<'a>(uart: &mut Uart, data_buffer: &'a mut [u8; 256]) -> &'a [u8] {
    let mut len_buffer = [0_u8; 1];
    let mut i = 0;
    loop {
        match uart.read(&mut len_buffer) {
            Ok(size) if size > 0 => {
                let data_size = len_buffer[0] as usize;
                while i < data_size {
                    match uart.read(&mut len_buffer) {
                        Ok(size) if size > 0 => {
                            data_buffer[i] = len_buffer[0];
                            i += 1;
                        }
                        _ => continue,
                    }
                }

                return &data_buffer[..(data_size as usize)];
            }
            _ => thread::sleep(Duration::from_millis(5)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: get solid logging up
    let Args {
        uart_device,
        uart_baudrate,
        uart_data_bits,
        uart_stop_bits,
        // uart_parity,
        mqtt_broker_address,
        mqtt_broker_port,
    } = Args::parse();

    println!("running!!!");

    let mut mqttoptions = MqttOptions::new("rumqtt-async", mqtt_broker_address, mqtt_broker_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    thread::spawn(move || {
        for (i, notification) in connection.iter().enumerate() {
            println!("Notification = {:?}", notification);
        }
    });

    let mut uart = Uart::with_path(
        uart_device,
        uart_baudrate,
        PARITY,
        // uart_parity,
        uart_data_bits,
        uart_stop_bits,
    )?;
    let mut data_buffer = [0_u8; 256];
    uart.set_read_mode(1, Duration::ZERO)?;
    loop {
        let data = read_uart_data(&mut uart, &mut data_buffer);
        let notification = postcard::from_bytes::<Notification<TemperatureReading>>(data).unwrap();
        let topic = {
            if notification.src == EspNowMacAddress([12, 139, 149, 66, 103, 160]) {
                "mushrooms/temperature-humidity-sensor"
            } else if notification.src == EspNowMacAddress([100, 183, 8, 194, 244, 32]) {
                "mushrooms/epulse-temperature-humidity-sensor"
            } else if notification.src == EspNowMacAddress([100, 183, 8, 194, 244, 40]) {
                "mushroom-container/epulse-temperature-humidity-sensor"
            } else {
                "unknown"
            }
        };

        let temp = notification.data.temperature_celsius;
        println!("Notification {:?}", notification);
        let json = format!(
            "{{ \"temperature\": {}, \"humidity\": {} }}",
            temp, notification.data.humidity_percentage
        );
        println!("Print data: {:?}", json);

        client
            .publish(
                // [area]/[name]/
                topic,
                QoS::AtLeastOnce,
                false,
                json,
            )
            .unwrap();
    }
}
