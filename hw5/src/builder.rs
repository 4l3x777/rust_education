//! Builder для SmartHome с type-state паттерном.
//!
//! Компилятор статически запрещает вызов `add_device` до добавления первой комнаты:
//! метод определён только на `SmartHomeBuilder<HasRoom>`.
//!
//! ```
//! use smart_home::builder::SmartHomeBuilder;
//! use smart_home::SmartSocket;
//!
//! let home = SmartHomeBuilder::new("Мой дом")
//!     .add_room("living", "Гостиная")
//!     .add_device("tv", SmartSocket::local("TV", true, 80.0))
//!     .add_room("kitchen", "Кухня")
//!     .add_device("kettle", SmartSocket::local("Чайник", false, 1500.0))
//!     .build();
//! ```

use crate::{Room, SmartDevice, SmartHome};
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct NoRoom;
pub struct HasRoom;

pub struct SmartHomeBuilder<State> {
    name: String,
    rooms: HashMap<String, Room>,
    current_room_key: Option<String>,
    _state: PhantomData<State>,
}

impl SmartHomeBuilder<NoRoom> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rooms: HashMap::new(),
            current_room_key: None,
            _state: PhantomData,
        }
    }

    pub fn add_room(
        mut self,
        key: impl Into<String>,
        display_name: impl Into<String>,
    ) -> SmartHomeBuilder<HasRoom> {
        let key = key.into();
        self.rooms
            .insert(key.clone(), Room::empty(&display_name.into()));
        SmartHomeBuilder {
            name: self.name,
            rooms: self.rooms,
            current_room_key: Some(key),
            _state: PhantomData,
        }
    }
}

impl SmartHomeBuilder<HasRoom> {
    pub fn add_room(mut self, key: impl Into<String>, display_name: impl Into<String>) -> Self {
        let key = key.into();
        self.rooms
            .insert(key.clone(), Room::empty(&display_name.into()));
        self.current_room_key = Some(key);
        self
    }

    pub fn add_device(
        mut self,
        device_key: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Self {
        let room_key = self
            .current_room_key
            .clone()
            .expect("внутренняя ошибка: current_room_key пуст в HasRoom");
        if let Some(room) = self.rooms.get_mut(&room_key) {
            room.add_device(device_key, device);
        }
        self
    }

    pub fn build(self) -> SmartHome {
        SmartHome::new(&self.name, self.rooms)
    }
}
