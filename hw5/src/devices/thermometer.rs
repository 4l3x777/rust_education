//! Умный термометр с двумя бэкендами: `local` (in-memory) и `udp`.
//!
//! UDP-режим: фоновый поток слушает входящие пакеты и обновляет `AtomicI64`
//! (температура × 1000). Значение `i64::MIN` означает «данных ещё нет» —
//! в этом случае `temperature()` возвращает `Err(NetworkError)`.
//!
//! При дропе объекта поток останавливается через `AtomicBool` и явный `join`.

use crate::{Report, SmartHomeError};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const NO_DATA: i64 = i64::MIN;

struct UdpState {
    temperature_milli: Arc<AtomicI64>,
    stop_flag: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl std::fmt::Debug for UdpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.temperature_milli.load(Ordering::Relaxed);
        if v == NO_DATA {
            write!(f, "UdpState(no data yet)")
        } else {
            write!(f, "UdpState(temp={:.3})", v as f64 / 1000.0)
        }
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
    pub fn local(name: &str, temperature: f64) -> Self {
        Self {
            name: name.to_string(),
            backend: ThermometerBackend::Local(temperature),
        }
    }

    /// Создаёт UDP-термометр и сразу запускает фоновый поток приёма пакетов.
    pub fn udp(name: &str, bind_addr: &str) -> Result<Self, SmartHomeError> {
        let socket =
            UdpSocket::bind(bind_addr).map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;

        let temperature_milli = Arc::new(AtomicI64::new(NO_DATA));
        let stop_flag = Arc::new(AtomicBool::new(false));

        let temp_clone = Arc::clone(&temperature_milli);
        let stop_clone = Arc::clone(&stop_flag);

        let handle = thread::spawn(move || {
            let mut buf = [0u8; 64];
            loop {
                if stop_clone.load(Ordering::Acquire) {
                    break;
                }
                match socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                            if let Ok(val) = s.trim().parse::<f64>() {
                                let milli = (val * 1000.0) as i64;
                                temp_clone.store(milli, Ordering::Release);
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
                thread: Some(handle),
            }),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn temperature(&self) -> Result<f64, SmartHomeError> {
        match &self.backend {
            ThermometerBackend::Local(t) => Ok(*t),
            ThermometerBackend::Udp(state) => {
                let milli = state.temperature_milli.load(Ordering::Acquire);
                if milli == NO_DATA {
                    Err(SmartHomeError::NetworkError(
                        "данные от термометра ещё не получены".to_string(),
                    ))
                } else {
                    Ok(milli as f64 / 1000.0)
                }
            }
        }
    }
}

impl Drop for SmartThermometer {
    fn drop(&mut self) {
        if let ThermometerBackend::Udp(state) = &mut self.backend {
            state.stop_flag.store(true, Ordering::Release);
            if let Some(handle) = state.thread.take() {
                handle.join().ok();
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::time::Duration;

    #[test]
    fn local_thermometer_returns_fixed_temperature() {
        let t = SmartThermometer::local("Тест", 21.5);
        assert_eq!(t.temperature().unwrap(), 21.5);
    }

    #[test]
    fn udp_thermometer_no_data_returns_error() {
        let t = SmartThermometer::udp("Тест", "127.0.0.1:0").expect("бинд должен удасться");
        assert!(matches!(
            t.temperature(),
            Err(SmartHomeError::NetworkError(_))
        ));
    }

    #[test]
    fn udp_thermometer_receives_temperature() {
        let helper = UdpSocket::bind("127.0.0.1:0").unwrap();
        let bind_addr = helper.local_addr().unwrap();
        drop(helper); // освобождаем порт перед тем, как займёт термометр

        let therm = SmartThermometer::udp("Тест", &bind_addr.to_string())
            .expect("бинд второго термометра");

        let probe = UdpSocket::bind("127.0.0.1:0").unwrap();
        probe.send_to(b"23.5", bind_addr).unwrap();

        thread::sleep(Duration::from_millis(100));

        let temp = therm.temperature().expect("должны получить значение");
        assert!(
            (temp - 23.5).abs() < 0.001,
            "Ожидали 23.5, получили {}",
            temp
        );
    }
}
