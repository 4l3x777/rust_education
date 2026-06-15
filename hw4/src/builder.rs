// # Builder для умного дома (type-state pattern)
//
// Состояния компилируются в типы — компилятор **статически** запрещает
// добавлять устройства до добавления первой комнаты.
//
// ```
// use smart_home::builder::SmartHomeBuilder;
// use smart_home::SmartSocket;
//
// let home = SmartHomeBuilder::new("Мой дом")
//     .add_room("living", "Гостиная")   // переходит в состояние HasRoom
//     .add_device("tv", SmartSocket::local("TV", true, 80.0))
//     .add_room("kitchen", "Кухня")
//     .add_device("kettle", SmartSocket::local("Чайник", false, 1500.0))
//     .build();
//
// println!("{}", home.name());
// ```

use crate::{Room, SmartDevice, SmartHome};
use std::collections::HashMap;
use std::marker::PhantomData;

// Типы-состояния

// Начальное состояние: комнаты ещё нет.
pub struct NoRoom;

// В билдере есть хотя бы одна комната — можно добавлять устройства.
pub struct HasRoom;

// Сам билдер

pub struct SmartHomeBuilder<State> {
    name: String,
    rooms: HashMap<String, Room>,
    current_room_key: Option<String>,
    _state: PhantomData<State>,
}

impl SmartHomeBuilder<NoRoom> {
    // Создать новый билдер. На этом этапе устройства добавлять нельзя.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rooms: HashMap::new(),
            current_room_key: None,
            _state: PhantomData,
        }
    }

    // Добавить первую комнату — переход в состояние `HasRoom`.
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
    // Добавить следующую комнату (уже в состоянии `HasRoom`).
    pub fn add_room(mut self, key: impl Into<String>, display_name: impl Into<String>) -> Self {
        let key = key.into();
        self.rooms
            .insert(key.clone(), Room::empty(&display_name.into()));
        self.current_room_key = Some(key);
        self
    }

    // Добавить устройство в **текущую** комнату.
    // Метод доступен **только** в состоянии `HasRoom` — гарантируется компилятором.
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

    // Собрать готовый `SmartHome`.
    pub fn build(self) -> SmartHome {
        SmartHome::new(&self.name, self.rooms)
    }
}
