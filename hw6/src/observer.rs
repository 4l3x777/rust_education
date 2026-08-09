//! Трейт Observer для событий комнаты.
//!
//! ```
//! use smart_home::{Room, SmartDevice, SmartSocket, DeviceObserver};
//!
//! #[derive(Debug)]
//! struct Logger;
//! impl DeviceObserver for Logger {
//!     fn on_device_added(&mut self, room: &str, key: &str, _device: &SmartDevice) {
//!         println!("[log] {}::{}", room, key);
//!     }
//! }
//!
//! let mut room = Room::empty("Кухня");
//! room.subscribe(Logger);
//! room.add_device("kettle", SmartSocket::local("Чайник", false, 1500.0));
//! ```

use crate::SmartDevice;

pub trait DeviceObserver: std::fmt::Debug + Send + Sync {
    fn on_device_added(&mut self, room: &str, key: &str, device: &SmartDevice);
}
