// Умный термометр.
// Поддерживает два режима:
//  - `local`: фиксированное значение температуры (для тестов).
//  - `udp`: получает температуру из UDP-пакетов в фоновом потоке.
//
// При создании UDP-термометра запускается фоновый поток, который слушает
// UDP-пакеты и обновляет AtomicI64 (температура × 1000).
// При дроппинге объекта поток завершается.

use crate::{Report, SmartHomeError};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Внутреннее хранилище для UDP-режима.
struct UdpState {
    temperature_milli: Arc<AtomicI64>, // температура × 1000
    stop_flag: Arc<AtomicBool>,
    _thread: thread::JoinHandle<()>,
}

impl std::fmt::Debug for UdpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UdpState(temp={})",
            self.temperature_milli.load(Ordering::Relaxed)
        )
    }
}

#[derive(Debug)]
enum ThermometerBackend {
    Local(f64),
    Udp(UdpState),
}

#[derive(Debug)]
pub struct SmartThermometer {
    name: String,
    backend: ThermometerBackend,
}

impl SmartThermometer {
    // Создать локальный (in-memory) термометр с фиксированной температурой.
    pub fn local(name: &str, temperature: f64) -> Self {
        Self {
            name: name.to_string(),
            backend: ThermometerBackend::Local(temperature),
        }
    }

    // Создать UDP-термометр.
    // `bind_addr` — локальный адрес для прослушивания UDP-пакетов (например, "0.0.0.0:9000").
    // Фоновый поток запускается немедленно и завершается при дропе объекта.
    pub fn udp(name: &str, bind_addr: &str) -> Result<Self, SmartHomeError> {
        let socket = UdpSocket::bind(bind_addr)
            .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;

        let temperature_milli = Arc::new(AtomicI64::new(0));
        let stop_flag = Arc::new(AtomicBool::new(false));

        let temp_clone = Arc::clone(&temperature_milli);
        let stop_clone = Arc::clone(&stop_flag);

        let _thread = thread::spawn(move || {
            let mut buf = [0u8; 64];
            loop {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                match socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                            if let Ok(val) = s.trim().parse::<f64>() {
                                let milli = (val * 1000.0) as i64;
                                temp_clone.store(milli, Ordering::Relaxed);
                            }
                        }
                    }
                    Err(ref e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            name: name.to_string(),
            backend: ThermometerBackend::Udp(UdpState {
                temperature_milli,
                stop_flag,
                _thread,
            }),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    // Получить последнее известное значение температуры.
    pub fn temperature(&self) -> Result<f64, SmartHomeError> {
        match &self.backend {
            ThermometerBackend::Local(t) => Ok(*t),
            ThermometerBackend::Udp(state) => {
                let milli = state.temperature_milli.load(Ordering::Relaxed);
                Ok(milli as f64 / 1000.0)
            }
        }
    }
}

impl Drop for SmartThermometer {
    fn drop(&mut self) {
        if let ThermometerBackend::Udp(state) = &self.backend {
            state.stop_flag.store(true, Ordering::Relaxed);
        }
    }
}

impl Report for SmartThermometer {
    fn report(&self) -> Result<String, SmartHomeError> {
        let temp = self.temperature()?;
        Ok(format!(
            "[Термометр '{}'] Температура: {:.1}°C",
            self.name, temp
        ))
    }
}
