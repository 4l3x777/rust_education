// Имитатор умного термометра — неблокирующий UDP-отправитель.
//
// Читает конфиг из файла (формат INI-подобный):
//   target_addr=127.0.0.1:9000
//   interval_ms=1000
//
// Запуск: `cargo run --bin thermometer_sim -- thermometer_sim.conf`

use std::fs;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq)]
struct ThermConfig {
    target_addr: String,
    interval_ms: u64,
}

fn parse_config(path: &str) -> ThermConfig {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Не удалось прочитать конфиг '{}': {}", path, e));
    parse_config_str(&content)
}

// Парсинг конфига из строки (выделено для тестируемости без файловой системы).
fn parse_config_str(content: &str) -> ThermConfig {
    let mut target_addr = String::from("127.0.0.1:9000");
    let mut interval_ms: u64 = 1000;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            match key.trim() {
                "target_addr" => target_addr = val.trim().to_string(),
                "interval_ms" => {
                    interval_ms = val.trim().parse().unwrap_or(1000);
                }
                _ => {}
            }
        }
    }

    ThermConfig {
        target_addr,
        interval_ms,
    }
}

// Запустить имитатор термометра (блокирует поток до Ctrl+C).
pub fn run_thermometer_simulator(config_path: &str) {
    let config = parse_config(config_path);
    println!(
        "[Имитатор термометра] Отправляю данные на {} каждые {} мс",
        config.target_addr, config.interval_ms
    );
    run_send_loop(&config.target_addr, config.interval_ms, || true);
}

// Цикл отправки; завершается, когда `keep_going()` возвращает `false`.
// Выделено для тестируемости (production передаёт `|| true`).
fn run_send_loop(target_addr: &str, interval_ms: u64, keep_going: impl Fn() -> bool) {
    let socket =
        UdpSocket::bind("0.0.0.0:0").expect("[Имитатор термометра] Не удалось создать сокет");
    socket
        .set_nonblocking(true)
        .expect("set_nonblocking failed");

    loop {
        let temp = pseudo_random_temp();
        let msg = format!("{:.1}", temp);

        match socket.send_to(msg.as_bytes(), target_addr) {
            Ok(_) => {}
            Err(e) => eprintln!("[Имитатор термометра] Ошибка отправки: {}", e),
        }

        if !keep_going() {
            break;
        }
        thread::sleep(Duration::from_millis(interval_ms));
    }
}

// Простой LCG без внешних крейтов.
static SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(12345);

