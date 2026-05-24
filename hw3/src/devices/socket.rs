// Умная розетка.
// Поддерживает два режима:
//  - `local`: чисто in-memory состояние (для тестов).
//  - `tcp`: реальное взаимодействие с имитатором через TCP.
//
// Протокол (текстовый, строки с '\n'):
//   Клиент -> сервер: "ON\n" | "OFF\n" | "POWER\n"
//   Сервер -> клиент: "OK\n" | "<число>\n" | "ERR <текст>\n"

use crate::{Report, SmartHomeError};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug)]
enum SocketBackend {
    /// In-memory состояние (для тестов / без сети)
    Local { is_on: bool, power_watts: f64 },
    /// Адрес TCP-имитатора
    Tcp(String),
}

#[derive(Debug)]
pub struct SmartSocket {
    name: String,
    backend: SocketBackend,
}

impl SmartSocket {
    // Создать локальную (in-memory) розетку.
    pub fn local(name: &str, is_on: bool, power_watts: f64) -> Self {
        Self {
            name: name.to_string(),
            backend: SocketBackend::Local { is_on, power_watts },
        }
    }

    // Создать TCP-розетку, подключающуюся к имитатору.
    pub fn tcp(name: &str, addr: &str) -> Self {
        Self {
            name: name.to_string(),
            backend: SocketBackend::Tcp(addr.to_string()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    // Включить розетку.
    pub fn turn_on(&mut self) -> Result<(), SmartHomeError> {
        match &mut self.backend {
            SocketBackend::Local { is_on, .. } => {
                *is_on = true;
                Ok(())
            }
            SocketBackend::Tcp(addr) => {
                let addr = addr.clone();
                let response = tcp_command(&addr, "ON")?;
                if response.trim() == "OK" {
                    Ok(())
                } else {
                    Err(SmartHomeError::NetworkError(response))
                }
            }
        }
    }

    // Выключить розетку.
    pub fn turn_off(&mut self) -> Result<(), SmartHomeError> {
        match &mut self.backend {
            SocketBackend::Local { is_on, .. } => {
                *is_on = false;
                Ok(())
            }
            SocketBackend::Tcp(addr) => {
                let addr = addr.clone();
                let response = tcp_command(&addr, "OFF")?;
                if response.trim() == "OK" {
                    Ok(())
                } else {
                    Err(SmartHomeError::NetworkError(response))
                }
            }
        }
    }

    // Запросить текущую мощность (Вт).
    pub fn current_power(&self) -> Result<f64, SmartHomeError> {
        match &self.backend {
            SocketBackend::Local { is_on, power_watts } => {
                Ok(if *is_on { *power_watts } else { 0.0 })
            }
            SocketBackend::Tcp(addr) => {
                let addr = addr.clone();
                let response = tcp_command(&addr, "POWER")?;
                response
                    .trim()
                    .parse::<f64>()
                    .map_err(|e| SmartHomeError::NetworkError(e.to_string()))
            }
        }
    }

    // Узнать состояние (включена/выключена).
    pub fn is_on(&self) -> Result<bool, SmartHomeError> {
        match &self.backend {
            SocketBackend::Local { is_on, .. } => Ok(*is_on),
            SocketBackend::Tcp(addr) => {
                let addr = addr.clone();
                let response = tcp_command(&addr, "STATE")?;
                match response.trim() {
                    "ON" => Ok(true),
                    "OFF" => Ok(false),
                    other => Err(SmartHomeError::NetworkError(format!(
                        "Неожиданный ответ: {}",
                        other
                    ))),
                }
            }
        }
    }
}

impl Report for SmartSocket {
    fn report(&self) -> Result<String, SmartHomeError> {
        let is_on = self.is_on()?;
        let power = self.current_power()?;
        let state = if is_on { "включена" } else { "выключена" };
        Ok(format!(
            "[Розетка '{}'] Состояние: {}, Мощность: {:.1} Вт",
            self.name, state, power
        ))
    }
}

// Открыть TCP-соединение, отправить команду, прочитать один ответ.
fn tcp_command(addr: &str, cmd: &str) -> Result<String, SmartHomeError> {
    let mut stream = TcpStream::connect(addr)
        .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;

    writeln!(stream, "{}", cmd)
        .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;

    Ok(response)
}
