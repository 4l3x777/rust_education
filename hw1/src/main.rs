use smart_home::{Room, SmartDevice, SmartHome, SmartSocket, SmartThermometer};

fn main() {
    // Создаём устройства для гостиной
    let thermometer = SmartDevice::Thermometer(SmartThermometer::new("Термометр-1", 22.5));
    let socket1 = SmartDevice::Socket(SmartSocket::new("Розетка-TV", true, 150.0));

    // Создаём устройства для спальни
    let socket2 = SmartDevice::Socket(SmartSocket::new("Розетка-лампа", true, 60.0));
    let thermometer2 = SmartDevice::Thermometer(SmartThermometer::new("Термометр-2", 20.0));

    // Создаём комнаты
    let living_room = Room::new("Гостиная", vec![thermometer, socket1]);
    let bedroom = Room::new("Спальня", vec![socket2, thermometer2]);

    // Создаём умный дом
    let mut home = SmartHome::new("Мой дом", vec![living_room, bedroom]);

    println!(">>> Отчёт ДО выключения розетки:");
    home.print_report();

    // Выключаем розетку в гостиной (индекс комнаты 0, устройство 1)
    let room = home.get_room_mut(0);
    if let SmartDevice::Socket(socket) = room.get_device_mut(1) {
        socket.turn_off();
    }

    println!();
    println!(">>> Отчёт ПОСЛЕ выключения 'Розетка-TV' в Гостиной:");
    home.print_report();
}
