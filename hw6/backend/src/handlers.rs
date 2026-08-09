use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use smart_home::{Report, Room, SmartDevice, SmartHomeError, SmartSocket, SmartThermometer};

use crate::models::*;
use crate::state::AppState;

type ApiResult<T> = Result<T, ApiError>;

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }

    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

fn map_home_error(e: SmartHomeError) -> ApiError {
    match e {
        SmartHomeError::RoomNotFound(r) => ApiError::not_found(format!("Комната '{}' не найдена", r)),
        SmartHomeError::DeviceNotFound { room, device } => {
            ApiError::not_found(format!("Устройство '{}' не найдено в комнате '{}'", device, room))
        }
        SmartHomeError::NetworkError(msg) => ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Сетевая ошибка: {}", msg),
        },
    }
}

fn device_info(key: &str, device: &SmartDevice) -> DeviceInfo {
    match device {
        SmartDevice::Socket(s) => DeviceInfo {
            key: key.to_string(),
            device_type: "socket".to_string(),
            name: s.name().to_string(),
            is_on: s.is_on().ok(),
            power: s.current_power().ok(),
            temperature: None,
        },
        SmartDevice::Thermometer(t) => DeviceInfo {
            key: key.to_string(),
            device_type: "thermometer".to_string(),
            name: t.name().to_string(),
            is_on: None,
            power: None,
            temperature: t.temperature().ok(),
        },
    }
}

fn device_summary(key: &str, device: &SmartDevice) -> DeviceSummary {
    let device_type = match device {
        SmartDevice::Socket(_) => "socket",
        SmartDevice::Thermometer(_) => "thermometer",
    };
    let name = match device {
        SmartDevice::Socket(s) => s.name(),
        SmartDevice::Thermometer(t) => t.name(),
    };
    DeviceSummary {
        key: key.to_string(),
        device_type: device_type.to_string(),
        name: name.to_string(),
    }
}

// ---- Room handlers ---------------------------------------------------------

pub async fn list_rooms(State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    let home = state.home.read().await;
    let rooms: Vec<RoomSummary> = home
        .room_keys()
        .into_iter()
        .filter_map(|key| {
            home.get_room(key).map(|room| RoomSummary {
                key: key.to_string(),
                name: room.name().to_string(),
                device_count: room.device_keys().len(),
            })
        })
        .collect();
    Ok(Json(rooms))
}

pub async fn add_room(
    State(state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> ApiResult<impl IntoResponse> {
    if req.name.trim().is_empty() {
        return Err(ApiError::bad_request("Имя комнаты не может быть пустым"));
    }
    let mut home = state.home.write().await;
    let room = Room::empty(&req.name);
    home.add_room(req.name.clone(), room);
    Ok((StatusCode::CREATED, Json(CreateRoomResponse { key: req.name })))
}

#[derive(serde::Serialize)]
struct CreateRoomResponse {
    key: String,
}

pub async fn get_room(
    State(state): State<AppState>,
    Path(room_key): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let home = state.home.read().await;
    let room = home
        .get_room(&room_key)
        .ok_or_else(|| ApiError::not_found(format!("Комната '{}' не найдена", room_key)))?;
    let devices: Vec<DeviceSummary> = room
        .device_keys()
        .into_iter()
        .filter_map(|key| room.get_device(key).map(|d| device_summary(key, d)))
        .collect();
    Ok(Json(RoomInfo {
        key: room_key,
        name: room.name().to_string(),
        devices,
    }))
}

pub async fn remove_room(
    State(state): State<AppState>,
    Path(room_key): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let mut home = state.home.write().await;
    home.remove_room(&room_key)
        .ok_or_else(|| ApiError::not_found(format!("Комната '{}' не найдена", room_key)))?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- Device handlers -------------------------------------------------------

pub async fn list_devices(
    State(state): State<AppState>,
    Path(room_key): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let home = state.home.read().await;
    let room = home
        .get_room(&room_key)
        .ok_or_else(|| ApiError::not_found(format!("Комната '{}' не найдена", room_key)))?;
    let devices: Vec<DeviceSummary> = room
        .device_keys()
        .into_iter()
        .filter_map(|key| room.get_device(key).map(|d| device_summary(key, d)))
        .collect();
    Ok(Json(devices))
}

pub async fn add_device(
    State(state): State<AppState>,
    Path(room_key): Path<String>,
    Json(req): Json<CreateDeviceRequest>,
) -> ApiResult<impl IntoResponse> {
    if req.key.trim().is_empty() {
        return Err(ApiError::bad_request("Ключ устройства не может быть пустым"));
    }
    if req.name.trim().is_empty() {
        return Err(ApiError::bad_request("Имя устройства не может быть пустым"));
    }

    let device = match req.device_type.as_str() {
        "socket" => {
            let is_on = req.is_on.unwrap_or(false);
            let power = req.power.unwrap_or(0.0);
            SmartDevice::from(SmartSocket::local(&req.name, is_on, power))
        }
        "thermometer" => {
            let temp = req.temperature.unwrap_or(20.0);
            SmartDevice::from(SmartThermometer::local(&req.name, temp))
        }
        other => {
            return Err(ApiError::bad_request(format!(
                "Неизвестный тип устройства: '{}'. Допустимые: socket, thermometer",
                other
            )));
        }
    };

    let mut home = state.home.write().await;
    let room = home
        .get_room_mut(&room_key)
        .ok_or_else(|| ApiError::not_found(format!("Комната '{}' не найдена", room_key)))?;
    room.add_device(req.key.clone(), device);
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "key": req.key }))))
}

