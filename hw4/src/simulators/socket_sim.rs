// Имитатор умной розетки — неблокирующий TCP-сервер.
//
// И `TcpListener`, и клиентские `TcpStream` работают в неблокирующем режиме.
// Цикл приёма соединений и обработка команд организованы через опрос
// (пауза при `WouldBlock`); внешнего потока на клиента не создаётся.
//
// Запуск: `cargo run --bin socket_sim -- 127.0.0.1:7878`
//
// Протокол (текстовый, строки с '\n'):
//   ON    → включить, ответ "OK"
//   OFF   → выключить, ответ "OK"
//   STATE → "ON" или "OFF"
//   POWER → мощность в ваттах
//   <прочее> → "ERR unknown command"

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct SocketState {
    is_on: bool,
    power_watts: f64,
}

// Буфер чтения для одного неблокирующего клиента.
// Накапливает байты до '\n', возвращая `WouldBlock`, если данных пока нет.
#[derive(Default)]
struct LineBuffer {
    buf: Vec<u8>,
}

impl LineBuffer {
    // Читает до '\n' из неблокирующего сокета.
    // Возвращает:
    //   `Ok(Some(line))` — получена полная строка
    //   `Ok(None)`       — соединение закрыто (EOF)
    //   `Err(WouldBlock)` — данных пока нет, повторить позже
    //   `Err(_)`          — фатальная ошибка
    fn try_read_line(&mut self, stream: &mut TcpStream) -> io::Result<Option<String>> {
        let mut tmp = [0u8; 256];
        loop {
            // Проверяем буфер: вдруг '\n' уже есть в накопленном
            if let Some(pos) = self.buf.iter().position(|&b| b == b'\n') {
                let line_bytes = self.buf.drain(..=pos).collect::<Vec<_>>();
                let line = String::from_utf8_lossy(&line_bytes).into_owned();
                return Ok(Some(line));
            }

            match stream.read(&mut tmp) {
                Ok(0) => return Ok(None), // EOF
                Ok(n) => self.buf.extend_from_slice(&tmp.as_slice()[..n]),
                Err(ref e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut =>
                {
                    return Err(io::Error::from(io::ErrorKind::WouldBlock));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

// Запустить неблокирующий имитатор розетки на указанном адресе.
//
// Цикл обслуживает всех клиентов в одном потоке:
//   - `listener.accept()` — неблокирующий (при `WouldBlock` поток продолжает цикл)
//   - для каждого клиента `stream.read()` — неблокирующий (при `WouldBlock` переходим к следующему)
pub fn run_socket_simulator(addr: &str) {
    let shared_state = Arc::new(Mutex::new(SocketState {
        is_on: false,
        power_watts: 1500.0,
    }));

    let listener = TcpListener::bind(addr).expect("Не удалось занять адрес");
    listener
        .set_nonblocking(true)
        .expect("set_nonblocking(true) для TcpListener не удалось");

    println!(
        "[Имитатор розетки] Слушаю на {} (неблокирующий режим)",
        addr
    );

    // id -> (TcpStream, LineBuffer)
    let mut clients: HashMap<u64, (TcpStream, LineBuffer)> = HashMap::new();
    let mut next_id: u64 = 0;

    loop {
        // прием новых соединений
        match listener.accept() {
            Ok((stream, peer)) => {
                stream
                    .set_nonblocking(true)
                    .expect("set_nonblocking(true) для клиента не удалось");
                println!("[Имитатор розетки] Подключился: {}", peer);
                clients.insert(next_id, (stream, LineBuffer::default()));
                next_id += 1;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => eprintln!("[Имитатор розетки] accept ошибка: {}", e),
        }

        // опрос всех активных клиентов
        let mut to_remove: Vec<u64> = Vec::new();

        for (&id, (stream, buf)) in clients.iter_mut() {
            match buf.try_read_line(stream) {
                Ok(Some(line)) => {
                    let cmd = line.trim();
                    let response = process_command(cmd, &shared_state);
                    if let Err(e) = stream.write_all(response.as_bytes()) {
                        eprintln!("[Имитатор розетки] Ошибка записи клиенту #{}: {}", id, e);
                        to_remove.push(id);
                    }
                }
                Ok(None) => {
                    println!("[Имитатор розетки] Клиент #{} закрыл соединение", id);
                    to_remove.push(id);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // данных пока нет — пропускаем до следующей итерации
                }
                Err(e) => {
                    eprintln!("[Имитатор розетки] Ошибка чтения клиента #{}: {}", id, e);
                    to_remove.push(id);
                }
            }
        }

        for id in to_remove {
            clients.remove(&id);
        }

        // Короткая пауза, чтобы не жечь процессор в холостом опросе
        thread::sleep(Duration::from_millis(1));
    }
}

fn process_command(cmd: &str, state: &Arc<Mutex<SocketState>>) -> String {
    let mut st = state.lock().unwrap();
    match cmd {
        "ON" => {
            st.is_on = true;
            "OK\n".into()
        }
        "OFF" => {
            st.is_on = false;
            "OK\n".into()
        }
        "STATE" => {
            if st.is_on {
                "ON\n".into()
            } else {
                "OFF\n".into()
            }
        }
        "POWER" => {
            let p = if st.is_on { st.power_watts } else { 0.0 };
            format!("{:.3}\n", p)
        }
        other => format!("ERR unknown command: {}\n", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    // Запускает неблокирующий имитатор в фоновом потоке.
    fn spawn_test_server() -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();

        let shared_state = Arc::new(Mutex::new(SocketState {
            is_on: false,
            power_watts: 1500.0,
        }));

        thread::spawn(move || {
            let mut clients: HashMap<u64, (TcpStream, LineBuffer)> = HashMap::new();
            let mut next_id: u64 = 0;

            loop {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream.set_nonblocking(true).unwrap();
                        clients.insert(next_id, (stream, LineBuffer::default()));
                        next_id += 1;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => break,
                }

                let mut to_remove = Vec::new();
                for (&id, (stream, buf)) in clients.iter_mut() {
                    match buf.try_read_line(stream) {
                        Ok(Some(line)) => {
                            let resp = process_command(line.trim(), &shared_state);
                            if stream.write_all(resp.as_bytes()).is_err() {
                                to_remove.push(id);
                            }
                        }
                        Ok(None) => to_remove.push(id),
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                        Err(_) => to_remove.push(id),
                    }
                }
                for id in to_remove {
                    clients.remove(&id);
                }
                thread::sleep(Duration::from_millis(1));
            }
        });

        addr
    }

    fn send_cmd(addr: std::net::SocketAddr, cmd: &str) -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        writeln!(stream, "{}", cmd).unwrap();
        let mut reader = BufReader::new(&stream);
        let mut resp = String::new();
        reader.read_line(&mut resp).unwrap();
        resp.trim().to_string()
    }

    #[test]
    fn tcp_sim_on_off_state() {
        let addr = spawn_test_server();
        thread::sleep(Duration::from_millis(50));

        assert_eq!(send_cmd(addr, "STATE"), "OFF");
        assert_eq!(send_cmd(addr, "ON"), "OK");
        assert_eq!(send_cmd(addr, "STATE"), "ON");
        assert_eq!(send_cmd(addr, "OFF"), "OK");
        assert_eq!(send_cmd(addr, "STATE"), "OFF");
    }

    #[test]
    fn tcp_sim_power_reflects_state() {
        let addr = spawn_test_server();
        thread::sleep(Duration::from_millis(50));

        let power_off: f64 = send_cmd(addr, "POWER").parse().unwrap();
        assert_eq!(power_off, 0.0);

        send_cmd(addr, "ON");
        let power_on: f64 = send_cmd(addr, "POWER").parse().unwrap();
        assert!(
            (power_on - 1500.0).abs() < 0.001,
            "Ожидали 1500.0, получили {}",
            power_on
        );
    }

    #[test]
    fn tcp_sim_smart_socket_client() {
        let addr = spawn_test_server();
        thread::sleep(Duration::from_millis(50));

        let mut socket = crate::devices::socket::SmartSocket::tcp("Тест", &addr.to_string());

        socket.turn_on().expect("включить");
        assert!(socket.is_on().expect("state"));

        let power = socket.current_power().expect("power");
        assert!(
            (power - 1500.0).abs() < 0.001,
            "Ожидали 1500.0, получили {}",
            power
        );

        socket.turn_off().expect("выключить");
        assert!(!socket.is_on().expect("state"));
    }

    #[test]
    fn tcp_sim_multiple_clients_concurrent() {
        let addr = spawn_test_server();
        thread::sleep(Duration::from_millis(50));

        // Два клиента одновременно: один включает, второй читает состояние
        let h1 = thread::spawn(move || send_cmd(addr, "ON"));
        thread::sleep(Duration::from_millis(10));
        let h2 = thread::spawn(move || send_cmd(addr, "STATE"));

        let r1 = h1.join().unwrap();
        let r2 = h2.join().unwrap();

        assert_eq!(r1, "OK");
        // Состояние может быть любым из ON/OFF в зависимости от гонки,
        // но должно быть валидным ответом
        assert!(r2 == "ON" || r2 == "OFF", "Неожиданный ответ: {}", r2);
    }
}
