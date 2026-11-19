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
    pub files: Vec<scanner::ScannedFile>,
}

#[tauri::command]
pub async fn scan_sources(sources: Vec<PathBuf>) -> Result<ScanReport> {
    let mut all_files = Vec::new();
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

        for f in res.files {
            total_files += 1;
            total_bytes += f.size_bytes;
            match f.kind {
                scanner::FileKind::Photo => photos += 1,
                scanner::FileKind::Video => videos += 1,
            }
            all_files.push(f);
        }
    }

    Ok(ScanReport {
        total_files,
        total_bytes,
        photos,
        videos,
        skipped,
        duration_ms: total_ms,
        files: all_files,
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
    pub log_path: PathBuf,
    pub log_entries: Vec<logger::LogEntry>,
    pub cancelled: bool,
    pub duration_ms: u64,
}

/// Payload event progres per item (event `transfer://item`)
#[derive(Debug, Clone, Serialize)]
pub struct ItemProgress {
    pub source: String,
    pub file_name: String,
    pub dest_root: String,
    pub dest_path: String,
    pub status: String,
    pub error: Option<String>,
    pub source_deleted: bool,
}

/// Payload event progres agregat (event `transfer://progress`)
#[derive(Debug, Clone, Serialize)]
pub struct TransferProgress {
    pub items_done: usize,
    pub items_error: usize,
    pub items_skipped: usize,
    pub items_total: usize,
    pub bytes_transferred: u64,
    /// Progres per root tujuan: (root, selesai, total)
    pub per_dest: Vec<DestProgressEntry>,
    pub current_source: String,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DestProgressEntry {
    pub dest_root: String,
    pub done: usize,
    pub total: usize,
    pub has_error: bool,
}

fn status_ok(s: transfer::ItemStatus) -> bool {
    matches!(
        s,
        transfer::ItemStatus::Done | transfer::ItemStatus::Duplicate | transfer::ItemStatus::Skipped
    )
}

#[tauri::command]
pub async fn start_transfer(
    app: tauri::AppHandle,
    state: State<'_, crate::AppState>,
    plan: planner::TransferPlan,
    mode: settings::TransferMode,
    source_roots: Vec<PathBuf>,
) -> Result<TransferReport> {
    use std::collections::HashMap;
    use tauri::Emitter;

    let start_all = std::time::Instant::now();
    let verification = state.settings.read().verification;
    let cfg = transfer::TransferConfig {
        mode,
        verification,
        buffer_size: 4 * 1024 * 1024,
    };

    // Reset cancel flag dari sesi sebelumnya
    state
        .cancel_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);

    let items_total = plan.items.len();

    // Kelompokkan item per file sumber (pertahankan urutan kemunculan)
    let mut groups: Vec<(PathBuf, Vec<planner::PlanItem>)> = Vec::new();
    let mut index: HashMap<PathBuf, usize> = HashMap::new();
    for item in plan.items {
        match index.get(&item.source) {
            Some(&i) => groups[i].1.push(item),
            None => {
                index.insert(item.source.clone(), groups.len());
                groups.push((item.source.clone(), vec![item]));
            }
        }
    }

    // Hitung item per root tujuan untuk progres
    let mut per_dest_total: HashMap<PathBuf, usize> = HashMap::new();
    for it in groups.iter().flat_map(|(_, v)| v.iter()) {
        *per_dest_total.entry(it.dest_root.clone()).or_insert(0) += 1;
    }
    let mut per_dest_done: HashMap<PathBuf, usize> = HashMap::new();
    let mut per_dest_error: HashMap<PathBuf, bool> = HashMap::new();

    let mut report = TransferReport {
        items_done: 0,
        items_error: 0,
        items_skipped: 0,
        total_bytes_transferred: 0,
        log_path: PathBuf::new(),
        log_entries: Vec::new(),
        cancelled: false,
        duration_ms: 0,
    };

    let mut log = logger::SessionLog::new(&state.config.log_dir);
    let mode_str = match mode {
        settings::TransferMode::Copy => "copy",
        settings::TransferMode::Move => "move",
    };

    'outer: for (source, items) in groups {
        // Progress: file yang sedang diproses
        let progress_now = TransferProgress {
            items_done: report.items_done,
            items_error: report.items_error,
            items_skipped: report.items_skipped,
            items_total,
            bytes_transferred: report.total_bytes_transferred,
            per_dest: build_per_dest(&per_dest_total, &per_dest_done, &per_dest_error),
            current_source: source.to_string_lossy().to_string(),
            cancelled: false,
        };
        let _ = app.emit("transfer://progress", &progress_now);

        let mut all_ok = true;
        let mut deleted = false;

        for item in &items {
            let res = transfer::execute_item(item, &cfg, &state.cancel_flag);

            let ok = status_ok(res.status);
            if !ok {
                all_ok = false;
                *per_dest_error.entry(item.dest_root.clone()).or_insert(false) = true;
            }
            *per_dest_done.entry(item.dest_root.clone()).or_insert(0) += 1;

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

            // Event per item untuk UI
            let _ = app.emit(
                "transfer://item",
                &ItemProgress {
                    source: item.source.to_string_lossy().to_string(),
                    file_name: item.file_name.clone(),
                    dest_root: item.dest_root.to_string_lossy().to_string(),
                    dest_path: res
                        .final_path
                        .clone()
                        .unwrap_or_else(|| item.dest_path.clone())
                        .to_string_lossy()
                        .to_string(),
                    status: format!("{:?}", res.status).to_lowercase(),
                    error: res.error.clone(),
                    source_deleted: false,
                },
            );

            let meta_src = item
                .metadata_source
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            log.push(&res, mode_str, &meta_src);

            if state.cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                report.cancelled = true;
                break 'outer;
            }
        }

        // Mode Move: hapus sumber HANYA jika semua tujuan sukses
        if mode == settings::TransferMode::Move && all_ok && !items.is_empty() {
            match transfer::delete_source(&source) {
                Ok(()) => deleted = true,
                Err(e) => {
                    // Pencatatan kegagalan hapus — file tetap dianggap tersalin
                    log.push_error(&source, mode_str, &format!("Hapus sumber gagal: {e}"));
                }
            }
        }

        if deleted {
            let _ = app.emit(
                "transfer://item",
                &ItemProgress {
                    source: source.to_string_lossy().to_string(),
                    file_name: String::new(),
                    dest_root: String::new(),
                    dest_path: String::new(),
                    status: "source_deleted".into(),
                    error: None,
                    source_deleted: true,
                },
            );
        }
    }

    // Mode Move: bersihkan folder sumber yang kosong
    if mode == settings::TransferMode::Move
        && state.settings.read().remove_empty_source_dirs
        && !report.cancelled
    {
        for root in &source_roots {
            let _ = transfer::remove_empty_dirs(root);
        }
    }

    log.save()?;
    report.log_path = log.file_path.clone();
    report.log_entries = log.entries.clone();
    report.duration_ms = start_all.elapsed().as_millis() as u64;

    // Event selesai
    let _ = app.emit(
        "transfer://progress",
        &TransferProgress {
            items_done: report.items_done,
            items_error: report.items_error,
            items_skipped: report.items_skipped,
            items_total,
            bytes_transferred: report.total_bytes_transferred,
            per_dest: build_per_dest(&per_dest_total, &per_dest_done, &per_dest_error),
            current_source: String::new(),
            cancelled: report.cancelled,
        },
    );

    Ok(report)
}

