use clap::Parser;
use common::get_info;
use rumqttc::{MqttOptions, AsyncClient, QoS};
use tokio::{task, time};
use std::time::Duration;
use std::error::Error;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// TODO: mqtt broker
    /// TODO:

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

let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
mqttoptions.set_keep_alive(Duration::from_secs(5));

let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    task::spawn(async move {
while let Ok(notification) = eventloop.poll().await {
    println!("Received = {:?}", notification);
}
    });

    // Listen for UART data and create a mqtt message
        // client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();



    Ok(())
}

// TODO: an mqtt gateway - reads in data from uart and translates it to an mqtt message
