//! Компоновщик отчётов.
//!
//! `ReportComposer` собирает отчёты нескольких комнат и домов в одном выводе.
//! Dispatch статический — через `match` в `ReportItem`, без vtable.
//!
//! ```
//! use smart_home::report::ReportComposer;
//! use smart_home::{Room, SmartThermometer};
//!
//! let mut room = Room::empty("Гостиная");
//! room.add_device("t", SmartThermometer::local("T1", 22.0));
//!
//! let mut composer = ReportComposer::new("Общий отчёт");
//! composer.add(room);
//! composer.report();
//! ```

use crate::{Report, Room, SmartHome, SmartHomeError};

pub enum ReportItem {
    Room(Room),
    Home(SmartHome),
}

impl ReportItem {
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

    pub fn add<T: Into<ReportItem>>(&mut self, item: T) {
        self.items.push(item.into());
    }

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
