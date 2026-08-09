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

// ---- SmartDevice -----------------------------------------------------------

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

// ---- SmartHomeError --------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub enum SmartHomeError {
    RoomNotFound(String),
    DeviceNotFound { room: String, device: String },
    NetworkError(String),
}

impl fmt::Display for SmartHomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmartHomeError::RoomNotFound(room) => {
                write!(f, "Комната '{}' не найдена", room)
            }
            SmartHomeError::DeviceNotFound { room, device } => {
                write!(f, "Устройство '{}' не найдено в комнате '{}'", device, room)
            }
            SmartHomeError::NetworkError(msg) => {
                write!(f, "Сетевая ошибка: {}", msg)
            }
        }
    }
}

impl Error for SmartHomeError {}

// ---- Room ------------------------------------------------------------------

#[derive(Debug)]
pub struct Room {
    name: String,
    devices: HashMap<String, SmartDevice>,
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
        Self::new(name, HashMap::new())
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

    pub fn add_device(
        &mut self,
        key: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Option<SmartDevice> {
        let key = key.into();
        let device = device.into();
        for obs in &mut self.observers {
            obs.on_device_added(&self.name, &key, &device);
        }
        self.devices.insert(key, device)
    }

    pub fn remove_device(&mut self, key: &str) -> Option<SmartDevice> {
        self.devices.remove(key)
    }

    pub fn device_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.devices.keys().map(|k| k.as_str()).collect();
        keys.sort();
        keys
    }

    pub fn subscribe<O: DeviceObserver + 'static>(&mut self, observer: O) {
        self.observers.push(Box::new(observer));
    }

    pub fn subscribe_fn<F>(&mut self, f: F)
    where
        F: Fn(&str, &str, &SmartDevice) + Send + Sync + 'static,
    {
        self.observers.push(Box::new(FnObserver(f)));
    }
}

impl Report for Room {
    fn report(&self) -> Result<String, SmartHomeError> {
        let mut lines = vec![format!("  Комната: '{}'", self.name)];
        let mut keys: Vec<_> = self.devices.keys().collect();
        keys.sort();

        // Сначала выводим успешные отчёты, затем ошибки — чтобы не прерывать вывод
        let (ok, err): (Vec<_>, Vec<_>) = keys
            .into_iter()
            .filter_map(|key| self.devices.get(key).map(|d| (key, d)))
            .partition(|(_, d)| d.report().is_ok());

        for (key, device) in ok {
            if let Ok(rep) = device.report() {
                lines.push(format!("    [{}] {}", key, rep));
            }
        }
        for (key, device) in err {
            if let Err(e) = device.report() {
                lines.push(format!("    [{}] ОШИБКА: {}", key, e));
            }
        }

        Ok(lines.join("\n"))
    }
}

// ---- FnObserver (private adapter) -----------------------------------------

// Оборачивает замыкание в DeviceObserver без отдельной структуры на стороне вызывающего кода.
struct FnObserver<F>(F)
where
    F: Fn(&str, &str, &SmartDevice) + Send + Sync;

impl<F: Fn(&str, &str, &SmartDevice) + Send + Sync> fmt::Debug for FnObserver<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FnObserver(<closure>)")
    }
}

impl<F: Fn(&str, &str, &SmartDevice) + Send + Sync> DeviceObserver for FnObserver<F> {
    fn on_device_added(&mut self, room: &str, key: &str, device: &SmartDevice) {
        (self.0)(room, key, device);
    }
}

// ---- SmartHome -------------------------------------------------------------

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
        Self::new(name, HashMap::new())
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

    pub fn room_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.rooms.keys().map(|k| k.as_str()).collect();
        keys.sort();
        keys
    }

    pub fn get_device(&self, room: &str, device: &str) -> Result<&SmartDevice, SmartHomeError> {
        self.get_room(room)
            .ok_or_else(|| SmartHomeError::RoomNotFound(room.to_string()))?
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

// ---- room! macro -----------------------------------------------------------

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

// ---- tests -----------------------------------------------------------------

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
        let room_report = room.report().expect("report не должен упасть");
        home.add_room("office", room);
        let home_report = home.report().expect("report не должен упасть");
        assert!(room_report.contains("Комната"));
        assert!(home_report.contains("Умный дом"));
    }

    #[test]
    fn room_empty_equals_new_with_empty_map() {
        let r1 = Room::empty("X");
        let r2 = Room::new("X", HashMap::new());
        assert_eq!(r1.name(), r2.name());
        assert!(r1.get_device("any").is_none());
        assert!(r2.get_device("any").is_none());
    }

    #[test]
    fn smart_home_empty_equals_new_with_empty_map() {
        let h1 = SmartHome::empty("Дом");
        let h2 = SmartHome::new("Дом", HashMap::new());
        assert_eq!(h1.name(), h2.name());
    }
}
