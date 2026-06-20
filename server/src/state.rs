use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::config::AppConfig;

pub type ClientMap = Arc<Mutex<HashMap<String, HashMap<String, ClientConnection>>>>;

pub struct ClientConnection {
    pub device_id: String,
    pub sender: mpsc::UnboundedSender<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub database: Pool<Sqlite>,
    pub clients: ClientMap,
    pub config: AppConfig,
}
