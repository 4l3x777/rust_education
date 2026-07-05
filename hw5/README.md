# hw5 

## Задание hw5
Реализовать workspace со следующими package-ами:

- Библиотека умной розетки с Си ABI.
- Приложение, использующее библиотеку умной розетки, линкуя её статически.
- Приложение, использующее библиотеку умной розетки, линкуя её динамически в runtime.

Библиотека умной розетки с Си ABI:

- Функционал не изменяется: включение/выключение + запрос мощности.

При сборке создаёт три артефакта:

- Rust библиотеку
- Статическую библиотеку с Си ABI.
- Динамическую библиотеку с Си ABI
- Пакеты-приложения должны демонстрировать функционал умной библиотеки.


*C ABI* (6 функций с `#[no_mangle]` + `extern "C"`):
```c
void*  socket_create (const char* name, int is_on, double power_watts);
int    socket_turn_on (void* socket);      // 0=ok, -1=null ptr
int    socket_turn_off(void* socket);
int    socket_is_on   (const void* socket); // 1/0/-1
double socket_power   (const void* socket); // Вт или -1.0
void   socket_destroy (void* socket);       // NULL-safe
```

Тесты покрывают Rust API, полный C ABI round-trip и NULL-safety.

## Запуск

### Базовый пример (без сети)

```bash
cargo run -p smart_home --bin local_smart_home
```

### Пример с имитаторами

```bash
# В отдельных терминалах:
cargo run -p smart_home --bin socket_sim -- 127.0.0.1:7878
cargo run -p smart_home --bin socket_sim -- 127.0.0.1:7879
cargo run -p smart_home --bin thermometer_sim -- thermometer_sim.conf
cargo run -p smart_home --bin thermometer_sim -- thermometer_sim2.conf

# Демо-пример
cargo run -p smart_home --bin simulators_smart_home
```

### FFI — статическое связывание

```bash
cargo run -p ffi_static_app
# или C static linking:
cargo build -p ffi_smart_socket
cargo run -p ffi_c_static_app
```

### FFI — динамическое связывание (runtime dlopen)

```bash
cargo build -p ffi_smart_socket
cargo run -p ffi_dynamic_app
# или явно:
cargo run -p ffi_dynamic_app -- target/debug/libffi_smart_socket.(so/dll/dylib)
```


### Тесты

```bash
cargo test
```

### Примеры

```bash
cargo run --example builder_demo
cargo run --example report_composer_demo
cargo run --example observer_demo
```
