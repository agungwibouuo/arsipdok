//! Tauri commands: IPC layer antara frontend dan backend

use crate::{error::Result, scanner, metadata, planner, transfer, settings, logger};
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
pub struct Ok {
    pub ok: bool,
    pub message: Option<String>,
}

#[tauri::command]
pub fn get_version() -> String {
    crate::VERSION.to_string()
}

#[tauri::command]
pub fn load_settings(state: State<'_, crate::AppState>) -> Result<settings::Settings> {
    let s = state.settings.read();
    Ok((*s).clone())
}

#[tauri::command]
pub fn save_settings(state: State<'_, crate::AppState>, s: settings::Settings) -> Result<()> {
    s.validate()?;
    s.save(&state.config.settings_file)?;
    *state.settings.write() = s;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ScanReport {
    pub total_files: usize,
    pub total_bytes: u64,
    pub photos: usize,
    pub videos: usize,
    pub skipped: usize,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn scan_sources(sources: Vec<PathBuf>) -> Result<ScanReport> {
    let mut total_files = 0;
    let mut total_bytes = 0;
    let mut photos = 0;
    let mut videos = 0;
    let mut skipped = 0;
    let mut total_ms = 0;

    for src in sources {
        let res = scanner::scan_source(&src)?;
        total_ms += res.duration_ms;
        skipped += res.skipped_count;

        for f in &res.files {
            total_files += 1;
            total_bytes += f.size_bytes;
            match f.kind {
                scanner::FileKind::Photo => photos += 1,
                scanner::FileKind::Video => videos += 1,
            }
        }
    }

    Ok(ScanReport {
        total_files,
        total_bytes,
        photos,
        videos,
        skipped,
        duration_ms: total_ms,
    })
}

#[tauri::command]
pub async fn extract_metadata(files: Vec<scanner::ScannedFile>) -> Result<BTreeMap<PathBuf, metadata::DateMetadata>> {
    let mut map = BTreeMap::new();
    for f in files {
        let m = metadata::extract_date(&f.path, f.created, f.modified);
        map.insert(f.path, m);
    }
    Ok(map)
}

#[tauri::command]
pub async fn build_plan(
    files: Vec<scanner::ScannedFile>,
    dates: BTreeMap<PathBuf, metadata::DateMetadata>,
    destinations: Vec<settings::Destination>,
) -> Result<planner::TransferPlan> {
    planner::build_plan(&files, &dates, &destinations)
}

#[derive(Debug, Serialize)]
pub struct TransferReport {
    pub items_done: usize,
    pub items_error: usize,
    pub items_skipped: usize,
    pub total_bytes_transferred: u64,
}

#[tauri::command]
pub async fn start_transfer(
    state: State<'_, crate::AppState>,
    plan: planner::TransferPlan,
    mode: settings::TransferMode,
) -> Result<TransferReport> {
    let settings = state.settings.read();
    let cfg = transfer::TransferConfig {
        mode,
        verification: settings.verification,
        buffer_size: 4 * 1024 * 1024,
    };

    let mut report = TransferReport {
        items_done: 0,
        items_error: 0,
        items_skipped: 0,
        total_bytes_transferred: 0,
    };

    let mut log = logger::SessionLog::new(&state.config.log_dir);

    for item in plan.items {
        if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let res = transfer::execute_item(&item, &cfg, &state.cancel_flag);

        match res.status {
            transfer::ItemStatus::Done => {
                report.items_done += 1;
                report.total_bytes_transferred += item.size_bytes;
            }
            transfer::ItemStatus::Skipped | transfer::ItemStatus::Duplicate => {
                report.items_skipped += 1;
            }
            _ => report.items_error += 1,
        }

        // Catat di log
        log.push(&res, &format!("{:?}", mode), "unknown");
    }

    log.save()?;

    Ok(report)
}

#[tauri::command]
pub fn cancel_transfer(state: State<'_, crate::AppState>) {
    state.cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[tauri::command]
pub fn export_log(log_path: PathBuf, export_path: PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(&log_path)?;
    std::fs::write(&export_path, content)?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct DiskSpaceInfo {
    pub path: PathBuf,
    pub available_bytes: u64,
    pub total_bytes: u64,
}

#[tauri::command]
pub fn check_disk_space(path: PathBuf) -> Result<DiskSpaceInfo> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let meta = std::fs::metadata(&path)?;
        Ok(DiskSpaceInfo {
            path,
            available_bytes: 0,
            total_bytes: meta.file_size(),
        })
    }
    #[cfg(not(windows))]
    {
        Ok(DiskSpaceInfo {
            path,
            available_bytes: 0,
            total_bytes: 0,
        })
    }
}

#[tauri::command]
pub fn validate_destination(path: PathBuf) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let meta = std::fs::metadata(&path)?;
    Ok(meta.is_dir())
}

#[tauri::command]
pub fn open_path(app: tauri::AppHandle, path: PathBuf) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| crate::error::Error::Other(format!("Gagal membuka path: {e}")))?;
    Ok(())
}
