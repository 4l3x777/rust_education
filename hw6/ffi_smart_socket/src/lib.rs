// ffi_smart_socket — C ABI wrapper для SmartSocket.
//
// Три артефакта одной сборкой (crate-type = ["rlib", "staticlib", "cdylib"]):
//   rlib        — Rust-to-Rust статическое связывание (static_app)
//   staticlib   — .a / .lib для C/C++ проектов
//   cdylib      — .so / .dll для runtime dlopen (dynamic_app)
//
// C ABI (6 функций):
//   void*  socket_create (const char* name, int is_on, double power_watts)
//   int    socket_turn_on (void* socket)       // 0=ok, -1=null
//   int    socket_turn_off(void* socket)        // 0=ok, -1=null
//   int    socket_is_on   (const void* socket)  // 1/0/-1=null
//   double socket_power   (const void* socket)  // Вт или -1.0=null/err
//   void   socket_destroy (void* socket)        // NULL-safe

pub use smart_home::SmartSocket;

use std::ffi::CStr;
use std::os::raw::{c_char, c_double, c_int};

// Коды возврата C ABI
const OK: c_int = 0;
const ERR_NULL: c_int = -1;
const ERR_OP: c_int = -2;
const ERR_POWER: c_double = -1.0;

// Разыменовать *mut T после проверки на null. Возвращает ERR_NULL при null.
macro_rules! deref_mut {
    ($ptr:expr) => {{
        if $ptr.is_null() {
            return ERR_NULL;
        }
        unsafe { &mut *$ptr }
    }};
}

// Разыменовать *const T после проверки на null.
macro_rules! deref_ref {
    ($ptr:expr, $sentinel:expr) => {{
        if $ptr.is_null() {
            return $sentinel;
        }
        unsafe { &*$ptr }
    }};
}

// Создать SmartSocket на куче и вернуть непрозрачный указатель.
// Вызывающая сторона владеет памятью — освободить через socket_destroy.
#[no_mangle]
pub extern "C" fn socket_create(
    name: *const c_char,
    is_on: c_int,
    power_watts: c_double,
) -> *mut SmartSocket {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name_str = unsafe {
        match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    Box::into_raw(Box::new(SmartSocket::local(name_str, is_on != 0, power_watts)))
}

// Включить розетку. Возвращает OK=0, ERR_NULL=-1, ERR_OP=-2.
#[no_mangle]
pub extern "C" fn socket_turn_on(socket: *mut SmartSocket) -> c_int {
    let s = deref_mut!(socket);
    match s.turn_on() {
        Ok(_) => OK,
        Err(_) => ERR_OP,
    }
}

// Выключить розетку. Возвращает OK=0, ERR_NULL=-1, ERR_OP=-2.
#[no_mangle]
pub extern "C" fn socket_turn_off(socket: *mut SmartSocket) -> c_int {
    let s = deref_mut!(socket);
    match s.turn_off() {
        Ok(_) => OK,
        Err(_) => ERR_OP,
    }
}

// Узнать состояние. Возвращает 1=включена, 0=выключена, ERR_NULL=-1.
#[no_mangle]
pub extern "C" fn socket_is_on(socket: *const SmartSocket) -> c_int {
    let s = deref_ref!(socket, ERR_NULL);
    match s.is_on() {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => ERR_OP,
    }
}

// Запросить мощность (Вт). Возвращает ERR_POWER=-1.0 при null или ошибке.
#[no_mangle]
pub extern "C" fn socket_power(socket: *const SmartSocket) -> c_double {
    let s = deref_ref!(socket, ERR_POWER);
    s.current_power().unwrap_or(ERR_POWER)
}

// Освободить память. NULL-safe.
#[no_mangle]
pub extern "C" fn socket_destroy(socket: *mut SmartSocket) {
    if !socket.is_null() {
        unsafe { drop(Box::from_raw(socket)) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn make_socket(name: &str, is_on: bool, power: f64) -> *mut SmartSocket {
        let cname = CString::new(name).unwrap();
        socket_create(cname.as_ptr(), is_on as c_int, power)
    }

    #[test]
    fn create_and_destroy() {
        let s = make_socket("Тест", false, 100.0);
        assert!(!s.is_null());
        socket_destroy(s);
    }

    #[test]
    fn destroy_null_is_safe() {
        socket_destroy(std::ptr::null_mut());
    }

    #[test]
    fn null_ptr_returns_sentinel() {
        assert_eq!(socket_turn_on(std::ptr::null_mut()), ERR_NULL);
        assert_eq!(socket_turn_off(std::ptr::null_mut()), ERR_NULL);
        assert_eq!(socket_is_on(std::ptr::null()), ERR_NULL);
        assert_eq!(socket_power(std::ptr::null()), ERR_POWER);
    }

    #[test]
    fn turn_on_off_round_trip() {
        let s = make_socket("Розетка", false, 1500.0);
        assert!(!s.is_null());

        assert_eq!(socket_is_on(s), 0);
        assert_eq!(socket_turn_on(s), OK);
        assert_eq!(socket_is_on(s), 1);
        assert!((socket_power(s) - 1500.0).abs() < 0.001);

        assert_eq!(socket_turn_off(s), OK);
        assert_eq!(socket_is_on(s), 0);
        assert!((socket_power(s) - 0.0).abs() < 0.001);

        socket_destroy(s);
    }

    #[test]
    fn create_with_null_name_returns_null() {
        assert!(socket_create(std::ptr::null(), 0, 100.0).is_null());
    }

    #[test]
    fn rust_api_direct() {
        let mut sock = SmartSocket::local("Прямой", true, 200.0);
        assert!(sock.is_on().unwrap());
        sock.turn_off().unwrap();
        assert!(!sock.is_on().unwrap());
        assert!((sock.current_power().unwrap()).abs() < 0.001);
    }
}
