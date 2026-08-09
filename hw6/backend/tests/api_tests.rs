use backend::{create_app, AppState};
use reqwest::{StatusCode};
use smart_home::SmartHome;
use serde_json::{json, Value};
use tokio::net::TcpListener;

async fn spawn_server() -> String {
    let state = AppState::new(SmartHome::empty("Тестовый дом"));
    let app = create_app(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    format!("http://{}", addr)
}

async fn spawn_server_with_demo() -> String {
    let mut home = SmartHome::empty("Демо-дом");
    let mut room = smart_home::Room::empty("Спальня");
    room.add_device("lamp", smart_home::SmartSocket::local("Лампа", true, 60.0));
    room.add_device("temp", smart_home::SmartThermometer::local("Термометр", 21.0));
    home.add_room("Спальня", room);

    let state = AppState::new(home);
    let app = create_app(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    format!("http://{}", addr)
}

// ---- Room tests ------------------------------------------------------------

#[tokio::test]
async fn list_empty_rooms() {
    let base = spawn_server().await;
    let res = reqwest::get(format!("{}/api/rooms", base)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let rooms: Vec<Value> = res.json().await.unwrap();
    assert!(rooms.is_empty());
}

#[tokio::test]
async fn add_room_and_list() {
    let base = spawn_server().await;
    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms", base))
        .json(&json!({ "name": "Гостиная" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = reqwest::get(format!("{}/api/rooms", base)).await.unwrap();
    let rooms: Vec<Value> = res.json().await.unwrap();
    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0]["name"], "Гостиная");
    assert_eq!(rooms[0]["device_count"], 0);
}

#[tokio::test]
async fn add_room_empty_name_rejected() {
    let base = spawn_server().await;
    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms", base))
        .json(&json!({ "name": "  " }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_room_info() {
    let base = spawn_server_with_demo().await;
    let res = reqwest::get(format!("{}/api/rooms/Спальня", base)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let room: Value = res.json().await.unwrap();
    assert_eq!(room["name"], "Спальня");
    assert_eq!(room["devices"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn get_nonexistent_room() {
    let base = spawn_server().await;
    let res = reqwest::get(format!("{}/api/rooms/Несуществующая", base)).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body: Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("не найдена"));
}

#[tokio::test]
async fn remove_room() {
    let base = spawn_server().await;
    reqwest::Client::new()
        .post(format!("{}/api/rooms", base))
        .json(&json!({ "name": "Кухня" }))
        .send()
        .await
        .unwrap();

    let res = reqwest::Client::new()
        .delete(format!("{}/api/rooms/Кухня", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = reqwest::get(format!("{}/api/rooms", base)).await.unwrap();
    let rooms: Vec<Value> = res.json().await.unwrap();
    assert!(rooms.is_empty());
}

#[tokio::test]
async fn remove_nonexistent_room() {
    let base = spawn_server().await;
    let res = reqwest::Client::new()
        .delete(format!("{}/api/rooms/Прихожая", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// ---- Device tests ----------------------------------------------------------

#[tokio::test]
async fn add_socket_and_get_info() {
    let base = spawn_server().await;
    // Create room first
    reqwest::Client::new()
        .post(format!("{}/api/rooms", base))
        .json(&json!({ "name": "Кабинет" }))
        .send()
        .await
        .unwrap();

    // Add socket
    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms/Кабинет/devices", base))
        .json(&json!({
            "key": "pc",
            "type": "socket",
            "name": "Компьютер",
            "is_on": true,
            "power": 350.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Get device info
    let res = reqwest::get(format!("{}/api/rooms/Кабинет/devices/pc", base)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let dev: Value = res.json().await.unwrap();
    assert_eq!(dev["type"], "socket");
    assert_eq!(dev["name"], "Компьютер");
    assert_eq!(dev["is_on"], true);
    assert_eq!(dev["power"], 350.0);
    assert!(dev["temperature"].is_null());
}

#[tokio::test]
async fn add_thermometer_and_get_info() {
    let base = spawn_server().await;
    reqwest::Client::new()
        .post(format!("{}/api/rooms", base))
        .json(&json!({ "name": "Ванная" }))
        .send()
        .await
        .unwrap();

    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms/Ванная/devices", base))
        .json(&json!({
            "key": "thermo",
            "type": "thermometer",
            "name": "Термометр",
            "temperature": 23.5
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = reqwest::get(format!("{}/api/rooms/Ванная/devices/thermo", base)).await.unwrap();
    let dev: Value = res.json().await.unwrap();
    assert_eq!(dev["type"], "thermometer");
    assert_eq!(dev["name"], "Термометр");
    assert_eq!(dev["temperature"], 23.5);
    assert!(dev["is_on"].is_null());
}

#[tokio::test]
async fn add_device_unknown_type() {
    let base = spawn_server().await;
    reqwest::Client::new()
        .post(format!("{}/api/rooms", base))
        .json(&json!({ "name": "Test" }))
        .send()
        .await
        .unwrap();

    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms/Test/devices", base))
        .json(&json!({
            "key": "dev",
            "type": "fridge",
            "name": "Холодильник"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn add_device_to_nonexistent_room() {
    let base = spawn_server().await;
    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms/Nope/devices", base))
        .json(&json!({
            "key": "dev",
            "type": "socket",
            "name": "Test"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_devices_in_room() {
    let base = spawn_server_with_demo().await;
    let res = reqwest::get(format!("{}/api/rooms/Спальня/devices", base)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let devices: Vec<Value> = res.json().await.unwrap();
    assert_eq!(devices.len(), 2);
    let keys: Vec<&str> = devices.iter().map(|d| d["key"].as_str().unwrap()).collect();
    assert!(keys.contains(&"lamp"));
    assert!(keys.contains(&"temp"));
}

#[tokio::test]
async fn remove_device() {
    let base = spawn_server_with_demo().await;
    let res = reqwest::Client::new()
        .delete(format!("{}/api/rooms/Спальня/devices/lamp", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = reqwest::get(format!("{}/api/rooms/Спальня/devices", base)).await.unwrap();
    let devices: Vec<Value> = res.json().await.unwrap();
    assert_eq!(devices.len(), 1);
}

#[tokio::test]
async fn get_nonexistent_device() {
    let base = spawn_server_with_demo().await;
    let res = reqwest::get(format!("{}/api/rooms/Спальня/devices/ghost", base)).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn turn_socket_on_and_off() {
    let base = spawn_server().await;
    reqwest::Client::new()
        .post(format!("{}/api/rooms", base))
        .json(&json!({ "name": "Test" }))
        .send()
        .await
        .unwrap();

    // Add socket that is off
    reqwest::Client::new()
        .post(format!("{}/api/rooms/Test/devices", base))
        .json(&json!({
            "key": "socket",
            "type": "socket",
            "name": "Розетка",
            "is_on": false,
            "power": 100.0
        }))
        .send()
        .await
        .unwrap();

    // Turn on
    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms/Test/devices/socket/turn_on", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify
    let res = reqwest::get(format!("{}/api/rooms/Test/devices/socket", base)).await.unwrap();
    let dev: Value = res.json().await.unwrap();
    assert_eq!(dev["is_on"], true);

    // Turn off
    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms/Test/devices/socket/turn_off", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = reqwest::get(format!("{}/api/rooms/Test/devices/socket", base)).await.unwrap();
    let dev: Value = res.json().await.unwrap();
    assert_eq!(dev["is_on"], false);
    assert_eq!(dev["power"], 0.0);
}

#[tokio::test]
async fn turn_on_thermometer_rejected() {
    let base = spawn_server_with_demo().await;
    let res = reqwest::Client::new()
        .post(format!("{}/api/rooms/Спальня/devices/temp/turn_on", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// ---- Report tests ----------------------------------------------------------

#[tokio::test]
async fn get_report_contains_home_and_rooms() {
    let base = spawn_server_with_demo().await;
    let res = reqwest::get(format!("{}/api/report", base)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    let report = body["report"].as_str().unwrap();
    assert!(report.contains("Демо-дом"));
    assert!(report.contains("Спальня"));
    assert!(report.contains("Лампа"));
    assert!(report.contains("Термометр"));
}

#[tokio::test]
async fn get_report_empty_home() {
    let base = spawn_server().await;
    let res = reqwest::get(format!("{}/api/report", base)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await.unwrap();
    let report = body["report"].as_str().unwrap();
    assert!(report.contains("Тестовый дом"));
}

// ---- Full workflow test ----------------------------------------------------

#[tokio::test]
async fn full_workflow() {
    let base = spawn_server().await;
    let client = reqwest::Client::new();

    // 1. Add room
    let res = client
        .post(format!("{}/api/rooms", base))
        .json(&json!({ "name": "Гостиная" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 2. Add socket
    let res = client
        .post(format!("{}/api/rooms/Гостиная/devices", base))
        .json(&json!({
            "key": "tv",
            "type": "socket",
            "name": "Телевизор",
            "is_on": false,
            "power": 120.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. Add thermometer
    let res = client
        .post(format!("{}/api/rooms/Гостиная/devices", base))
        .json(&json!({
            "key": "temp",
            "type": "thermometer",
            "name": "Уличный",
            "temperature": -5.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 4. List rooms
    let res = client.get(format!("{}/api/rooms", base)).send().await.unwrap();
    let rooms: Vec<Value> = res.json().await.unwrap();
    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0]["device_count"], 2);

    // 5. Get room detail
    let res = client.get(format!("{}/api/rooms/Гостиная", base)).send().await.unwrap();
    let room: Value = res.json().await.unwrap();
    assert_eq!(room["devices"].as_array().unwrap().len(), 2);

    // 6. Turn on socket
    let res = client
        .post(format!("{}/api/rooms/Гостиная/devices/tv/turn_on", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 7. Get device info
    let res = client.get(format!("{}/api/rooms/Гостиная/devices/tv", base)).send().await.unwrap();
    let dev: Value = res.json().await.unwrap();
    assert_eq!(dev["is_on"], true);

    // 8. Get report
    let res = client.get(format!("{}/api/report", base)).send().await.unwrap();
    let body: Value = res.json().await.unwrap();
    let report = body["report"].as_str().unwrap();
    assert!(report.contains("Телевизор"));
    assert!(report.contains("включена"));

    // 9. Remove device
    let res = client
        .delete(format!("{}/api/rooms/Гостиная/devices/tv", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 10. Remove room
    let res = client
        .delete(format!("{}/api/rooms/Гостиная", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 11. Verify empty
    let res = client.get(format!("{}/api/rooms", base)).send().await.unwrap();
    let rooms: Vec<Value> = res.json().await.unwrap();
    assert!(rooms.is_empty());
}
