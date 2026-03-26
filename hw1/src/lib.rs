// Умный термометр
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

// Умная розетка
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

// Умное устройство (enum-обёртка)
pub enum SmartDevice {
    Thermometer(SmartThermometer),
    Socket(SmartSocket),
}

impl SmartDevice {
    pub fn print_status(&self) {
        match self {
            SmartDevice::Thermometer(t) => {
                println!(
                    "[Термометр '{}'] Температура: {:.1}°C",
                    t.name(),
                    t.temperature()
                );
            }
            SmartDevice::Socket(s) => {
                let state = if s.is_on() {
                    "включена"
                } else {
                    "выключена"
                };
                println!(
                    "[Розетка '{}'] Состояние: {}, Мощность: {:.1} Вт",
                    s.name(),
                    state,
                    s.current_power()
                );
            }
        }
    }
}

// Комната
pub struct Room {
    name: String,
    devices: Vec<SmartDevice>,
}

impl Room {
    pub fn new(name: &str, devices: Vec<SmartDevice>) -> Self {
        Self {
            name: name.to_string(),
            devices,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_device(&self, index: usize) -> &SmartDevice {
        if index >= self.devices.len() {
            panic!(
                "Индекс {} выходит за пределы массива устройств (размер: {})",
                index,
                self.devices.len()
            );
        }
        &self.devices[index]
    }

    pub fn get_device_mut(&mut self, index: usize) -> &mut SmartDevice {
        let len = self.devices.len();
        if index >= len {
            panic!(
                "Индекс {} выходит за пределы массива устройств (размер: {})",
                index, len
            );
        }
        &mut self.devices[index]
    }

    pub fn print_report(&self) {
        println!("  Комната: '{}'", self.name);
        for device in &self.devices {
            print!("    ");
            device.print_status();
        }
    }
}

// Умный дом
pub struct SmartHome {
    name: String,
    rooms: Vec<Room>,
}

impl SmartHome {
    pub fn new(name: &str, rooms: Vec<Room>) -> Self {
        Self {
            name: name.to_string(),
            rooms,
        }
    }

    pub fn get_room(&self, index: usize) -> &Room {
        if index >= self.rooms.len() {
            panic!(
                "Индекс {} выходит за пределы массива комнат (размер: {})",
                index,
                self.rooms.len()
            );
        }
        &self.rooms[index]
    }

    pub fn get_room_mut(&mut self, index: usize) -> &mut Room {
        let len = self.rooms.len();
        if index >= len {
            panic!(
                "Индекс {} выходит за пределы массива комнат (размер: {})",
                index, len
            );
        }
        &mut self.rooms[index]
    }

    pub fn print_report(&self) {
        println!("=== Умный дом: '{}' ===", self.name);
        for room in &self.rooms {
            room.print_report();
        }
        println!("==============================");
    }
}
