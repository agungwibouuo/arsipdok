//! Engine transfer: copy streaming dengan verifikasi dan mode Move

use crate::error::{Error, Result};
use crate::planner::PlanItem;
use crate::settings::{TransferMode, VerificationMode};
use crate::verifier;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Status per item
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Pending,
    Copying,
    Verifying,
    Done,
    Skipped,
    Duplicate,
    Error,
    Partial,
}

/// Hasil transfer per item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    pub item: PlanItem,
    pub status: ItemStatus,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub final_path: Option<PathBuf>,
}

/// Konfigurasi engine
pub struct TransferConfig {
    pub mode: TransferMode,
    pub verification: VerificationMode,
    pub buffer_size: usize,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            mode: TransferMode::Copy,
            verification: VerificationMode::SizeMtime,
            buffer_size: 4 * 1024 * 1024,
        }
    }
}

/// Eksekusi transfer satu item (TANPA menghapus sumber — penghapusan sumber
/// dilakukan di level pemanggil setelah semua tujuan berhasil).
pub fn execute_item(item: &PlanItem, cfg: &TransferConfig, cancel: &Arc<AtomicBool>) -> TransferResult {
    let start = std::time::Instant::now();

    if cancel.load(Ordering::Relaxed) {
        return TransferResult {
            item: item.clone(),
            status: ItemStatus::Error,
            duration_ms: 0,
            error: Some("Dibatalkan".into()),
            final_path: None,
        };
    }

    // Pastikan folder tujuan ada
    if let Some(parent) = item.dest_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return fail(item, start, format!("Gagal buat folder tujuan: {e}"));
        }
    }

    // Cek konflik
    let final_path = match resolve_conflict(&item.dest_path) {
        Ok(p) => p,
        Err(e) => return fail(item, start, format!("Konflik: {e}")),
    };

    // Duplikat tepat (isi identik)
    if item.dest_path.exists() {
        let same_size = std::fs::metadata(&item.dest_path)
            .map(|m| m.len() == item.size_bytes)
            .unwrap_or(false);

        if same_size && cfg.verification == VerificationMode::Blake3 {
            if let Ok(true) = verifier::compare_blake3(&item.source, &item.dest_path) {
                return TransferResult {
                    item: item.clone(),
                    status: ItemStatus::Duplicate,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                    final_path: Some(item.dest_path.clone()),
                };
            }
        } else if same_size {
            // Default: ukuran sama dianggap duplikat
            return TransferResult {
                item: item.clone(),
                status: ItemStatus::Duplicate,
                duration_ms: start.elapsed().as_millis() as u64,
                error: None,
                final_path: Some(item.dest_path.clone()),
            };
        }
    }

    // Copy file ke .part lalu rename
    let part_path = with_extension(&final_path, "part");
    if let Err(e) = copy_stream(&item.source, &part_path, cfg.buffer_size, cancel) {
        let _ = std::fs::remove_file(&part_path);
        return fail(item, start, format!("Copy gagal: {e}"));
    }

    // Rename
    if let Err(e) = std::fs::rename(&part_path, &final_path) {
        let _ = std::fs::remove_file(&part_path);
        return fail(item, start, format!("Rename gagal: {e}"));
    }

    // Verifikasi
    if let Err(e) = verifier::verify(&item.source, &final_path, cfg.verification) {
        return fail(item, start, format!("Verifikasi gagal: {e}"));
    }

    // CATATAN: sumber TIDAK dihapus di sini. Mode Move menghapus sumber
    // hanya jika semua tujuan untuk file ini sudah sukses (lihat `delete_source`).
    TransferResult {
        item: item.clone(),
        status: ItemStatus::Done,
        duration_ms: start.elapsed().as_millis() as u64,
        error: None,
        final_path: Some(final_path),
    }
}

/// Hapus sumber setelah semua tujuan sukses (khusus mode Move).
pub fn delete_source(path: &Path) -> Result<()> {
    std::fs::remove_file(path)?;
    Ok(())
}

/// Hapus folder kosong di bawah root (dari dalam ke luar, root itu sendiri tidak dihapus).
pub fn remove_empty_dirs(root: &Path) -> Result<usize> {
    if !root.exists() {
        return Ok(0);
    }

    // Kumpulkan direktori, urutkan paling dalam dulu
    let mut dirs: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_path_buf())
        .collect();

    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

    let mut removed = 0usize;
    for d in dirs {
        // Hanya hapus jika benar-benar kosong
        let is_empty = std::fs::read_dir(&d)
            .map(|mut it| it.next().is_none())
            .unwrap_or(false);
        if is_empty && std::fs::remove_dir(&d).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

fn fail(item: &PlanItem, start: std::time::Instant, err: String) -> TransferResult {
    TransferResult {
        item: item.clone(),
        status: ItemStatus::Error,
        duration_ms: start.elapsed().as_millis() as u64,
        error: Some(err),
        final_path: None,
    }
}

/// Resolusi konflik nama: tambah (1), (2), dst.
fn resolve_conflict(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        return Ok(path.to_path_buf());
    }
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
    let parent = path.parent().ok_or_else(|| Error::InvalidPath("Parent kosong".into()))?;

    for n in 1..1000 {
        let candidate = if ext.is_empty() {
            parent.join(format!("{} ({})", stem, n))
        } else {
            parent.join(format!("{} ({}).{}", stem, n, ext))
        };
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(Error::Other("Konflik nama terlalu banyak".into()))
}

fn with_extension(p: &Path, ext: &str) -> PathBuf {
    let mut s = p.to_path_buf();
    let base = s
        .extension()
        .map(|e| format!("{}.{}", e.to_string_lossy(), ext))
        .unwrap_or_else(|| ext.to_string());
    s.set_extension(base);
    s
}

fn copy_stream(
    src: &Path,
    dst: &Path,
    buffer_size: usize,
    cancel: &Arc<AtomicBool>,
) -> Result<u64> {
    let mut input = File::open(src)?;
    let mut output = File::create(dst)?;
    let mut buf = vec![0u8; buffer_size];
    let mut total = 0u64;

    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        output.write_all(&buf[..n])?;
        total += n as u64;
    }
    output.sync_all()?;
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_ext_works() {
        let p = Path::new("/tmp/a.jpg");
        assert!(with_extension(p, "part").to_string_lossy().ends_with("a.jpg.part"));
    }
}
