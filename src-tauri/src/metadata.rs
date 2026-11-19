//! Ekstraksi metadata tanggal dari EXIF (foto) dan QuickTime/MP4 (video)

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use exif as kamadak_exif;

/// Asal tanggal yang diperoleh
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MetadataSource {
    Exif,
    Video,
    FileCreation,
    FileModified,
    Manual,
}

/// Informasi tanggal yang diekstrak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateMetadata {
    pub date: NaiveDate,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub source: MetadataSource,
    pub needs_review: bool,
    pub raw: Option<String>,
}

/// Ekstrak tanggal dari file foto/video dengan hirarki prioritas
pub fn extract_date(path: &Path, file_created: Option<i64>, file_modified: Option<i64>) -> DateMetadata {
    // 1. Coba baca EXIF (foto)
    if let Some(d) = extract_exif_date(path) {
        return DateMetadata {
            date: d.date(),
            year: d.year(),
            month: d.month(),
            day: d.day(),
            source: MetadataSource::Exif,
            needs_review: false,
            raw: Some(d.to_string()),
        };
    }

    // 2. Coba baca mvhd video MP4/MOV
    if let Some(d) = extract_video_date(path) {
        return DateMetadata {
            date: d.date(),
            year: d.year(),
            month: d.month(),
            day: d.day(),
            source: MetadataSource::Video,
            needs_review: false,
            raw: Some(d.to_string()),
        };
    }

    // 3. Fallback: creation time file
    if let Some(ts) = file_created {
        if let Some(dt) = DateTime::from_timestamp(ts, 0) {
            let ndt = dt.naive_local();
            if is_valid_year(ndt.year()) {
                return DateMetadata {
                    date: ndt.date(),
                    year: ndt.year(),
                    month: ndt.month(),
                    day: ndt.day(),
                    source: MetadataSource::FileCreation,
                    needs_review: true,
                    raw: Some(ndt.to_string()),
                };
            }
        }
    }

    // 4. Fallback: modified time file
    if let Some(ts) = file_modified {
        if let Some(dt) = DateTime::from_timestamp(ts, 0) {
            let ndt = dt.naive_local();
            if is_valid_year(ndt.year()) {
                return DateMetadata {
                    date: ndt.date(),
                    year: ndt.year(),
                    month: ndt.month(),
                    day: ndt.day(),
                    source: MetadataSource::FileModified,
                    needs_review: true,
                    raw: Some(ndt.to_string()),
                };
            }
        }
    }

    // 5. Fallback terakhir: hari ini (tetap ditandai butuh review)
    let today = Utc::now().naive_local().date();
    DateMetadata {
        date: today,
        year: today.year(),
        month: today.month(),
        day: today.day(),
        source: MetadataSource::FileModified,
        needs_review: true,
        raw: None,
    }
}

fn is_valid_year(y: i32) -> bool {
    (1970..=2100).contains(&y)
}

/// Ekstrak tanggal dari EXIF (JPEG, TIFF, WebP, PNG)
fn extract_exif_date(path: &Path) -> Option<NaiveDateTime> {
    let file = File::open(path).ok()?;
    let mut buf = BufReader::new(file);
    let reader = kamadak_exif::Reader::new();
    let exif = reader.read_from_container(&mut buf).ok()?;

    // Prioritas tag: DateTimeOriginal -> DateTimeDigitized -> DateTime
    let tags = [
        kamadak_exif::Tag::DateTimeOriginal,
        kamadak_exif::Tag::DateTimeDigitized,
        kamadak_exif::Tag::DateTime,
    ];

    for tag in tags {
        if let Some(field) = exif.get_field(tag, kamadak_exif::In::PRIMARY) {
            if let kamadak_exif::Value::Ascii(ref vec) = field.value {
                if let Some(bytes) = vec.first() {
                    let s = String::from_utf8_lossy(bytes);
                    if let Some(dt) = parse_exif_date_str(&s) {
                        if is_valid_year(dt.year()) {
                            return Some(dt);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Parse "YYYY:MM:DD HH:MM:SS"
fn parse_exif_date_str(s: &str) -> Option<NaiveDateTime> {
    let clean = s.trim().trim_matches('\0');
    NaiveDateTime::parse_from_str(clean, "%Y:%m:%d %H:%M:%S").ok()
}

/// Ekstrak tanggal dari container ISO BMFF (MP4, MOV) via atom `mvhd`
fn extract_video_date(path: &Path) -> Option<NaiveDateTime> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    if !["mp4", "mov", "m4v"].contains(&ext.as_str()) {
        return None;
    }

    let mut f = File::open(path).ok()?;
    use std::io::Read;

    // Scan sederhana untuk box 'moov' lalu 'mvhd' di 20 MB pertama
    let mut buf = vec![0u8; 1024 * 1024 * 4];
    let n = f.read(&mut buf).ok()?;
    let buf = &buf[..n];

    // Cari marker "mvhd"
    let mvhd_marker = b"mvhd";
    let pos = buf.windows(4).position(|w| w == mvhd_marker)?;

    // mvhd layout:
    // [4 bytes size][4 bytes 'mvhd'][1 byte version][3 bytes flags]
    // version 0: 4 bytes creation_time (detik sejak 1904-01-01)
    // version 1: 8 bytes creation_time
    let data_start = pos + 4; // setelah 'mvhd'
    if data_start + 8 > buf.len() {
        return None;
    }

    let version = buf[data_start];
    let time_offset = data_start + 4; // lewat version (1) + flags (3)

    let secs_since_1904: u64 = if version == 0 {
        if time_offset + 4 > buf.len() {
            return None;
        }
        let bytes: [u8; 4] = buf[time_offset..time_offset + 4].try_into().ok()?;
        u32::from_be_bytes(bytes) as u64
    } else {
        if time_offset + 8 > buf.len() {
            return None;
        }
        let bytes: [u8; 8] = buf[time_offset..time_offset + 8].try_into().ok()?;
        u64::from_be_bytes(bytes)
    };

    // Beda antara 1904-01-01 dan 1970-01-01 = 2.082.844.800 detik
    const MAC_EPOCH_DIFF: u64 = 2_082_844_800;
    if secs_since_1904 < MAC_EPOCH_DIFF {
        return None;
    }
    let unix_secs = (secs_since_1904 - MAC_EPOCH_DIFF) as i64;
    let dt = DateTime::from_timestamp(unix_secs, 0)?;
    let local = dt.naive_local();

    if is_valid_year(local.year()) {
        Some(local)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_exif_date() {
        let parsed = parse_exif_date_str("2026:03:15 09:41:22").unwrap();
        assert_eq!(parsed.year(), 2026);
        assert_eq!(parsed.month(), 3);
        assert_eq!(parsed.day(), 15);
    }
}
