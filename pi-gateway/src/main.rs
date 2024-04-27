use clap::Parser;
use common::get_info;
use rppal::uart::{Parity, Uart};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::{error::Error, thread};
use std::time::Duration;
use tokio::{task, time};

const UART: &str = "/dev/ttyAMA1";
/// From the defailt uart config for esp-hal see /esp-hal-0.17.0/src/uart.rs
const BAUD_RATE: u32 = 115_200;
/// From the defailt uart config for esp-hal see /esp-hal-0.17.0/src/uart.rs
const DATA_BITS: u8 = 3; // This might need to be 8 (2^3 = 8)
/// From the defailt uart config for esp-hal see /esp-hal-0.17.0/src/uart.rs
const STOP_BITS: u8 = 1;
/// From the defailt uart config for esp-hal see /esp-hal-0.17.0/src/uart.rs
const PARITY: Parity = Parity::None;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// TODO: mqtt broker
    /// TODO:

    //
    // /// Name of the person to greet
    // #[arg(short, long)]
    // name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("{}", get_info());
    println!("Hello, world!");

    let mut uart = Uart::with_path(UART, BAUD_RATE, PARITY, DATA_BITS, 2)?;
    let mut buffer = [0_u8; 8];

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    task::spawn(async move {
        while let Ok(notification) = eventloop.poll().await {
            println!("Received = {:?}", notification);
        }
    });

    // Spawning a thread since this is synchronous work that runs forever
    thread::spawn(move || {
        // TODO: consume data from channel to send via mqtt


    });

    // Listen for UART data and create a mqtt message
    // client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();

    Ok(())
}

// TODO: an mqtt gateway - reads in data from uart and translates it to an mqtt message
