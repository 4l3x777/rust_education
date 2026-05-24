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

#[derive(Debug)]
struct ThermConfig {
    target_addr: String,
    interval_ms: u64,
}

fn parse_config(path: &str) -> ThermConfig {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Не удалось прочитать конфиг '{}': {}", path, e));

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

    // Привязываем к произвольному порту
    let socket =
        UdpSocket::bind("0.0.0.0:0").expect("[Имитатор термометра] Не удалось создать сокет");
    socket
        .set_nonblocking(true)
        .expect("set_nonblocking failed");

    loop {
        // Генерируем псевдослучайную температуру в диапазоне 18.0..26.0
        let temp = pseudo_random_temp();
        let msg = format!("{:.1}", temp);

        match socket.send_to(msg.as_bytes(), &config.target_addr) {
            Ok(_) => println!("[Имитатор термометра] Отправлено: {}°C → {}", temp, config.target_addr),
            Err(e) => eprintln!("[Имитатор термометра] Ошибка отправки: {}", e),
        }

        thread::sleep(Duration::from_millis(config.interval_ms));
    }
}

// Простой LCG для получения псевдослучайной температуры без внешних крейтов.
static SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(12345);

fn pseudo_random_temp() -> f64 {
    use std::sync::atomic::Ordering;
    let prev = SEED.load(Ordering::Relaxed);
    // LCG: Xn+1 = (a * Xn + c) mod m
    let next = prev
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    SEED.store(next, Ordering::Relaxed);
    // Диапазон 18.0..26.0
    let raw = 18.0 + (next % 80) as f64 / 10.0;
    (raw * 10.0).round() / 10.0
}
