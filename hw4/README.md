# hw4

- Реализовать билдер для умного дома, позволяющий инициализировать объект умного дома в стиле <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=5d0527e4684f726d54dc375829d983f4>.

- До добавления первой комнаты, билдер запрещает добавлять устройства.

- Это должно контролироваться компилятором.

- Реализовать компоновщик для построения отчёта об объектах умного дома в стиле: <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c07dfc726e8ccbccdcc2d88a79d3f190>.

- Использовать статический полиморфизм (дженерики).

- Вызов метода report() должен выводить в терминал отчёт обо всех добавленных объектах.

- Добавить возможность добавления callback-ов в объект комнаты, которые срабатывают при добавлении новых устройств в комнату (паттерн Observer).

- Использовать динамический полиморфизм (трейт-объекты).

- Можно передавать как объект-subscriber, так и замыкание: <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=06e9dc9bcce297d1e80a22d7e9338ee8>.

- Добавить example-ы, демонстрирующие новый функционал.

## Запуск

### Базовый пример (без сети)

```bash
cargo run --bin local_smart_home
```

### Пример с имитаторами

Запустите в отдельных терминалах:

```bash
# Имитатор розетки — гостиная
cargo run --bin socket_sim -- 127.0.0.1:7878

# Имитатор розетки — кухня
cargo run --bin socket_sim -- 127.0.0.1:7879

# Имитатор термометра — гостиная
cargo run --bin thermometer_sim -- thermometer_sim.conf

# Имитатор термометра — кухня
cargo run --bin thermometer_sim -- thermometer_sim2.conf

# Демо-пример
cargo run --bin simulators_smart_home
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