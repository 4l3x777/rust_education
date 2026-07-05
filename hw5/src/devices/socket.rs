//! Умная розетка с двумя бэкендами: `local` (in-memory) и `tcp`.
//!
//! Протокол (текстовый, `\n`-терминированный):
//! - `ON` / `OFF` → `OK`
//! - `STATE` → `ON` | `OFF`
//! - `POWER` → число в ваттах
//! - прочее → `ERR <текст>`

use crate::{Report, SmartHomeError};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug)]
enum SocketBackend {
    Local { is_on: bool, power_watts: f64 },
    Tcp(String),
}

#[derive(Debug)]
pub struct SmartSocket {
    name: String,
    backend: SocketBackend,
}

impl SmartSocket {
    pub fn local(name: &str, is_on: bool, power_watts: f64) -> Self {
        Self {
            name: name.to_string(),
            backend: SocketBackend::Local { is_on, power_watts },
        }
    }

    pub fn tcp(name: &str, addr: &str) -> Self {
        Self {
            name: name.to_string(),
            backend: SocketBackend::Tcp(addr.to_string()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

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

// Открывает TCP-соединение, отправляет одну команду и читает одну строку ответа.
fn tcp_command(addr: &str, cmd: &str) -> Result<String, SmartHomeError> {
    let mut stream =
        TcpStream::connect(addr).map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;

    writeln!(stream, "{}", cmd).map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| SmartHomeError::NetworkError(e.to_string()))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn local_initial_state_off() {
        let s = SmartSocket::local("Тест", false, 1500.0);
        assert!(!s.is_on().unwrap());
    }

    #[test]
    fn local_initial_state_on() {
        let s = SmartSocket::local("Тест", true, 1500.0);
        assert!(s.is_on().unwrap());
    }

    #[test]
    fn local_turn_on_off() {
        let mut s = SmartSocket::local("Тест", false, 1000.0);
        s.turn_on().unwrap();
        assert!(s.is_on().unwrap());
        s.turn_off().unwrap();
        assert!(!s.is_on().unwrap());
    }

    #[test]
    fn local_power_zero_when_off() {
        let s = SmartSocket::local("Тест", false, 800.0);
        assert_eq!(s.current_power().unwrap(), 0.0);
    }

    #[test]
    fn local_power_when_on() {
        let s = SmartSocket::local("Тест", true, 800.0);
        assert!((s.current_power().unwrap() - 800.0).abs() < 1e-9);
    }

    #[test]
    fn local_report_contains_name_and_state() {
        let s = SmartSocket::local("Гостиная", true, 500.0);
        let r = s.report().unwrap();
        assert!(r.contains("Гостиная"));
        assert!(r.contains("включена"));
    }

    // Состояние розетки на стороне тестового сервера.
    #[derive(Clone)]
    struct ServerState {
        is_on: bool,
        power_watts: f64,
    }

    // Поднимает мини-сервер на случайном порту (передаём 0 — ОС выдаст свободный).
    fn spawn_stub_server() -> (std::net::SocketAddr, Arc<Mutex<ServerState>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let state = Arc::new(Mutex::new(ServerState {
            is_on: false,
            power_watts: 1500.0,
        }));
        let state_srv = Arc::clone(&state);

        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(s) => s,
                    Err(_) => break,
                };
                let st = Arc::clone(&state_srv);
                handle_stub_client(stream, st);
            }
        });

        (addr, state)
    }

    fn handle_stub_client(stream: TcpStream, state: Arc<Mutex<ServerState>>) {
        let mut writer = stream.try_clone().unwrap();
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let mut st = state.lock().unwrap();
            let resp = match line.trim() {
                "ON" => {
                    st.is_on = true;
                    "OK\n".to_string()
                }
                "OFF" => {
                    st.is_on = false;
                    "OK\n".to_string()
                }
                "STATE" => {
                    if st.is_on { "ON\n".to_string() } else { "OFF\n".to_string() }
                }
                "POWER" => format!("{:.3}\n", if st.is_on { st.power_watts } else { 0.0 }),
                other => format!("ERR unknown command: {}\n", other),
            };
            drop(st);
            if writer.write_all(resp.as_bytes()).is_err() {
                break;
            }
        }
    }

    #[test]
    fn tcp_turn_on_sets_state() {
        let (addr, srv_state) = spawn_stub_server();
        thread::sleep(Duration::from_millis(20));

        let mut s = SmartSocket::tcp("Тест", &addr.to_string());
        s.turn_on().expect("turn_on должен сработать");

        assert!(s.is_on().unwrap(), "STATE должен быть ON");
        assert!(srv_state.lock().unwrap().is_on);
    }

    #[test]
    fn tcp_turn_off_sets_state() {
        let (addr, srv_state) = spawn_stub_server();
        thread::sleep(Duration::from_millis(20));

        let mut s = SmartSocket::tcp("Тест", &addr.to_string());
        s.turn_on().unwrap();
        s.turn_off().expect("turn_off должен сработать");

        assert!(!s.is_on().unwrap(), "STATE должен быть OFF");
        assert!(!srv_state.lock().unwrap().is_on);
    }

    #[test]
    fn tcp_power_zero_when_off() {
        let (addr, _) = spawn_stub_server();
        thread::sleep(Duration::from_millis(20));

        let s = SmartSocket::tcp("Тест", &addr.to_string());
        let power = s.current_power().unwrap();
        assert_eq!(power, 0.0, "Мощность должна быть 0 при выключенной розетке");
    }

    #[test]
    fn tcp_power_when_on() {
        let (addr, _) = spawn_stub_server();
        thread::sleep(Duration::from_millis(20));

        let mut s = SmartSocket::tcp("Тест", &addr.to_string());
        s.turn_on().unwrap();
        let power = s.current_power().unwrap();
        assert!(
            (power - 1500.0).abs() < 0.001,
            "Ожидали 1500.0, получили {}",
            power
        );
    }

    #[test]
    fn tcp_report_contains_name_and_state() {
        let (addr, _) = spawn_stub_server();
        thread::sleep(Duration::from_millis(20));

        let mut s = SmartSocket::tcp("Кухня", &addr.to_string());
        s.turn_on().unwrap();
        let r = s.report().unwrap();
        assert!(r.contains("Кухня"), "Имя должно быть в отчёте");
        assert!(r.contains("включена"), "Состояние должно быть в отчёте");
    }

    #[test]
    fn tcp_no_server_returns_network_error() {
        let s = SmartSocket::tcp("Тест", "127.0.0.1:1");
        assert!(
            matches!(s.is_on(), Err(SmartHomeError::NetworkError(_))),
            "Ожидали NetworkError при недоступном сервере"
        );
    }

    #[test]
    fn tcp_unknown_command_returns_err_response() {
        let (addr, _) = spawn_stub_server();
        thread::sleep(Duration::from_millis(20));

        let resp = tcp_command(&addr.to_string(), "FOOBAR").unwrap();
        assert!(
            resp.trim().starts_with("ERR"),
            "Ожидали ERR-ответ, получили: {}",
            resp.trim()
        );
    }

    #[test]
    fn tcp_sequential_commands_maintain_state() {
        let (addr, _) = spawn_stub_server();
        thread::sleep(Duration::from_millis(20));

        let mut s = SmartSocket::tcp("Тест", &addr.to_string());

        assert!(!s.is_on().unwrap());
        s.turn_on().unwrap();
        assert!(s.is_on().unwrap());
        s.turn_off().unwrap();
        assert!(!s.is_on().unwrap());
        s.turn_on().unwrap();
        assert!((s.current_power().unwrap() - 1500.0).abs() < 0.001);
    }
}
