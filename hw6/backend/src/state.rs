use std::sync::Arc;
use tokio::sync::RwLock;

use smart_home::SmartHome;

#[derive(Clone)]
pub struct AppState {
    pub home: Arc<RwLock<SmartHome>>,
}

impl AppState {
    pub fn new(home: SmartHome) -> Self {
        Self {
            home: Arc::new(RwLock::new(home)),
        }
    }
}
