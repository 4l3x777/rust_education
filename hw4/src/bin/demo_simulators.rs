// Пример умного дома с розетками и термометрами,
// работающими через имитаторы (TCP / UDP).
//
// Запуск (в отдельных терминалах):
//   cargo run --bin socket_sim   -- 127.0.0.1:7878
//   cargo run --bin socket_sim   -- 127.0.0.1:7879
//   cargo run --bin thermometer_sim -- thermometer_sim.conf   (target_addr=127.0.0.1:9001)
//   cargo run --bin thermometer_sim -- thermometer_sim2.conf  (target_addr=127.0.0.1:9002)
//   cargo run --bin demo_simulators

use smart_home::{
    devices::socket::SmartSocket, devices::thermometer::SmartThermometer, Report, Room, SmartHome,
};
use std::thread;
use std::time::Duration;

fn main() {
    // Гостиная
    // Розетка — TCP-соединение с имитатором
    let tv_socket = SmartSocket::tcp("Розетка-TV", "127.0.0.1:7878");
    // Термометр — UDP, слушает на порту 9001
    let thermometer1 = match SmartThermometer::udp("Термометр-Гостиная", "0.0.0.0:9001")
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Не удалось создать термометр: {}", e);
            std::process::exit(1);
        }
    };

    let mut living = Room::empty("Гостиная");
    living.add_device("tv_socket", tv_socket);
    living.add_device("thermometer", thermometer1);

    // Кухня
    let kettle_socket = SmartSocket::tcp("Розетка-чайник", "127.0.0.1:7879");
    let thermometer2 = match SmartThermometer::udp("Термометр-Кухня", "0.0.0.0:9002")
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Не удалось создать термометр: {}", e);
            std::process::exit(1);
        }
    };

    let mut kitchen = Room::empty("Кухня");
    kitchen.add_device("kettle_socket", kettle_socket);
    kitchen.add_device("thermometer", thermometer2);

    // Собираем дом
    let mut home = SmartHome::empty("Умный дом с имитаторами");
    home.add_room("living", living);
    home.add_room("kitchen", kitchen);

    // Ждём, пока имитаторы пришлют первые данные
    println!("Ожидание данных от имитаторов (2 секунды)...");
    thread::sleep(Duration::from_secs(2));

    // Включаем розетки через TCP
    for (room_key, device_key) in &[("living", "tv_socket"), ("kitchen", "kettle_socket")] {
        if let Some(room) = home.get_room_mut(room_key) {
            if let Some(smart_home::SmartDevice::Socket(socket)) = room.get_device_mut(device_key) {
                match socket.turn_on() {
                    Ok(_) => println!("Розетка '{}' включена", socket.name()),
                    Err(e) => println!("Не удалось включить '{}': {}", socket.name(), e),
                }
            }
        }
    }

    // Отчёт о состоянии дома
    println!("\n=== Отчёт о состоянии дома ===");
    match home.report() {
        Ok(text) => println!("{}", text),
        Err(e) => println!("Ошибка формирования отчёта: {}", e),
    }

    // Ждём, ещё пока имитаторы пришлют первые данные
    println!("Ожидание новых данных от имитаторов (2 секунды)...");
    thread::sleep(Duration::from_secs(2));

    // Новый отчёт о состоянии дома
    println!("\n=== Новый отчёт о состоянии дома ===");
    match home.report() {
        Ok(text) => println!("{}", text),
        Err(e) => println!("Ошибка формирования отчёта: {}", e),
    }
}
