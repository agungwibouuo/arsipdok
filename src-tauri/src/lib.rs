//! ArsipDok — Sortir & Transfer Dokumentasi Foto & Video

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub mod commands;
pub mod error;
pub mod logger;
pub mod metadata;
pub mod paths;
pub mod planner;
pub mod scanner;
pub mod settings;
pub mod transfer;
pub mod verifier;

pub use error::{Error, Result};

pub const VERSION: &str = "0.1.0";

/// State aplikasi yang dibagi antar command
pub struct AppState {
    pub config: AppConfig,
    pub settings: parking_lot::RwLock<settings::Settings>,
    pub cancel_flag: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let settings = settings::Settings::load(&config.settings_file).unwrap_or_default();
        Self {
            config,
            settings: parking_lot::RwLock::new(settings),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Konfigurasi aplikasi
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub app_data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub settings_file: PathBuf,
}

impl AppConfig {
    pub fn from_home() -> Self {
        let app_data_dir = dirs::data_dir()
            .map(|d| d.join("ArsipDok"))
            .unwrap_or_else(|| PathBuf::from("./data"));
        let log_dir = app_data_dir.join("logs");
        let settings_file = app_data_dir.join("settings.json");
        Self { app_data_dir, log_dir, settings_file }
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.app_data_dir)?;
        std::fs::create_dir_all(&self.log_dir)?;
        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::from_home();
    if let Err(e) = config.ensure_dirs() {
        eprintln!("Gagal membuat direktori aplikasi: {e}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(config))
        .invoke_handler(tauri::generate_handler![
            commands::get_version,
            commands::load_settings,
            commands::save_settings,
            commands::scan_sources,
            commands::extract_metadata,
            commands::build_plan,
            commands::start_transfer,
            commands::cancel_transfer,
            commands::export_log,
            commands::check_disk_space,
            commands::validate_destination,
            commands::open_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ArsipDok");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_paths_contain_app_name() {
        let cfg = AppConfig::from_home();
        assert!(cfg.app_data_dir.to_string_lossy().contains("ArsipDok"));
        assert!(cfg.settings_file.to_string_lossy().ends_with("settings.json"));
    }
}
