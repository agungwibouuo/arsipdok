//! Scanner rekursif: cari file foto/video di folder sumber

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Ekstensi foto default
pub const PHOTO_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "heic", "heif", "webp", "tif", "tiff", "cr2", "cr3", "nef", "arw", "dng",
];

/// Ekstensi video default
pub const VIDEO_EXTS: &[&str] = &[
    "mp4", "mov", "m4v", "avi", "mkv", "mts", "m2ts", "3gp", "wmv", "flv", "webm", "mpg", "mpeg",
];

/// Tipe file
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Photo,
    Video,
}

/// File yang ditemukan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub extension: String,
    pub kind: FileKind,
    pub modified: Option<i64>,
    pub created: Option<i64>,
}

impl ScannedFile {
    /// Cek apakah file merupakan foto/video berdasarkan ekstensi
    pub fn classify(path: &Path) -> Option<FileKind> {
        let ext = path.extension()?.to_string_lossy().to_lowercase();
        if PHOTO_EXTS.contains(&ext.as_str()) {
            Some(FileKind::Photo)
        } else if VIDEO_EXTS.contains(&ext.as_str()) {
            Some(FileKind::Video)
        } else {
            None
        }
    }
}

/// Hasil scan per sumber
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub source_path: PathBuf,
    pub files: Vec<ScannedFile>,
    pub skipped_count: usize,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Scan folder rekursif untuk file foto/video
pub fn scan_source(source: &Path) -> Result<ScanResult> {
    let start = std::time::Instant::now();

    if !source.exists() {
        return Ok(ScanResult {
            source_path: source.to_path_buf(),
            files: Vec::new(),
            skipped_count: 0,
            duration_ms: 0,
            error: Some(format!("Path tidak ditemukan: {}", source.display())),
        });
    }

    let mut files = Vec::new();
    let mut skipped = 0usize;

    for entry in WalkDir::new(source)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let Some(kind) = ScannedFile::classify(path) else {
            skipped += 1;
            continue;
        };

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        // File 0 byte diabaikan
        if meta.len() == 0 {
            skipped += 1;
            continue;
        }

        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);
        let created = meta
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        files.push(ScannedFile {
            path: path.to_path_buf(),
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            size_bytes: meta.len(),
            extension: path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default(),
            kind,
            modified,
            created,
        });
    }

    // Urutkan berdasarkan path agar deterministik
    files.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(ScanResult {
        source_path: source.to_path_buf(),
        files,
        skipped_count: skipped,
        duration_ms: start.elapsed().as_millis() as u64,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_photo() {
        assert_eq!(ScannedFile::classify(Path::new("a.JPG")), Some(FileKind::Photo));
        assert_eq!(ScannedFile::classify(Path::new("a.heic")), Some(FileKind::Photo));
    }

    #[test]
    fn classify_video() {
        assert_eq!(ScannedFile::classify(Path::new("v.mp4")), Some(FileKind::Video));
        assert_eq!(ScannedFile::classify(Path::new("v.MOV")), Some(FileKind::Video));
    }

    #[test]
    fn classify_other() {
        assert_eq!(ScannedFile::classify(Path::new("doc.pdf")), None);
    }
}
