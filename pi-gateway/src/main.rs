use clap::Parser;
use common::get_info;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// TODO: mqtt broker
    /// TODO:

    // /// Name of the person to greet
    // #[arg(short, long)]
    // name: String,

    // /// Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

fn main() {
    let args = Args::parse();
    println!("{}", get_info());
    println!("Hello, world!");
}

// TODO: an mqtt gateway - reads in data from uart and translates it to an mqtt message
