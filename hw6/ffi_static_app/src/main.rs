// ffi_static_app — демонстрация статического связывания через Rust rlib.
//
// SmartSocket подключается напрямую через Rust-типы:
//   use ffi_smart_socket::SmartSocket;
// Весь код библиотеки встраивается в бинарник при линковке.
//
// Запуск: cargo run -p ffi_static_app

use ffi_smart_socket::SmartSocket;

fn main() {
    println!("=== ffi_static_app: статическое связывание (rlib) ===");

    let mut socket = SmartSocket::local("Розетка (rlib)", false, 1200.0);

    println!(
        "Начальное состояние: включена={:?}, мощность={:?}",
        socket.is_on(),
        socket.current_power()
    );

    socket.turn_on().expect("turn_on failed");
    println!(
        "После включения:    включена={:?}, мощность={:?}",
        socket.is_on(),
        socket.current_power()
    );

    socket.turn_off().expect("turn_off failed");
    println!(
        "После выключения:   включена={:?}, мощность={:?}",
        socket.is_on(),
        socket.current_power()
    );

    println!("\nГотово. Розетка '{}' освобождена автоматически (Drop).", socket.name());
}
