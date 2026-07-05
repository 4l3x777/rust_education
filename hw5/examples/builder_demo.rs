// Демонстрация **Builder** (type-state).
//
// Запуск: `cargo run --example builder_demo`

use smart_home::{builder::SmartHomeBuilder, Report, SmartSocket, SmartThermometer};

fn main() {
    // Строим дом через билдер.
    // До вызова add_room компилятор не даст вызвать add_device:
    //   SmartHomeBuilder::new("X").add_device(...) - ошибка компиляции!
    let home = SmartHomeBuilder::new("Мой умный дом")
        .add_room("living", "Гостиная")
        .add_device("tv", SmartSocket::local("Телевизор", true, 80.0))
        .add_device("lamp", SmartSocket::local("Торшер", true, 40.0))
        .add_room("bedroom", "Спальня")
        .add_device("thermo", SmartThermometer::local("Термометр", 20.5))
        .add_device("ac", SmartSocket::local("Кондиционер", false, 2000.0))
        .add_room("kitchen", "Кухня")
        .add_device("kettle", SmartSocket::local("Чайник", false, 1500.0))
        .build();

    println!("Дом '{}' построен.", home.name());
    println!();
    match home.report() {
        Ok(r) => println!("{}", r),
        Err(e) => eprintln!("Ошибка: {}", e),
    }
}
