use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub trait Report {
    fn report(&self) -> String;
}

// Умный термометр
#[derive(Debug, Clone)]
pub struct SmartThermometer {
    name: String,
    temperature: f64,
}

impl SmartThermometer {
    pub fn new(name: &str, temperature: f64) -> Self {
        Self {
            name: name.to_string(),
            temperature,
        }
    }

    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Report for SmartThermometer {
    fn report(&self) -> String {
        format!(
            "[Термометр '{}'] Температура: {:.1}°C",
            self.name(),
            self.temperature()
        )
    }
}

// Умная розетка
#[derive(Debug, Clone)]
pub struct SmartSocket {
    name: String,
    is_on: bool,
    power_watts: f64,
}

impl SmartSocket {
    pub fn new(name: &str, is_on: bool, power_watts: f64) -> Self {
        Self {
            name: name.to_string(),
            is_on,
            power_watts,
        }
    }

    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn current_power(&self) -> f64 {
        if self.is_on {
            self.power_watts
        } else {
            0.0
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Report for SmartSocket {
    fn report(&self) -> String {
        let state = if self.is_on() {
            "включена"
        } else {
            "выключена"
        };
        format!(
            "[Розетка '{}'] Состояние: {}, Мощность: {:.1} Вт",
            self.name(),
            state,
            self.current_power()
        )
    }
}

// Умное устройство (enum-обёртка)
#[derive(Debug, Clone)]
pub enum SmartDevice {
    Thermometer(SmartThermometer),
    Socket(SmartSocket),
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

impl Report for SmartDevice {
    fn report(&self) -> String {
        match self {
            SmartDevice::Thermometer(t) => t.report(),
            SmartDevice::Socket(s) => s.report(),
        }
    }
}

// Комната
#[derive(Debug, Clone)]
pub struct Room {
    name: String,
    devices: HashMap<String, SmartDevice>,
}

impl Room {
    pub fn new(name: &str, devices: HashMap<String, SmartDevice>) -> Self {
        Self {
            name: name.to_string(),
            devices,
        }
    }

    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            devices: HashMap::new(),
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

    pub fn add_device(
        &mut self,
        key: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Option<SmartDevice> {
        self.devices.insert(key.into(), device.into())
    }

    pub fn remove_device(&mut self, key: &str) -> Option<SmartDevice> {
        self.devices.remove(key)
    }
}

impl Report for Room {
    fn report(&self) -> String {
        let mut lines = vec![format!("  Комната: '{}'", self.name)];
        let mut keys: Vec<_> = self.devices.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(device) = self.devices.get(key) {
                lines.push(format!("    [{}] {}", key, device.report()));
            }
        }

        lines.join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartHomeError {
    RoomNotFound(String),
    DeviceNotFound { room: String, device: String },
}

impl fmt::Display for SmartHomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmartHomeError::RoomNotFound(room) => write!(f, "Комната '{}' не найдена", room),
            SmartHomeError::DeviceNotFound { room, device } => {
                write!(f, "Устройство '{}' не найдено в комнате '{}'", device, room)
            }
        }
    }
}

impl Error for SmartHomeError {}

// Умный дом
#[derive(Debug, Clone)]
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
    fn report(&self) -> String {
        let mut lines = vec![format!("=== Умный дом: '{}' ===", self.name)];
        let mut keys: Vec<_> = self.rooms.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(room) = self.rooms.get(key) {
                lines.push(format!("[{}]", key));
                lines.push(room.report());
            }
        }

        lines.push("==============================".to_string());
        lines.join("\n")
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
        let room = crate::room!(
            "Гостиная",
            ("socket_tv", SmartSocket::new("TV", true, 150.0)),
            ("thermo", SmartThermometer::new("T1", 22.5))
        );

        let mut home = SmartHome::empty("Дом");
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
        room.add_device("socket", SmartSocket::new("Чайник", true, 1200.0));
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

        let room = crate::room!("Спальня", ("thermo", SmartThermometer::new("Т", 19.0)));
        let mut home = SmartHome::empty("Дом");
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
        let room = crate::room!(
            "Кабинет",
            ("thermo", SmartThermometer::new("Т1", 21.2)),
            ("socket", SmartSocket::new("ПК", false, 500.0))
        );
        let mut home = SmartHome::empty("Дом");
        home.add_room("office", room.clone());

        let room_report = room.report();
        let home_report = home.report();

        assert!(room_report.contains("Комната"));
        assert!(home_report.contains("Умный дом"));
    }
}
