# hw6

## Задание hw6
Реализовать backend сервис для управления умным домом и frontend приложение для взаимодействия с ним.

Технология взаимодействия — REST API (axum + tokio).

API backend сервиса предоставляет доступ ко всему базовому функционалу библиотеки умного дома:

- Добавление/удаление/перечисление комнат в доме и получение информации о конкретной комнате.
- Добавление/удаление/перечисление устройств в комнате и получение информации о конкретном устройстве.
- Получение отчёта о доме.
- Присутствуют функциональные тесты, которые общаются с backend-ом и проверяют его ответы.

Frontend приложение:

- Отображает список комнат в доме.
- Позволяет перейти к конкретной комнате или добавить новую комнату.
- Отображает список устройств в комнате.
- Позволяет перейти к конкретному устройству или добавить новое устройство.
- Позволяет запросить отчёт о состоянии дома.

## Backend

Crate `backend` — REST API на axum, раздаёт статические файлы frontend-а.

*REST endpoints*:
```
GET    /api/rooms                              — список комнат
POST   /api/rooms                              — добавить комнату
GET    /api/rooms/:room                        — информация о комнате
DELETE /api/rooms/:room                        — удалить комнату
GET    /api/rooms/:room/devices                — список устройств в комнате
POST   /api/rooms/:room/devices                — добавить устройство
GET    /api/rooms/:room/devices/:device        — информация об устройстве
DELETE /api/rooms/:room/devices/:device        — удалить устройство
POST   /api/rooms/:room/devices/:device/turn_on  — включить розетку
POST   /api/rooms/:room/devices/:device/turn_off — выключить розетку
GET    /api/report                             — отчёт о доме
```

Типы устройств: `socket` (розетка) и `thermometer` (термометр).

Тесты (19 функциональных) покрывают полный CRUD комнат и устройств, 404-ошибки,
валидацию входных данных, включение/выключение розеток, отчёты и сквозной workflow.

## Запуск

### Backend сервер (REST API + frontend)

```bash
cargo run -p backend
```

Сервер доступен на `http://127.0.0.1:3000`.

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
