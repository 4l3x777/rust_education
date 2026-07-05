// Демонстрация **Компоновщика отчёта** (статический полиморфизм).
//
// Запуск: `cargo run --example report_composer_demo`

use smart_home::{report::ReportComposer, Room, SmartHome, SmartSocket, SmartThermometer};

fn main() {
    // Создаём несколько независимых объектов
    let mut living = Room::empty("Гостиная");
    living.add_device("tv", SmartSocket::local("Телевизор", true, 80.0));
    living.add_device("thermo", SmartThermometer::local("Термометр", 22.3));

    let mut kitchen = Room::empty("Кухня");
    kitchen.add_device("kettle", SmartSocket::local("Чайник", false, 1500.0));

    let mut home = SmartHome::empty("Коттедж");
    let mut cottage_living = Room::empty("Зал коттеджа");
    cottage_living.add_device("fireplace", SmartSocket::local("Камин", true, 3000.0));
    home.add_room("hall", cottage_living);

    // Компоновщик принимает любой тип, реализующий Report
    let mut composer = ReportComposer::new("Полный отчёт по всем объектам");
    composer.add(living); // Room
    composer.add(kitchen); // Room
    composer.add(home); // SmartHome

    // Единственный вызов — выводит все отчёты сразу
    composer.report();
}
