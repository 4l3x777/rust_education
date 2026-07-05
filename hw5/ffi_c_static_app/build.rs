//! Компиляция main.c и линковка с libffi_smart_socket.a.

use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Компилируем main.c -> main_c.o
    let compiler = cc::Build::new()
        .file("src/main.c")
        .include("include")
        .get_compiler();

    let obj = out_dir.join("main_c.o");

    let status = compiler
        .to_command()
        .arg("-c")
        .arg("-Iinclude")
        .arg("-std=c11")
        .arg("-Wall")
        .arg("-D__USE_MINGW_ANSI_STDIO=0")
        .arg("src/main.c")
        .arg("-o").arg(&obj)
        .status()
        .expect("failed to run C compiler");

    assert!(status.success(), "C compilation failed");

    // OUT_DIR = target/<profile>/build/<crate>/out  => 3 уровня вверх = target/<profile>
    let profile_dir = out_dir.ancestors().nth(3).unwrap().to_path_buf();
    let ffi_lib = profile_dir.join("libffi_smart_socket.a");

    // Линкуем в правильном порядке: сначала объекты, потом библиотеки.
    // GNU ld разрешает символы слева направо, поэтому msvcrt должна идти после .o
    println!("cargo:rustc-link-arg={}", obj.display());
    println!("cargo:rustc-link-arg={}", ffi_lib.display());
    println!("cargo:rustc-link-arg=-lmsvcrt");
    println!("cargo:rustc-link-arg=-lm");

    println!("cargo:rerun-if-changed=src/main.c");
    println!("cargo:rerun-if-changed=include/ffi_smart_socket.h");
    println!("cargo:rerun-if-changed=build.rs");
}
