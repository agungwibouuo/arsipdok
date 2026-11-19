//! Logger: log sesi transfer ke CSV + tampilan tabel

use crate::error::Result;
use crate::transfer::TransferResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Satu entri log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub mode: String,
    pub source_path: String,
    pub dest_root: String,
    pub dest_path: String,
    pub size_bytes: u64,
    pub status: String,
    pub metadata_source: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Log sesi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub file_path: PathBuf,
    pub entries: Vec<LogEntry>,
}

impl SessionLog {
    /// Buat sesi baru dengan nama file timestamp
    pub fn new(log_dir: &std::path::Path) -> Self {
        let ts = chrono::Local::now().format("%Y-%m-%d_%H%M%S");
        Self {
            file_path: log_dir.join(format!("{ts}.csv")),
            entries: Vec::new(),
        }
    }

    /// Tambah hasil transfer
    pub fn push(&mut self, r: &TransferResult, mode: &str, metadata_source: &str) {
        self.entries.push(LogEntry {
            timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
            mode: mode.to_string(),
            source_path: r.item.source.to_string_lossy().to_string(),
            dest_root: r.item.dest_root.to_string_lossy().to_string(),
            dest_path: r
                .final_path
                .clone()
                .unwrap_or_else(|| r.item.dest_path.clone())
                .to_string_lossy()
                .to_string(),
            size_bytes: r.item.size_bytes,
            status: format!("{:?}", r.status).to_lowercase(),
            metadata_source: metadata_source.to_string(),
            duration_ms: r.duration_ms,
            error: r.error.clone(),
        });
    }

    /// Catat kegagalan non-item (mis. hapus sumber gagal)
    pub fn push_error(&mut self, source: &std::path::Path, mode: &str, error: &str) {
        self.entries.push(LogEntry {
            timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
            mode: mode.to_string(),
            source_path: source.to_string_lossy().to_string(),
            dest_root: String::new(),
            dest_path: String::new(),
            size_bytes: 0,
            status: "error".to_string(),
            metadata_source: "unknown".to_string(),
            duration_ms: 0,
            error: Some(error.to_string()),
        });
    }

    /// Simpan ke CSV
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut wtr = csv::Writer::from_path(&self.file_path)?;
        for e in &self.entries {
            wtr.serialize(e)?;
        }
        wtr.flush()?;
        Ok(())
    }

    /// Ekspor ke path lain
    pub fn export_csv(&self, path: &std::path::Path) -> Result<()> {
        let mut wtr = csv::Writer::from_path(path)?;
        for e in &self.entries {
            wtr.serialize(e)?;
        }
        wtr.flush()?;
        Ok(())
    }

    /// Ringkasan jumlah per status
    pub fn summary(&self) -> serde_json::Value {
        let mut done = 0;
        let mut dup = 0;
        let mut err = 0;
        let mut skipped = 0;
        let mut bytes = 0u64;

        for e in &self.entries {
            match e.status.as_str() {
                "done" => { done += 1; bytes += e.size_bytes; }
                "duplicate" => dup += 1,
                "error" | "partial" => err += 1,
                "skipped" => skipped += 1,
                _ => {}
            }
        }

        serde_json::json!({
            "total": self.entries.len(),
            "done": done,
            "duplicate": dup,
            "error": err,
            "skipped": skipped,
            "bytes": bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_file_name_format() {
        let s = SessionLog::new(std::path::Path::new("C:/tmp"));
        let name = s.file_path.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.ends_with(".csv"));
        assert!(name.len() >= 20); // YYYY-MM-DD_HHMMSS.csv minimal 19+4
    }
}
