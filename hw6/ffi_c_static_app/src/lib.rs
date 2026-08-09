// lib.rs — пустая точка входа для Cargo.
// Вся логика — в src/main.c, компилируемом через build.rs (cc crate).
// Rust-бинарник служит лишь оберткой, которая вызывает C main().

use std::os::raw::c_int;

extern "C" {
    // C-функция main из src/main.c
    fn main() -> c_int;
}

pub fn run_c_main() -> i32 {
    unsafe { main() as i32 }
}
