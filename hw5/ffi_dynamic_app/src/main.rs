// ffi_dynamic_app — демонстрация runtime dlopen через libloading.
//
// Загружает cdylib (libffi_smart_socket.so / libffi_smart_socket.dll) во время выполнения.
// Путь к библиотеке передаётся первым аргументом CLI или определяется
// автоматически рядом с исполняемым файлом.
//
// Запуск:
//   cargo build -p ffi_smart_socket
//   cargo run -p ffi_dynamic_app
//   # или явно:
//   cargo run -p ffi_dynamic_app -- target/debug/libffi_smart_socket.so

use libloading::{Library, Symbol};
use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int};
use std::path::PathBuf;

// Типы символов C ABI
type FnCreate  = unsafe extern "C" fn(*const c_char, c_int, c_double) -> *mut u8;
type FnTurnOn  = unsafe extern "C" fn(*mut u8) -> c_int;
type FnTurnOff = unsafe extern "C" fn(*mut u8) -> c_int;
type FnIsOn    = unsafe extern "C" fn(*const u8) -> c_int;
type FnPower   = unsafe extern "C" fn(*const u8) -> c_double;
type FnDestroy = unsafe extern "C" fn(*mut u8);

// Имена экспортированных символов
const SYM_CREATE:   &[u8] = b"socket_create\0";
const SYM_TURN_ON:  &[u8] = b"socket_turn_on\0";
const SYM_TURN_OFF: &[u8] = b"socket_turn_off\0";
const SYM_IS_ON:    &[u8] = b"socket_is_on\0";
const SYM_POWER:    &[u8] = b"socket_power\0";
const SYM_DESTROY:  &[u8] = b"socket_destroy\0";

fn default_lib_path() -> PathBuf {
    let mut path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    #[cfg(target_os = "windows")]
    path.push("ffi_smart_socket.dll");
    #[cfg(target_os = "macos")]
    path.push("libffi_smart_socket.dylib");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    path.push("libffi_smart_socket.so");

    path
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_lib_path);

    println!("=== ffi_dynamic_app: runtime dlopen ===");
    println!("Загружаем: {}", path.display());

    let lib = unsafe {
        Library::new(&path).unwrap_or_else(|e| {
            eprintln!("Ошибка загрузки '{}': {}", path.display(), e);
            eprintln!("Подсказка: сначала выполните `cargo build -p ffi_smart_socket`");
            std::process::exit(1);
        })
    };

    unsafe {
        let create:   Symbol<FnCreate>  = lib.get(SYM_CREATE).unwrap();
        let turn_on:  Symbol<FnTurnOn>  = lib.get(SYM_TURN_ON).unwrap();
        let turn_off: Symbol<FnTurnOff> = lib.get(SYM_TURN_OFF).unwrap();
        let is_on:    Symbol<FnIsOn>    = lib.get(SYM_IS_ON).unwrap();
        let power:    Symbol<FnPower>   = lib.get(SYM_POWER).unwrap();
        let destroy:  Symbol<FnDestroy> = lib.get(SYM_DESTROY).unwrap();

        let name = CString::new("Розетка (dlopen)").unwrap();
        let socket = create(name.as_ptr(), 0, 800.0);

        if socket.is_null() {
            eprintln!("socket_create вернул NULL");
            std::process::exit(1);
        }

        println!("Создана розетка через dlopen");
        println!("  is_on = {} (0=выкл)", is_on(socket));
        println!("  power = {:.1} Вт (0.0 — выключена)", power(socket));

        println!("Включаем...");
        println!("  turn_on => {} (0=ok)", turn_on(socket));
        println!("  is_on   = {} (1=вкл)", is_on(socket));
        println!("  power   = {:.1} Вт", power(socket));

        println!("Выключаем...");
        turn_off(socket);
        println!("  is_on = {} (0=выкл)", is_on(socket));
        println!("  power = {:.1} Вт", power(socket));

        println!("\nПроверка NULL-safety:");
        println!("  turn_on(NULL)  = {} (-1=null)", turn_on(std::ptr::null_mut()));
        println!("  is_on(NULL)    = {} (-1=null)", is_on(std::ptr::null()));
        println!("  power(NULL)    = {:.1} (-1.0=null)", power(std::ptr::null()));
        destroy(std::ptr::null_mut());
        println!("  destroy(NULL)  — OK");

        destroy(socket);
        println!("\nРозетка уничтожена. Готово.");
    }
}
