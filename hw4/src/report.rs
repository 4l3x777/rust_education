// # Компоновщик отчёта (Composite + статический полиморфизм)
//
// `ReportComposer<Items>` хранит все элементы в типизированном `Vec<ReportItem>`,
// где `ReportItem` — закрытый `enum` над всеми поддерживаемыми `Report`-типами.
//
// Публичный API остаётся дженериковым: `add<T: Into<ReportItem>>(item: T)`,
// поэтому со стороны вызывающего кода всё выглядит как статический полиморфизм.
//
// ```
// use smart_home::report::ReportComposer;
// use smart_home::{Room, SmartHome, SmartThermometer};
//
// let mut room = Room::empty("Гостиная");
// room.add_device("t", SmartThermometer::local("T1", 22.0));
//
// let mut composer = ReportComposer::new("Общий отчёт");
// composer.add(room);
// composer.report();
// ```

use crate::{Report, Room, SmartHome, SmartHomeError};

// Дискриминированное объединение всех типов, которые можно добавить в
// `ReportComposer`. Dispatch через `match` — чисто статический, без vtable.
pub enum ReportItem {
    Room(Room),
    Home(SmartHome),
}

impl ReportItem {
    // Делегирует вызов конкретному типу через `match` (статический dispatch).
    fn generate(&self) -> Result<String, SmartHomeError> {
        match self {
            ReportItem::Room(r) => r.report(),
            ReportItem::Home(h) => h.report(),
        }
    }
}

impl From<Room> for ReportItem {
    fn from(r: Room) -> Self {
        ReportItem::Room(r)
    }
}

impl From<SmartHome> for ReportItem {
    fn from(h: SmartHome) -> Self {
        ReportItem::Home(h)
    }
}

// Компоновщик отчётов. Хранит `Vec<ReportItem>`.
pub struct ReportComposer {
    title: String,
    items: Vec<ReportItem>,
}

impl ReportComposer {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    // Добавить элемент. `T: Into<ReportItem>` — статический полиморфизм:
    // компилятор выбирает нужный `From`-конвертер на этапе компиляции.
    pub fn add<T: Into<ReportItem>>(&mut self, item: T) {
        self.items.push(item.into());
    }

    // Вывести сводный отчёт в терминал.
    // Вызов `generate()` разрешается через `match` в `ReportItem` — без vtable.
    pub fn report(&self) {
        println!("\n{}", self.title);
        for (i, item) in self.items.iter().enumerate() {
            match item.generate() {
                Ok(text) => println!("{}", text),
                Err(e) => println!("  [элемент {}] ОШИБКА: {}", i, e),
            }
        }
    }
}