//! Sanitasi nama folder dan path panjang Windows

use crate::error::{Error, Result};
use regex::Regex;
use once_cell::sync::Lazy;

/// Reserved names Windows (case-insensitive)
static RESERVED: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
});

/// Karakter terlarang di nama file/folder Windows
static ILLEGAL_CHARS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[<>:"/\\|?*\x00-\x1F]"#).unwrap());

/// Karakter kontrol + whitespace berlebih
static MULTI_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

/// Batas panjang nama segmen
pub const MAX_SEGMENT_LEN: usize = 150;

/// Sanitasi nama kegiatan/suffix agar valid sebagai nama folder
pub fn sanitize_name(input: &str) -> String {
    // 1. Ganti karakter ilegal menjadi spasi
    let s = ILLEGAL_CHARS.replace_all(input, " ");

    // 2. Rapikan spasi berlebih
    let s = MULTI_SPACE.replace_all(&s, " ");

    // 3. Trim spasi dan titik
    let s = s.trim().trim_matches(|c: char| c == '.' || c.is_whitespace());

    // 4. Potong panjang
    let s = if s.chars().count() > MAX_SEGMENT_LEN {
        s.chars().take(MAX_SEGMENT_LEN).collect::<String>()
    } else {
        s.to_string()
    };

    // 5. Hindari reserved name
    if is_reserved(&s) {
        format!("{s}_")
    } else {
        s
    }
}

/// Cek apakah nama adalah reserved name Windows
pub fn is_reserved(name: &str) -> bool {
    let upper = name.to_uppercase();
    RESERVED.contains(&upper.as_str())
}

/// Format nama folder: MM-DD_Kegiatan[_Suffix]
pub fn build_folder_name(date_month: u32, date_day: u32, activity: &str, suffix: Option<&str>) -> Result<String> {
    let act = sanitize_name(activity);
    if act.is_empty() {
        return Err(Error::InvalidPath(
            "Nama kegiatan kosong setelah sanitasi".into(),
        ));
    }

    let base = format!("{:02}-{:02}_{}", date_month, date_day, act);

    let name = match suffix {
        Some(s) if !sanitize_name(s).is_empty() => format!("{}_{}", base, sanitize_name(s)),
        _ => base,
    };

    Ok(name)
}

/// Tambahkan prefix long-path Windows (\\?\)
pub fn long_path(p: &std::path::Path) -> std::path::PathBuf {
    #[cfg(windows)]
    {
        let s = p.to_string_lossy();
        if s.starts_with("\\\\?\\") || s.starts_with("\\\\") {
            return std::path::PathBuf::from(s.as_ref());
        }
        if p.is_absolute() {
            return std::path::PathBuf::from(format!("\\\\?\\{}", s));
        }
    }
    p.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_illegal() {
        assert_eq!(sanitize_name("Rapat: Koordinasi?"), "Rapat Koordinasi");
    }

    #[test]
    fn test_sanitize_reserved() {
        assert!(sanitize_name("CON").ends_with('_'));
    }

    #[test]
    fn test_build_folder_name() {
        let s = build_folder_name(3, 5, "Pelatihan Petani", Some("Hari 2")).unwrap();
        assert_eq!(s, "03-05_Pelatihan Petani_Hari 2");
    }

    #[test]
    fn test_build_folder_name_no_suffix() {
        let s = build_folder_name(11, 9, "Kunjungan", None).unwrap();
        assert_eq!(s, "11-09_Kunjungan");
    }

    #[test]
    fn test_long_path() {
        #[cfg(windows)]
        {
            let p = std::path::Path::new("C:\\Windows\\System32");
            assert!(long_path(p).to_string_lossy().starts_with("\\\\?\\"));
        }
    }
}
