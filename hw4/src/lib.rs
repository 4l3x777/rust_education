pub mod builder;
pub mod devices;
pub mod observer;
pub mod report;
pub mod simulators;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub use devices::socket::SmartSocket;
pub use devices::thermometer::SmartThermometer;
pub use observer::DeviceObserver;

pub trait Report {
    fn report(&self) -> Result<String, SmartHomeError>;
}

// Умное устройство (enum-обёртка)
#[derive(Debug)]
pub enum SmartDevice {
    Thermometer(SmartThermometer),
    Socket(SmartSocket),
}

impl Report for SmartDevice {
    fn report(&self) -> Result<String, SmartHomeError> {
        match self {
            SmartDevice::Thermometer(t) => t.report(),
            SmartDevice::Socket(s) => s.report(),
        }
    }
}

impl From<SmartSocket> for SmartDevice {
    fn from(value: SmartSocket) -> Self {
        SmartDevice::Socket(value)
    }
}

impl From<SmartThermometer> for SmartDevice {
    fn from(value: SmartThermometer) -> Self {
        SmartDevice::Thermometer(value)
    }
}

// Комната с поддержкой Observer
#[derive(Debug)]
pub struct Room {
    name: String,
    devices: HashMap<String, SmartDevice>,
    // Подписчики-наблюдатели
    #[allow(clippy::type_complexity)]
    observers: Vec<Box<dyn DeviceObserver>>,
}

impl Room {
    pub fn new(name: &str, devices: HashMap<String, SmartDevice>) -> Self {
        Self {
            name: name.to_string(),
            devices,
            observers: Vec::new(),
        }
    }

    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            devices: HashMap::new(),
            observers: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_device(&self, key: &str) -> Option<&SmartDevice> {
        self.devices.get(key)
    }

    pub fn get_device_mut(&mut self, key: &str) -> Option<&mut SmartDevice> {
        self.devices.get_mut(key)
    }

    // Добавить устройство и уведомить всех наблюдателей (Observer).
    pub fn add_device(
        &mut self,
        key: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Option<SmartDevice> {
        let key_str: String = key.into();
        let device = device.into();
        // Уведомляем наблюдателей
        for obs in &mut self.observers {
            obs.on_device_added(&self.name, &key_str, &device);
        }
        self.devices.insert(key_str, device)
    }

    pub fn remove_device(&mut self, key: &str) -> Option<SmartDevice> {
        self.devices.remove(key)
    }

    // Подписать объект-наблюдатель (impl DeviceObserver).
    pub fn subscribe<O: DeviceObserver + 'static>(&mut self, observer: O) {
        self.observers.push(Box::new(observer));
    }

    // Подписать замыкание в качестве наблюдателя.
    // Замыкание должно иметь сигнатуру `Fn(&str, &str, &SmartDevice)`.
    pub fn subscribe_fn<F>(&mut self, f: F)
    where
        F: Fn(&str, &str, &SmartDevice) + 'static,
    {
        self.observers.push(Box::new(FnObserver(f)));
    }
}

impl Report for Room {
    fn report(&self) -> Result<String, SmartHomeError> {
        let mut lines = vec![format!("  Комната: '{}'", self.name)];
        let mut keys: Vec<_> = self.devices.keys().collect();
        keys.sort();
        let mut errors = Vec::new();

        for key in keys {
            if let Some(device) = self.devices.get(key) {
                match device.report() {
                    Ok(rep) => lines.push(format!("    [{}] {}", key, rep)),
                    Err(e) => {
                        errors.push(format!("    [{}] ОШИБКА: {}", key, e));
                    }
                }
            }
        }

        lines.extend(errors);
        Ok(lines.join("\n"))
    }
}

// Адаптер: оборачивает замыкание в трейт-объект DeviceObserver.
struct FnObserver<F>(F)
where
    F: Fn(&str, &str, &SmartDevice);

impl<F> fmt::Debug for FnObserver<F>
where
    F: Fn(&str, &str, &SmartDevice),
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FnObserver(<closure>)")
    }
}

impl<F> DeviceObserver for FnObserver<F>
where
    F: Fn(&str, &str, &SmartDevice),
{
    fn on_device_added(&mut self, room: &str, key: &str, device: &SmartDevice) {
        (self.0)(room, key, device);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SmartHomeError {
    RoomNotFound(String),
    DeviceNotFound { room: String, device: String },
    NetworkError(String),
}

impl fmt::Display for SmartHomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmartHomeError::RoomNotFound(room) => write!(f, "Комната '{}' не найдена", room),
            SmartHomeError::DeviceNotFound { room, device } => {
                write!(f, "Устройство '{}' не найдено в комнате '{}'", device, room)
            }
            SmartHomeError::NetworkError(msg) => write!(f, "Сетевая ошибка: {}", msg),
        }
    }
}

