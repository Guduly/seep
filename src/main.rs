use crate::app::{App, Screen};
use std::process::Command;
pub mod chat;
pub mod app;
pub mod event;
pub mod ui;
pub mod config;
pub mod bridge;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _ = std::fs::remove_file("/tmp/seep-ready");

    let whatsapp_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("whatsapp");

    let bridge_path = [
        std::env::current_exe().unwrap().parent().unwrap().to_path_buf(),
        whatsapp_dir.clone(),
    ]
    .iter()
    .find(|p| p.join("whatsapp").exists())
    .cloned()
    .expect("Could not find whatsapp bridge")
    .join("whatsapp");

    let session_exists = whatsapp_dir.join("session.db").exists();

    if session_exists {
        let log_file = std::fs::File::create("/tmp/seep-bridge.log").unwrap();
        Command::new(&bridge_path)
            .current_dir(&whatsapp_dir)
            .stdout(log_file.try_clone().unwrap())
            .stderr(log_file)
            .spawn()
            .expect("Failed to start WhatsApp bridge");
    } else {
        // first run - show QR in terminal
        Command::new(&bridge_path)
            .current_dir(&whatsapp_dir)
            .spawn()
            .expect("Failed to start WhatsApp bridge");
    }

    // wait for bridge - long timeout to allow QR scanning
    let ready_path = std::path::Path::new("/tmp/seep-ready");
    let mut waited = 0;
    while !ready_path.exists() && waited < 120 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        waited += 1;
    }
    if !ready_path.exists() {
        eprintln!("Bridge failed to start in time");
        std::process::exit(1);
    }

    let terminal = ratatui::init();
    let mut app = App::new();
    if let Some(config) = config::load() {
        if !config.discord_token.is_empty() {
            app.discord_token = config.discord_token;
            app.screen = Screen::Main;
        }
    }

    let result = app.run(terminal).await;
    ratatui::restore();
    result
}
