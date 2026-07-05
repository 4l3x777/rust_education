// Демонстрация **Observer** (динамический полиморфизм).
//
// Запуск: `cargo run --example observer_demo`

use smart_home::{DeviceObserver, Room, SmartDevice, SmartSocket, SmartThermometer};

// Пример 1: подписчик-структура

#[derive(Debug)]
struct AuditLog {
    records: Vec<String>,
}

impl AuditLog {
    fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
}

impl DeviceObserver for AuditLog {
    fn on_device_added(&mut self, room: &str, key: &str, device: &SmartDevice) {
        let kind = match device {
            SmartDevice::Socket(_) => "розетка",
            SmartDevice::Thermometer(_) => "термометр",
        };
        let msg = format!(
            "[AuditLog] комната='{}', устройство='{}', тип={}",
            room, key, kind
        );
        println!("{}", msg);
        self.records.push(msg);
    }
}

fn main() {
    // объект-подписчик
    let mut room = Room::empty("Гостиная");

    room.subscribe(AuditLog::new());

    // замыкание в качестве подписчика
    room.subscribe_fn(|room_name, key, _dev| {
        println!("[замыкание] добавлено в '{}': '{}'", room_name, key);
    });

    println!("--- Добавляем устройства ---");
    room.add_device("tv", SmartSocket::local("Телевизор", true, 80.0));
    room.add_device("thermo", SmartThermometer::local("Термометр", 21.7));
    room.add_device("lamp", SmartSocket::local("Торшер", false, 40.0));

    // ещё одна комната с замыканием
    println!("\n--- Кухня (только замыкание) ---");
    let mut kitchen = Room::empty("Кухня");
    kitchen.subscribe_fn(|room_name, key, _dev| {
        println!("  >> кухня: '{}' / '{}'", room_name, key);
    });
    kitchen.add_device("kettle", SmartSocket::local("Чайник", false, 1500.0));
}
