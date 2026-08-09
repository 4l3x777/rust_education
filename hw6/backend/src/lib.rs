pub mod handlers;
pub mod models;
pub mod state;

use axum::routing::{get, post};
use axum::Router;
use smart_home::{SmartHome, SmartSocket, SmartThermometer};
use tower_http::services::ServeDir;

pub use state::AppState;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/api/rooms", get(handlers::list_rooms).post(handlers::add_room))
        .route(
            "/api/rooms/:room",
            get(handlers::get_room).delete(handlers::remove_room),
        )
        .route(
            "/api/rooms/:room/devices",
            get(handlers::list_devices).post(handlers::add_device),
        )
        .route(
            "/api/rooms/:room/devices/:device",
            get(handlers::get_device).delete(handlers::remove_device),
        )
        .route(
            "/api/rooms/:room/devices/:device/turn_on",
            post(handlers::turn_on),
        )
        .route(
            "/api/rooms/:room/devices/:device/turn_off",
            post(handlers::turn_off),
        )
        .route("/api/report", get(handlers::get_report))
        .fallback_service(ServeDir::new("backend/static"))
        .with_state(state)
}

pub fn default_state() -> AppState {
    let mut home = SmartHome::empty("Мой умный дом");

    let mut living = smart_home::Room::empty("Гостиная");
    living.add_device("tv", SmartSocket::local("Телевизор", true, 150.0));
    living.add_device("temp1", SmartThermometer::local("Термометр", 22.5));

    let mut kitchen = smart_home::Room::empty("Кухня");
    kitchen.add_device("kettle", SmartSocket::local("Чайник", false, 1200.0));

    home.add_room("Гостиная", living);
    home.add_room("Кухня", kitchen);

    AppState::new(home)
}
