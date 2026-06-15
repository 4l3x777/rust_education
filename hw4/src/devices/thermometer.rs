// Умный термометр.
// Поддерживает два режима:
//  - `local`: фиксированное значение температуры (для тестов).
//  - `udp`: получает температуру из UDP-пакетов в фоновом потоке.
//
// При создании UDP-термометра запускается фоновый поток, который слушает
// UDP-пакеты и обновляет AtomicI64 (температура × 1000).
// При дроппинге объекта поток завершается и join вызывается явно.
//
// `temperature_milli` инициализируется сентинел-значением `i64::MIN`,
// пока не пришёл хотя бы один UDP-пакет. Если данных ещё нет,
// `temperature()` возвращает `Err(SmartHomeError::NetworkError)`.

use crate::{Report, SmartHomeError};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Сентинел — означает «данных ещё не получено».
const NO_DATA: i64 = i64::MIN;

// Внутреннее хранилище для UDP-режима.
struct UdpState {
    // Температура × 1000; `NO_DATA` пока не получен ни один пакет.
    temperature_milli: Arc<AtomicI64>,
    stop_flag: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

// Реализация Debug для UdpState.
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
    // Создать локальный (in-memory) термометр с фиксированной температурой.
    pub fn local(name: &str, temperature: f64) -> Self {
        Self {
            name: name.to_string(),
            backend: ThermometerBackend::Local(temperature),
        }
    }

    // Создать UDP-термометр.
    // `bind_addr` — локальный адрес для прослушивания UDP-пакетов.
    // Фоновый поток запускается немедленно и завершается при дропе объекта.
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

    // Получить последнее известное значение температуры.
    // Возвращает `Err(NetworkError)`, если UDP-пакеты ещё не приходили.
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
            // Сигнализируем поток завершиться
            state.stop_flag.store(true, Ordering::Release);
            // Явный join устраняет гонку при уничтожении объекта
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

    // UDP-термометр возвращает ошибку, пока не пришёл ни один пакет.
    #[test]
    fn udp_thermometer_no_data_returns_error() {
        let t = SmartThermometer::udp("Тест", "127.0.0.1:0").expect("бинд должен удасться");
        // Данных ещё нет — ожидаем NetworkError
        assert!(matches!(
            t.temperature(),
            Err(SmartHomeError::NetworkError(_))
        ));
    }

    // UDP-термометр обновляет значение после прихода пакета.
    #[test]
    fn udp_thermometer_receives_temperature() {
        // Биндимся на порт 0 — ОС выдаст свободный порт;
        // узнаем назначенный адрес через local_addr().
        let therm = SmartThermometer::udp("Тест", "127.0.0.1:0").expect("бинд");

        // Получаем адрес, выданный ОС, через вспомогательный сокет
        let helper = UdpSocket::bind("127.0.0.1:0").unwrap();
        // Чтобы узнать порт термометра, создаём временный сокет с тем же биндом.
        // Мы не можем получить local_addr напрямую, поэтому биндимся
        // на чуть другом порту и отправляем на известный адрес.
        // Извлекаем адрес через обходной схему:
        // создаём второй UDP-сокет, связываем с адресом термометра через connect
        let probe = UdpSocket::bind("127.0.0.1:0").unwrap();

        // Для получения порта термометра используем временно связанный сокет:
        // отправляем пакет на себя же, зная свой адрес, а адрес термометра
        // берём из вспомогательного сокета, прослушивающего тот же порт.
        //
        // Проще всего: используем обход: биндимся на порт 0
        // и отправляем пакет на адрес therm, узнав его через helper.local_addr.
        // Но helper уже занял port:0 -> OS выдал port X.
        // Термометр занял другой port Y. helper.local_addr() = 127.0.0.1:X.
        // Нам нужен адрес термометра (Y).
        //
        // Самый простой путь: SmartThermometer имеет непубличное поле socket.
        // Мы не можем его читать, но можем знать адрес заранее:
        // передаём его в конструктор как bind_addr, потом helper отправляет на него.
        let bind_addr = helper.local_addr().unwrap();
        drop(helper); // освобождаем порт

        let therm2 =
            SmartThermometer::udp("Тест̦", &bind_addr.to_string()).expect("бинд второго термометра");
        let _ = therm; // первый нужен только чтобы показать no_data; перепривязка не нужна

        // Отправляем пакет с температурой
        probe.send_to(b"23.5", bind_addr).unwrap();

        // Даём фоновому потоку получить пакет
        thread::sleep(Duration::from_millis(100));

        let temp = therm2.temperature().expect("должны получить значение");
        assert!(
            (temp - 23.5).abs() < 0.001,
            "Ожидали 23.5, получили {}",
            temp
        );
    }
}
