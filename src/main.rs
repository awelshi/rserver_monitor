pub mod app;
pub mod icon;
pub mod server;
pub mod config;

pub use app::AppState;
pub use icon::load_icon_texture;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native("Server Monitor", options, Box::new(|cc| {
        let mut app = AppState::new();
        app.icon_texture = Some(load_icon_texture(&cc.egui_ctx));
        Box::new(app)
    }))
}