fn build_per_dest(
    total: &std::collections::HashMap<PathBuf, usize>,
    done: &std::collections::HashMap<PathBuf, usize>,
    error: &std::collections::HashMap<PathBuf, bool>,
) -> Vec<DestProgressEntry> {
    let mut roots: Vec<&PathBuf> = total.keys().collect();
    roots.sort();
    roots
        .into_iter()
        .map(|r| DestProgressEntry {
            dest_root: r.to_string_lossy().to_string(),
            done: done.get(r).copied().unwrap_or(0),
            total: total.get(r).copied().unwrap_or(0),
            has_error: error.get(r).copied().unwrap_or(false),
        })
        .collect()
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
    // Windows: GetDiskFreeSpaceExW via FFI (tanpa dependensi tambahan)
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetDiskFreeSpaceExW(
                lpdirectoryname: *const u16,
                lpfreebytesavailabletocaller: *mut u64,
                lptotalnumberofbytes: *mut u64,
                lptotalnumberoffreebytes: *mut u64,
            ) -> i32;
        }

        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut free_avail: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free: u64 = 0;

        let ok = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_avail,
                &mut total_bytes,
                &mut total_free,
            )
        };

        if ok == 0 {
            return Ok(DiskSpaceInfo {
                path,
                available_bytes: 0,
                total_bytes: 0,
            });
        }

        Ok(DiskSpaceInfo {
            path,
            available_bytes: free_avail,
            total_bytes,
        })
    }

    // Non-Windows: fs4 jika tersedia, selain itu 0
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
