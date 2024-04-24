#![no_std]

use serde::{Deserialize, Serialize};
// TODO: figure out
// use uom::si::thermodynamic_temperature;
// use uom::si::u16::*;

// unit! {
//     system: uom::si;
//     quantity: uom::si::thermodynamic_temperature;

//     @milli_celsius: 1000; "mC", "degree milli celsius", "degrees milli celsius";
// }
// unit_symbol!(millicelsus, "mC", degree_celsius / 1000.0);

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn get_info() -> &'static str {
    return "Hello world this is from the common lib!!!";
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct DevAddr(pub u8);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TemperatureReading {
    // temperature: ThermodynamicTemperature<milli_celsius>,
    milli_celsius: u16,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Transmission<T> {
    pub src: DevAddr,
    pub msg: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
