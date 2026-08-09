use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct RoomSummary {
    pub key: String,
    pub name: String,
    pub device_count: usize,
}

#[derive(Serialize)]
pub struct RoomInfo {
    pub key: String,
    pub name: String,
    pub devices: Vec<DeviceSummary>,
}

#[derive(Serialize)]
pub struct DeviceSummary {
    pub key: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateDeviceRequest {
    pub key: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub name: String,
    pub is_on: Option<bool>,
    pub power: Option<f64>,
    pub temperature: Option<f64>,
}

#[derive(Serialize)]
pub struct DeviceInfo {
    pub key: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub name: String,
    pub is_on: Option<bool>,
    pub power: Option<f64>,
    pub temperature: Option<f64>,
}

#[derive(Serialize)]
pub struct ReportResponse {
    pub report: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