impl Error for SmartHomeError {}

// Умный дом
#[derive(Debug)]
pub struct SmartHome {
    name: String,
    rooms: HashMap<String, Room>,
}

impl SmartHome {
    pub fn new(name: &str, rooms: HashMap<String, Room>) -> Self {
        Self {
            name: name.to_string(),
            rooms,
        }
    }

    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rooms: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_room(&self, key: &str) -> Option<&Room> {
        self.rooms.get(key)
    }

    pub fn get_room_mut(&mut self, key: &str) -> Option<&mut Room> {
        self.rooms.get_mut(key)
    }

    pub fn add_room(&mut self, key: impl Into<String>, room: Room) -> Option<Room> {
        self.rooms.insert(key.into(), room)
    }

    pub fn remove_room(&mut self, key: &str) -> Option<Room> {
        self.rooms.remove(key)
    }

    pub fn get_device(&self, room: &str, device: &str) -> Result<&SmartDevice, SmartHomeError> {
        let room_ref = self
            .get_room(room)
            .ok_or_else(|| SmartHomeError::RoomNotFound(room.to_string()))?;

        room_ref
            .get_device(device)
            .ok_or_else(|| SmartHomeError::DeviceNotFound {
                room: room.to_string(),
                device: device.to_string(),
            })
    }
}

impl Report for SmartHome {
    fn report(&self) -> Result<String, SmartHomeError> {
        let mut lines = vec![format!("=== Умный дом: '{}' ===", self.name)];
        let mut keys: Vec<_> = self.rooms.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(room) = self.rooms.get(key) {
                lines.push(format!("[{}]", key));
                match room.report() {
                    Ok(rep) => lines.push(rep),
                    Err(e) => lines.push(format!("  ОШИБКА в комнате '{}': {}", key, e)),
                }
            }
        }

        lines.push("==============================".to_string());
        Ok(lines.join("\n"))
    }
}

#[macro_export]
macro_rules! room {
    ($name:expr $(, ($key:expr, $device:expr))* $(,)?) => {{
        let mut room = $crate::Room::empty($name);
        $(
            room.add_device($key, $device);
        )*
        room
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_room_and_device_return_option() {
        let mut home = SmartHome::empty("Дом");
        let mut room = Room::empty("Гостиная");
        room.add_device("socket_tv", SmartSocket::local("TV", true, 150.0));
        room.add_device("thermo", SmartThermometer::local("T1", 22.5));
        home.add_room("living", room);

        assert!(home.get_room("living").is_some());
        assert!(home.get_room("kitchen").is_none());

        let living = home
            .get_room("living")
            .expect("Комната должна существовать");
        assert!(living.get_device("socket_tv").is_some());
        assert!(living.get_device("none").is_none());
    }

    #[test]
    fn can_add_and_remove_rooms_and_devices() {
        let mut room = Room::empty("Кухня");
        room.add_device("socket", SmartSocket::local("Чайник", true, 1200.0));
        assert!(room.get_device("socket").is_some());
        assert!(room.remove_device("socket").is_some());
        assert!(room.get_device("socket").is_none());

        let mut home = SmartHome::empty("Дом");
        home.add_room("kitchen", room);
        assert!(home.get_room("kitchen").is_some());
        assert!(home.remove_room("kitchen").is_some());
        assert!(home.get_room("kitchen").is_none());
    }

    #[test]
    fn get_device_returns_typed_error() {
        let home = SmartHome::empty("Дом");
        let err = home
            .get_device("missing_room", "socket")
            .expect_err("Ожидаем ошибку отсутствующей комнаты");
        assert_eq!(err, SmartHomeError::RoomNotFound("missing_room".into()));

        let mut home = SmartHome::empty("Дом");
        let mut room = Room::empty("Спальня");
        room.add_device("thermo", SmartThermometer::local("Т", 19.0));
        home.add_room("bedroom", room);

        let err = home
            .get_device("bedroom", "socket")
            .expect_err("Ожидаем ошибку отсутствующего устройства");
        assert_eq!(
            err,
            SmartHomeError::DeviceNotFound {
                room: "bedroom".into(),
                device: "socket".into()
            }
        );
    }

    #[test]
    fn report_trait_is_implemented() {
        let mut home = SmartHome::empty("Дом");
        let mut room = Room::empty("Кабинет");
        room.add_device("thermo", SmartThermometer::local("Т1", 21.2));
        room.add_device("socket", SmartSocket::local("ПК", false, 500.0));
        let report_str = room.report().expect("report не должен упасть");
        home.add_room("office", room);

        let home_report = home.report().expect("report не должен упасть");

        assert!(report_str.contains("Комната"));
        assert!(home_report.contains("Умный дом"));
    }
}
