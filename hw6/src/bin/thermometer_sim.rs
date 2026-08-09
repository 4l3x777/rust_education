//! Запуск: `cargo run --bin thermometer_sim -- thermometer_sim.conf`

use smart_home::simulators::thermometer_sim::run_thermometer_simulator;
use std::env;

fn main() {
    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "thermometer_sim.conf".to_string());
    run_thermometer_simulator(&config_path);
}
