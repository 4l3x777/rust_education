// Имитатор умной розетки — неблокирующий TCP-сервер.
//
// Запуск: `cargo run --bin socket_sim -- 127.0.0.1:7878`
//
// Протокол (текстовый, строки с '\n'):
//   ON    → включить розетку, ответ "OK"
//   OFF   → выключить розетку, ответ "OK"
//   STATE → ответ "ON" или "OFF"
//   POWER → мощность в ваттах (например "150.0")
//   <прочее> → "ERR unknown command"

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
struct SocketState {
    is_on: bool,
    power_watts: f64,
}

// Запустить имитатор розетки на указанном адресе.
// Возвращает сразу — принятие соединений идёт в фоновых потоках.
pub fn run_socket_simulator(addr: &str) {
    let state = Arc::new(Mutex::new(SocketState {
        is_on: false,
        power_watts: 1500.0,
    }));

    let listener = TcpListener::bind(addr).expect("Не удалось занять адрес");
    listener
        .set_nonblocking(false)
        .expect("set_nonblocking failed");

    println!("[Имитатор розетки] Слушаю на {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state_clone = Arc::clone(&state);
                thread::spawn(move || handle_socket_client(stream, state_clone));
            }
            Err(e) => eprintln!("[Имитатор розетки] Ошибка принятия соединения: {}", e),
        }
    }
}

fn handle_socket_client(mut stream: TcpStream, state: Arc<Mutex<SocketState>>) {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("[Имитатор розетки] Подключился клиент: {}", peer);

    let reader_stream = stream.try_clone().expect("clone failed");
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // соединение закрыто
            Err(e) => {
                eprintln!("[Имитатор розетки] Ошибка чтения от {}: {}", peer, e);
                break;
            }
            Ok(_) => {}
        }

        let cmd = line.trim();
        let response = {
            let mut st = state.lock().unwrap();
            match cmd {
                "ON" => {
                    st.is_on = true;
                    println!("[Имитатор розетки] {} → ON", peer);
                    "OK\n".to_string()
                }
                "OFF" => {
                    st.is_on = false;
                    println!("[Имитатор розетки] {} → OFF", peer);
                    "OK\n".to_string()
                }
                "STATE" => {
                    if st.is_on {
                        "ON\n".to_string()
                    } else {
                        "OFF\n".to_string()
                    }
                }
                "POWER" => {
                    let power = if st.is_on { st.power_watts } else { 0.0 };
                    format!("{:.3}\n", power)
                }
                other => {
                    eprintln!("[Имитатор розетки] Неизвестная команда: {:?}", other);
                    format!("ERR unknown command: {}\n", other)
                }
            }
        };

        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("[Имитатор розетки] Ошибка записи клиенту {}: {}", peer, e);
            break;
        }
    }

    println!("[Имитатор розетки] Клиент отключился: {}", peer);
}
