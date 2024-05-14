#![no_std]

pub fn get_info() -> &'static str {
    return "Hello world this is from the common lib!!!";
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EspNowMacAddress(pub [u8; 6]);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TemperatureReading {
    // temperature: ThermodynamicTemperature<milli_celsius>,
    pub temperature_celsius: f32,
    pub humidity_percentage: f32,
}

// impl TemperatureReading {
//     pub fn celsius(&self) -> f32 {
//         self.deci_celsius as f32 / 10.0
//     }
// }

pub enum NotificationData {}

// TODO: better name
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Notification<T> {
    pub src: EspNowMacAddress,
    pub data: T,
}
