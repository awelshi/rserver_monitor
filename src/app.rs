use eframe::egui::{self, TextureHandle, ColorImage, TextureOptions};
use std::net::{IpAddr};
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

use crate::server::{Server, ServerConfig, check_server_status};
use crate::config::{AppConfig};
use crate::icon::load_icon_texture;

pub struct AppState {
    pub servers: Vec<Server>,
    pub rt: Runtime,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,

    pub new_server_name: String,
    pub new_server_ip: String,
    pub new_server_ports: String,
    pub add_dialog_open: bool,

    pub edit_dialog_open: Option<usize>,
    pub edit_name: String,
    pub edit_ip: String,
    pub edit_ports: String,

    pub about_dialog_open: bool,
    pub icon_texture: Option<TextureHandle>,
}

impl AppState {
    pub fn new() -> Self {
        let mut state = Self {
            servers: vec![
                Server::new("Server A", "192.1.1.1", vec![22, 80]),
                Server::new("Server B", "192.1.1.2", vec![443]),
            ],
            rt: Runtime::new().expect("Tokio runtime init failed"),
            last_refresh: Instant::now() - Duration::from_secs(601),
            refresh_interval: Duration::from_secs(600),
            new_server_name: String::new(),
            new_server_ip: String::new(),
            new_server_ports: String::new(),
            add_dialog_open: false,
            edit_dialog_open: None,
            edit_name: String::new(),
            edit_ip: String::new(),
            edit_ports: String::new(),
            about_dialog_open: false,
            icon_texture: None,
        };
        let path = Self::config_path();
        if path.exists() {
            state.import_config(path.to_str().unwrap());
        }

        state
    }
   //Set the configuration file path
    fn config_path() -> std::path::PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".servermon.cfg");
        path
    }
    pub fn update_status(&mut self) {
        for server in &mut self.servers {
            check_server_status(server, &self.rt);
        }
    }
    pub fn export_config(&self, path: &str) {
        crate::config::export_config(self, path);
    }
    pub fn import_config(&mut self, path: &str) {
        crate::config::import_config(self, path);
    }
    pub fn save_on_exit(&self) {
        let path = Self::config_path();
        self.export_config(path.to_str().unwrap());
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        if self.refresh_interval.as_secs() > 0
            && now.duration_since(self.last_refresh) >= self.refresh_interval
        {
            self.update_status();
            self.last_refresh = now;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
              if let Some(icon) = &self.icon_texture {
                ui.image((icon.id(), egui::Vec2::splat(24.0))); // Small icon
              }
              ui.heading("Server Monitor");
            });
            ui.horizontal(|ui| {
                if ui.button("🔄 Sync Now").clicked() {
                    self.update_status();
                    self.last_refresh = Instant::now();
                }
                if ui.button("📤 Export Config").clicked() {
                    let path = Self::config_path();
                    self.export_config(path.to_str().unwrap());
                }
                if ui.button("📥 Import Config").clicked() {
                    let path = Self::config_path();
                    self.import_config(path.to_str().unwrap());
                }
                if ui.button("➕ Add Server").clicked() {
                    self.add_dialog_open = true;
                }
                if ui.button("About").clicked() {
                    self.about_dialog_open = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Refresh every:");
                let mut secs_str = self.refresh_interval.as_secs().to_string();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut secs_str)
                            .desired_width(25.0)
                            .interactive(true),
                    )
                    .changed()
                {
                    if let Ok(val) = secs_str.parse::<u64>() {
                        self.refresh_interval = Duration::from_secs(val);
                    }
                }
                ui.label("seconds");
                if self.refresh_interval.as_secs() == 0 {
                    ui.label("Auto-refresh disabled");
                } else {
                    ui.label(format!(
                        "Next refresh in: {}s",
                        self.refresh_interval
                            .saturating_sub(now.duration_since(self.last_refresh))
                            .as_secs()
                    ));
                }
            });
            ui.separator();
            //New Server dialogue box
            if self.add_dialog_open {
                egui::Window::new("Add New Server")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Name:");
                                ui.label("IP:");
                                ui.label("Ports (comma-separated):");
                            });
                            ui.vertical(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_server_name)
                                        .desired_width(175.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_server_ip)
                                        .desired_width(175.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.new_server_ports)
                                        .desired_width(175.0),
                                );
                            });
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Add").clicked() {
                                if self.new_server_ip.parse::<IpAddr>().is_ok() {
                                    let mut ports = self
                                        .new_server_ports
                                        .split(',')
                                        .filter_map(|p| p.trim().parse::<u16>().ok())
                                        .collect::<Vec<_>>();
                                    ports.sort_unstable();

                                    self.servers.push(Server::new(
                                        &self.new_server_name,
                                        &self.new_server_ip,
                                        ports,
                                    ));

                                    self.new_server_name.clear();
                                    self.new_server_ip.clear();
                                    self.new_server_ports.clear();
                                    self.add_dialog_open = false;
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                self.add_dialog_open = false;
                            }
                        });
                    });
            }
            if let Some(edit_idx) = self.edit_dialog_open {
                egui::Window::new("Edit Server")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Name:");
                                ui.label("IP:");
                                ui.label("Ports (comma-separated):");
                            });
                            ui.vertical(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.edit_name)
                                        .desired_width(175.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.edit_ip)
                                        .desired_width(175.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.edit_ports)
                                        .desired_width(175.0),
                                );
                            });
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                if self.edit_ip.parse::<IpAddr>().is_ok() {
                                    if let Some(server) = self.servers.get_mut(edit_idx) {
                                        server.name = self.edit_name.clone();
                                        server.ip = self.edit_ip.clone();
                                        server.ports = self
                                            .edit_ports
                                            .split(',')
                                            .filter_map(|p| p.trim().parse::<u16>().ok())
                                            .collect();
                                        server.ports.sort_unstable();
                                    }
                                    self.edit_dialog_open = None;
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                self.edit_dialog_open = None;
                            }
                        });
                    });
            }
       //About dialogue box
        if self.about_dialog_open {
            egui::Window::new("About Server Monitor")
                .collapsible(false)
                .resizable(false)
                .auto_sized()
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if let Some(icon) = &self.icon_texture {
                            ui.image((icon.id(), egui::Vec2::splat(128.0)));
                        }
                        ui.label("Server Monitor");
                        ui.label("Built with Rust + egui + tokio");
                        ui.hyperlink_to("By awelshi", "https://github.com/awelshi");
                    });
                    ui.vertical_centered(|ui| {
                        if ui.button("Close").clicked() {
                            self.about_dialog_open = false;
                        }
                    });
                });
            }
            ui.separator();
            let mut to_remove = vec![];
            //Create a card for each server, and populate it
            for (i, server) in self.servers.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("🔹 {}", server.name));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌ Remove").clicked() {
                                to_remove.push(i);
                            }
                            if ui.button("✏️ Edit").clicked() {
                                self.edit_dialog_open = Some(i);
                                self.edit_name = server.name.clone();
                                self.edit_ip = server.ip.clone();
                                self.edit_ports = server
                                    .ports
                                    .iter()
                                    .map(|p| p.to_string())
                                    .collect::<Vec<_>>()
                                    .join(",");
                            }
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_max_width(175.0);
                            ui.label(format!("IP: {}", server.ip));
                            if server.is_online {
                                ui.colored_label(egui::Color32::GREEN, "Status: ONLINE");
                            } else {
                                ui.colored_label(egui::Color32::RED, "Status: OFFLINE");
                            }
                            if let Some(t) = server.last_checked {
                                ui.label(format!(
                                    "Last checked: {:.1}s ago",
                                    t.elapsed().as_secs_f32()
                                ));
                            }
                        });
                        let mut sorted_ports = server.ports.clone();
                        sorted_ports.sort_unstable();
                        let ports_per_col = 4;
                        let port_cols = (sorted_ports.len() + ports_per_col - 1) / ports_per_col;
                        for col in 0..port_cols {
                            ui.vertical(|ui| {
                                let start = col * ports_per_col;
                                let end = ((col + 1) * ports_per_col).min(sorted_ports.len());
                                for port in &sorted_ports[start..end] {
                                    if server.open_ports.contains(port) {
                                        ui.colored_label(
                                            egui::Color32::GREEN,
                                            format!("Port {}: OPEN", port),
                                        );
                                    } else {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("Port {}: CLOSED", port),
                                        );
                                    }
                                }
                            });
                        }
                    });
                });
                ui.separator();
            }
            for &i in to_remove.iter().rev() {
                self.servers.remove(i);
            }
        });
        ctx.request_repaint_after(Duration::from_secs(1));
    }
    //Request the config be saved on exit
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_on_exit();
    }
}

