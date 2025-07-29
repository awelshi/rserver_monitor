use crate::server::{Server, ServerConfig};
use crate::app::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub servers: Vec<ServerConfig>,
    pub refresh_interval_secs: u64,
}

pub fn export_config(app: &AppState, path: &str) {
    let config = AppConfig {
        servers: app.servers.iter().map(|s| {
            let mut sorted_ports = s.ports.clone();
            sorted_ports.sort_unstable();
            ServerConfig {
                name: s.name.clone(),
                ip: s.ip.clone(),
                ports: sorted_ports,
            }
        }).collect(),
        refresh_interval_secs: app.refresh_interval.as_secs(),
    };
    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            if fs::write(path, json).is_ok() {
                println!("> Exported config to {}", path);
            } else {
                eprintln!("! Failed to write config file.");
            }
        }
        Err(e) => eprintln!("! Serialization error: {}", e),
    }
}

pub fn import_config(app: &mut AppState, path: &str) {
    match fs::read_to_string(path) {
        Ok(data) => match serde_json::from_str::<AppConfig>(&data) {
            Ok(mut config) => {
                for server in &mut config.servers {
                    server.ports.sort_unstable();
                }
                app.servers = config.servers.into_iter().map(|s| Server {
                    name: s.name,
                    ip: s.ip,
                    ports: s.ports,
                    last_checked: None,
                    is_online: false,
                    open_ports: vec![],
                }).collect();
                app.refresh_interval = Duration::from_secs(config.refresh_interval_secs);
                app.update_status();
                println!("< Imported config from {}", path);
            }
            Err(e) => eprintln!("! JSON parse error: {}", e),
        },
        Err(e) => eprintln!("! File read error: {}", e),
    }
}

