// main.rs — Rust entry point для ffi_c_static_app.
// Вся логика — в src/main.c.
// Rust вызывает c_main() через FFI и пробрасывает exit-код.

use std::os::raw::c_int;

unsafe extern "C" {
    fn c_main() -> c_int;
}

fn main() {
    let code = unsafe { c_main() };
    std::process::exit(code);
}