pub async fn get_device(
    State(state): State<AppState>,
    Path((room_key, device_key)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let home = state.home.read().await;
    let device = home.get_device(&room_key, &device_key).map_err(map_home_error)?;
    Ok(Json(device_info(&device_key, device)))
}

pub async fn remove_device(
    State(state): State<AppState>,
    Path((room_key, device_key)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let mut home = state.home.write().await;
    let room = home
        .get_room_mut(&room_key)
        .ok_or_else(|| ApiError::not_found(format!("Комната '{}' не найдена", room_key)))?;
    room.remove_device(&device_key)
        .ok_or_else(|| ApiError::not_found(format!("Устройство '{}' не найдено", device_key)))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn turn_on(
    State(state): State<AppState>,
    Path((room_key, device_key)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let mut home = state.home.write().await;
    let room = home
        .get_room_mut(&room_key)
        .ok_or_else(|| ApiError::not_found(format!("Комната '{}' не найдена", room_key)))?;
    let device = room
        .get_device_mut(&device_key)
        .ok_or_else(|| ApiError::not_found(format!("Устройство '{}' не найдено", device_key)))?;
    match device {
        SmartDevice::Socket(s) => {
            s.turn_on().map_err(|e| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: e.to_string(),
            })?;
        }
        SmartDevice::Thermometer(_) => {
            return Err(ApiError::bad_request(
                "Термометр нельзя включить/выключить",
            ));
        }
    }
    Ok(Json(serde_json::json!({ "status": "on" })))
}

pub async fn turn_off(
    State(state): State<AppState>,
    Path((room_key, device_key)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let mut home = state.home.write().await;
    let room = home
        .get_room_mut(&room_key)
        .ok_or_else(|| ApiError::not_found(format!("Комната '{}' не найдена", room_key)))?;
    let device = room
        .get_device_mut(&device_key)
        .ok_or_else(|| ApiError::not_found(format!("Устройство '{}' не найдено", device_key)))?;
    match device {
        SmartDevice::Socket(s) => {
            s.turn_off().map_err(|e| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: e.to_string(),
            })?;
        }
        SmartDevice::Thermometer(_) => {
            return Err(ApiError::bad_request(
                "Термометр нельзя включить/выключить",
            ));
        }
    }
    Ok(Json(serde_json::json!({ "status": "off" })))
}

// ---- Report handler --------------------------------------------------------

pub async fn get_report(State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    let home = state.home.read().await;
    let report = home.report().map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: e.to_string(),
    })?;
    Ok(Json(ReportResponse { report }))
}
