// # Паттерн Observer для комнаты
//
// `DeviceObserver` — трейт с единственным методом `on_device_added`.
// Реализации могут быть как структурами, так и замыканиями (через адаптер
// `FnObserver` в `lib.rs`).
//
// ## Пример с объектом-подписчиком
// ```
// use smart_home::{Room, SmartDevice, SmartSocket};
// use smart_home::DeviceObserver;
//
// struct Logger;
// impl DeviceObserver for Logger {
//     fn on_device_added(&mut self, room: &str, key: &str, _device: &SmartDevice) {
//         println!("[Logger] В комнату '{}' добавлено устройство '{}'", room, key);
//     }
// }
//
// let mut room = Room::empty("Кухня");
// room.subscribe(Logger);
// room.add_device("kettle", SmartSocket::local("Чайник", false, 1500.0));
// ```
//
// ## Пример с замыканием
// ```
// use smart_home::{Room, SmartSocket};
//
// let mut room = Room::empty("Спальня");
// room.subscribe_fn(|room_name, key, _dev| {
//     println!("[замыкание] {}::{}", room_name, key);
// });
// room.add_device("light", SmartSocket::local("Люстра", true, 60.0));
// ```

use crate::SmartDevice;

// Подписчик, который получает уведомление при добавлении устройства.
pub trait DeviceObserver: std::fmt::Debug {
    fn on_device_added(&mut self, room: &str, key: &str, device: &SmartDevice);
}
