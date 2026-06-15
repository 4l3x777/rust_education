// # Компоновщик отчёта (Composite + статический полиморфизм)
//
// `ReportComposer` собирает любые объекты, реализующие трейт `Report`,
// и при вызове `.report()` выводит сводный отчёт в терминал.
//
// Используется **статический полиморфизм** через дженерики: каждый узел
// `Leaf<T: Report>` хранит конкретный тип, поэтому вызовы инлайнятся.
//
// ```
// use smart_home::report::ReportComposer;
// use smart_home::{Room, SmartThermometer};
//
// let mut room = Room::empty("Гостиная");
// room.add_device("t", SmartThermometer::local("T1", 22.0));
//
// let mut composer = ReportComposer::new("Общий отчёт");
// composer.add(room);
// composer.report();
// ```

use crate::{Report, SmartHomeError};

// Трейт-объект внутри компоновщика.
trait ReportNode {
    fn generate(&self) -> Result<String, SmartHomeError>;
}

// Обёртка над конкретным `T: Report` — статический полиморфизм (generic).
struct Leaf<T: Report> {
    inner: T,
}

impl<T: Report> ReportNode for Leaf<T> {
    fn generate(&self) -> Result<String, SmartHomeError> {
        self.inner.report()
    }
}

// Компоновщик: хранит список узлов и объединяет их отчёты.
pub struct ReportComposer {
    title: String,
    nodes: Vec<Box<dyn ReportNode>>,
}

impl ReportComposer {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            nodes: Vec::new(),
        }
    }

    // Добавить любой объект, реализующий `Report` (дженерик на уровне метода).
    pub fn add<T: Report + 'static>(&mut self, item: T) {
        self.nodes.push(Box::new(Leaf { inner: item }));
    }

    // Вывести сводный отчёт в терминал.
    pub fn report(&self) {
        println!("\n{}", self.title);
        for (i, node) in self.nodes.iter().enumerate() {
            match node.generate() {
                Ok(text) => println!("{}", text),
                Err(e) => println!("  [узел {}] ОШИБКА: {}", i, e),
            }
        }
    }
}