pub(crate) fn pseudo_random_temp() -> f64 {
    use std::sync::atomic::Ordering;
    let prev = SEED.load(Ordering::Relaxed);
    let next = prev
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    SEED.store(next, Ordering::Relaxed);
    let raw = 18.0 + (next % 80) as f64 / 10.0;
    (raw * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    // -------------------------------------------------------------------------
    // Тесты парсера конфига
    // -------------------------------------------------------------------------

    #[test]
    fn parse_config_defaults() {
        // Пустой файл — используются значения по умолчанию
        let cfg = parse_config_str("");
        assert_eq!(cfg.target_addr, "127.0.0.1:9000");
        assert_eq!(cfg.interval_ms, 1000);
    }

    #[test]
    fn parse_config_full() {
        let content = "target_addr=192.168.1.10:5555\ninterval_ms=250\n";
        let cfg = parse_config_str(content);
        assert_eq!(cfg.target_addr, "192.168.1.10:5555");
        assert_eq!(cfg.interval_ms, 250);
    }

    #[test]
    fn parse_config_ignores_comments_and_blanks() {
        let content = "
            # это комментарий

            target_addr = 10.0.0.1:8888
            # interval не задан — должно быть 1000
        ";
        let cfg = parse_config_str(content);
        assert_eq!(cfg.target_addr, "10.0.0.1:8888");
        assert_eq!(cfg.interval_ms, 1000);
    }

    #[test]
    fn parse_config_invalid_interval_falls_back_to_default() {
        let content = "interval_ms=not_a_number\n";
        let cfg = parse_config_str(content);
        assert_eq!(cfg.interval_ms, 1000);
    }

    #[test]
    fn parse_config_unknown_keys_ignored() {
        let content = "unknown_key=foobar\ntarget_addr=1.2.3.4:1234\n";
        let cfg = parse_config_str(content);
        assert_eq!(cfg.target_addr, "1.2.3.4:1234");
        assert_eq!(cfg.interval_ms, 1000);
    }

    // -------------------------------------------------------------------------
    // Тесты генератора температуры
    // -------------------------------------------------------------------------

    #[test]
    fn pseudo_random_temp_in_range() {
        // 1000 значений — все должны лежать в [18.0, 25.9]
        for _ in 0..1000 {
            let t = pseudo_random_temp();
            assert!(
                (18.0..=25.9).contains(&t),
                "Температура вышла за диапазон: {}",
                t
            );
        }
    }

    #[test]
    fn pseudo_random_temp_one_decimal_place() {
        // Округление до десятых: (t * 10).round() == t * 10
        for _ in 0..200 {
            let t = pseudo_random_temp();
            let rounded = (t * 10.0).round() / 10.0;
            assert!(
                (t - rounded).abs() < 1e-9,
                "Значение {} не округлено до десятых",
                t
            );
        }
    }

    #[test]
    fn pseudo_random_temp_not_constant() {
        // Генератор должен давать разные значения
        let vals: Vec<f64> = (0..20).map(|_| pseudo_random_temp()).collect();
        let unique: std::collections::HashSet<u64> =
            vals.iter().map(|v| v.to_bits()).collect();
        assert!(
            unique.len() > 1,
            "Генератор выдаёт одно и то же значение"
        );
    }

    // -------------------------------------------------------------------------
    // Интеграционные тесты: реальная отправка UDP-пакетов
    // -------------------------------------------------------------------------

    /// Запускает `run_send_loop` в фоновом потоке, N раз с затем останавливает цикл.
    fn spawn_sim(target_addr: String, count: usize) -> thread::JoinHandle<()> {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let limit = count;
        let counter_clone = Arc::clone(&counter);
        thread::spawn(move || {
            run_send_loop(&target_addr, 0, move || {
                let prev = counter_clone.fetch_add(1, Ordering::Relaxed);
                prev < limit - 1  // после limit-й отправки возвращает false
            });
        })
    }

    #[test]
    fn sim_sends_udp_packets_to_receiver() {
        // Слушатель на случайном порту
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let addr = receiver.local_addr().unwrap().to_string();

        // Отправляем ровно 3 пакета
        let handle = spawn_sim(addr, 3);

        let mut received = Vec::new();
        let mut buf = [0u8; 64];
        for _ in 0..3 {
            match receiver.recv_from(&mut buf) {
                Ok((n, _)) => received.push(String::from_utf8_lossy(&buf[..n]).trim().to_string()),
                Err(e) => panic!("Не получили пакет: {}", e),
            }
        }

        handle.join().unwrap();
        assert_eq!(received.len(), 3, "Ожидали 3 пакета, имеем {:?}", received);
    }

    #[test]
    fn sim_packets_are_valid_temperatures() {
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let addr = receiver.local_addr().unwrap().to_string();

        spawn_sim(addr, 5);

        let mut buf = [0u8; 64];
        for _ in 0..5 {
            let (n, _) = receiver.recv_from(&mut buf).expect("пакет");
            let s = String::from_utf8_lossy(&buf[..n]);
            let val: f64 = s.trim().parse().unwrap_or_else(|_| {
                panic!("Пакет '{}' не является f64", s)
            });
            assert!(
                (18.0..=25.9).contains(&val),
                "Значение {} вышло за диапазон",
                val
            );
            // Один знак после точки
            let s = s.trim();
            if let Some(dot_pos) = s.find('.') {
                assert_eq!(
                    s.len() - dot_pos - 1,
                    1,
                    "Ожидали 1 знак после точки в '{}'",
                    s
                );
            }
        }
    }

    #[test]
    fn sim_sends_to_correct_address() {
        // Имитатор должен отправлять пакеты только на целевой адрес,
        // а не на посторонний порт
        let target = UdpSocket::bind("127.0.0.1:0").unwrap();
        target
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        let target_addr = target.local_addr().unwrap().to_string();

        let decoy = UdpSocket::bind("127.0.0.1:0").unwrap();
        decoy
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        spawn_sim(target_addr, 2);

        // target получает пакеты
        let mut buf = [0u8; 64];
        assert!(
            target.recv_from(&mut buf).is_ok(),
            "Целевой адрес должен получать пакет"
        );
        // посторонний порт не получает ничего
        assert!(
            decoy.recv_from(&mut buf).is_err(),
            "Посторонний порт не должен получать пакеты"
        );
    }

    #[test]
    fn sim_stop_flag_terminates_loop() {
        // Цикл должен остановиться, как только keep_going() возвращает false
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let addr = receiver.local_addr().unwrap().to_string();

        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);

        let handle = thread::spawn(move || {
            run_send_loop(&addr, 0, move || !stop_clone.load(Ordering::Relaxed));
        });

        // Даём циклу отправить несколько пакетов
        let mut buf = [0u8; 64];
        receiver.recv_from(&mut buf).expect("пакет до остановки");

        // Ставим флаг и ждём завершения потока
        stop.store(true, Ordering::Relaxed);
        let joined = handle.join();
        assert!(joined.is_ok(), "Поток должен завершиться без паники");
    }
}