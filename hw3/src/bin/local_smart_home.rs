// Демонстрация использования умного дома с динамическим управлением комнатами и устройствами
//
// Запуск: 
//  cargo run --bin local_smart_home
//
// Этот пример показывает, как создавать комнаты и устройства, а также управлять ими через типизированный интерфейс.
// В этом примере мы создаём умный дом с двумя комнатами: гостиной и спальней. В каждой комнате есть термометр и розетка.
// Мы также демонстрируем динамическое добавление и удаление устройств, а также обработку ошибок при доступе к несуществующим устройствам и комнатам.
// В отличие от simulators_smart_home, здесь мы используем локальные устройства, а не имитаторы через сеть.

use smart_home::{room, Report, SmartDevice, SmartHome, SmartSocket, SmartThermometer};

fn print_any_report(item: &impl Report) {
    match item.report() {
        Ok(text) => println!("{}", text),
        Err(e) => println!("ОШИБКА получения отчёта: {}", e),
    }
}

fn main() {
    // Создание первой комнаты с набором устройств
    let living_room = room!(
        "Гостиная",
        ("thermometer", SmartThermometer::local("Термометр-1", 22.5)),
        ("tv_socket", SmartSocket::local("Розетка-TV", true, 150.0))
    );

    // Создание второй комнаты с другим набором устройств
    let bedroom = room!(
        "Спальня",
        ("lamp_socket", SmartSocket::local("Розетка-лампа", true, 60.0)),
        ("thermometer", SmartThermometer::local("Термометр-2", 20.0))
    );

    // Создание умного дома и добавление комнат динамически через типизированный интерфейс
    let mut home = SmartHome::empty("Мой дом");

    // Динамическое добавление комнат в дом
    home.add_room("living", living_room);
    home.add_room("bedroom", bedroom);

    println!(">>> Отчёт о доме после добавления комнат:");
    print_any_report(&home);

    // Динамическое добавление и удаление устройства в комнате
    if let Some(room) = home.get_room_mut("living") {
        room.add_device(
            "heater_socket",
            SmartSocket::local("Розетка-обогреватель", false, 2000.0),
        );
        room.remove_device("heater_socket");
    }

    // Динамическое управление устройством через типизированный интерфейс
    match home.get_device("living", "tv_socket") {
        Ok(SmartDevice::Socket(socket)) => {
            println!("\nВыключаем {}", socket.name());
        }
        Ok(_) => println!("Найдено устройство другого типа"),
        Err(err) => println!("Ошибка доступа к устройству: {err}"),
    }

    // Динамическое изменение состояния устройства через типизированный интерфейс
    if let Some(room) = home.get_room_mut("living") {
        if let Some(SmartDevice::Socket(socket)) = room.get_device_mut("tv_socket") {
            socket.turn_off().ok();
        }
    }

    // Демонстрация обработки ошибок при доступе к несуществующему устройству
    if let Err(err) = home.get_device("kitchen", "socket") {
        println!("\nОжидаемая ошибка: {err}");
    }

    // Динамическое удаление комнаты
    home.remove_room("bedroom");

    println!("\n>>> Отчёт о доме после изменений:");
    print_any_report(&home);

    // Отчёт о комнате и устройстве через универсальную функцию
    if let Some(room) = home.get_room("living") {
        println!("\n>>> Отчёт по комнате:");
        print_any_report(room);

        if let Some(device) = room.get_device("tv_socket") {
            println!("\n>>> Отчёт по устройству:");
            print_any_report(device);
        }
    }
}
