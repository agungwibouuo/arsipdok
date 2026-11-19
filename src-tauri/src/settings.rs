//! Pengaturan aplikasi yang tersimpan di %APPDATA%\ArsipDok\settings.json

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Satu baris tujuan (root + nama kegiatan + suffix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destination {
    #[serde(default)]
    pub id: String,
    pub root: PathBuf,
    pub activity: String,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Mode transfer
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransferMode {
    Copy,
    Move,
}

/// Mode verifikasi
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMode {
    SizeMtime,
    Blake3,
}

/// Konfigurasi pengaturan lengkap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub version: u32,
    #[serde(default)]
    pub destinations: Vec<Destination>,
    #[serde(default)]
    pub root_history: Vec<PathBuf>,
    #[serde(default)]
    pub activity_history: Vec<String>,
    #[serde(default)]
    pub suffix_history: Vec<String>,
    #[serde(default = "default_mode")]
    pub transfer_mode: TransferMode,
    #[serde(default = "default_verification")]
    pub verification: VerificationMode,
    #[serde(default = "default_workers")]
    pub parallel_workers: u8,
    #[serde(default = "default_true")]
    pub remove_empty_source_dirs: bool,
    #[serde(default = "default_true")]
    pub confirm_before_start: bool,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_mode() -> TransferMode {
    TransferMode::Copy
}

fn default_verification() -> VerificationMode {
    VerificationMode::SizeMtime
}

fn default_workers() -> u8 {
    4
}

fn default_language() -> String {
    "id".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 2,
            destinations: Vec::new(),
            root_history: Vec::new(),
            activity_history: Vec::new(),
            suffix_history: Vec::new(),
            transfer_mode: default_mode(),
            verification: default_verification(),
            parallel_workers: default_workers(),
            remove_empty_source_dirs: true,
            confirm_before_start: true,
            language: default_language(),
        }
    }
}

impl Settings {
    /// Muat dari file; jika tidak ada, kembalikan default
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)?;
        let s: Settings = serde_json::from_str(&data)?;
        Ok(s)
    }

    /// Simpan ke file
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Validasi pengaturan
    pub fn validate(&self) -> Result<()> {
        if self.parallel_workers == 0 || self.parallel_workers > 8 {
            return Err(crate::error::Error::Other(
                "parallel_workers harus 1–8".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_workers_valid() {
        assert!(Settings::default().validate().is_ok());
    }

    #[test]
    fn serialization_roundtrip() {
        let s = Settings::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert_eq!(back.parallel_workers, s.parallel_workers);
    }
}
