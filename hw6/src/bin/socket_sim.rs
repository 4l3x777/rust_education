//! Запуск: `cargo run --bin socket_sim -- 127.0.0.1:7878`

use smart_home::simulators::socket_sim::run_socket_simulator;
use std::env;

fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7878".to_string());
    run_socket_simulator(&addr);
}
