//! Planner: hitung rencana transfer (file → folder tujuan) untuk multi-tujuan

use crate::error::Result;
use crate::metadata::DateMetadata;
use crate::paths::build_folder_name;
use crate::scanner::ScannedFile;
use crate::settings::Destination;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Satu item rencana transfer (1 file × 1 tujuan)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub source: PathBuf,
    pub file_name: String,
    pub size_bytes: u64,
    pub dest_root: PathBuf,
    pub dest_folder: String,
    pub dest_path: PathBuf,
    pub date: String,
    pub month: u32,
    pub day: u32,
}

/// Rencana transfer lengkap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPlan {
    pub items: Vec<PlanItem>,
    pub total_files: usize,
    pub total_bytes: u64,
    pub folder_names: Vec<String>,
}

/// Bangun rencana dari daftar file + daftar tujuan
pub fn build_plan(files: &[ScannedFile], dates: &BTreeMap<PathBuf, DateMetadata>, destinations: &[Destination]) -> Result<TransferPlan> {
    let mut items = Vec::new();
    let mut folder_names = std::collections::BTreeSet::new();
    let mut total_bytes = 0u64;

    for f in files {
        let Some(meta) = dates.get(&f.path) else { continue };

        for d in destinations.iter().filter(|d| d.enabled) {
            let folder = build_folder_name(meta.month, meta.day, &d.activity, d.suffix.as_deref())?;
            let dest_dir = d.root.join(&folder);
            let dest_path = dest_dir.join(&f.name);

            folder_names.insert(folder.clone());
            total_bytes += f.size_bytes;

            items.push(PlanItem {
                source: f.path.clone(),
                file_name: f.name.clone(),
                size_bytes: f.size_bytes,
                dest_root: d.root.clone(),
                dest_folder: folder,
                dest_path,
                date: meta.date.format("%Y-%m-%d").to_string(),
                month: meta.month,
                day: meta.day,
            });
        }
    }

    Ok(TransferPlan {
        total_files: files.len(),
        total_bytes,
        folder_names: folder_names.into_iter().collect(),
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::MetadataSource;
    use chrono::NaiveDate;
    use crate::scanner::FileKind;

    fn mkfile(name: &str, size: u64) -> ScannedFile {
        ScannedFile {
            path: PathBuf::from(format!("E:\\DCIM\\{name}")),
            name: name.to_string(),
            size_bytes: size,
            extension: "jpg".into(),
            kind: FileKind::Photo,
            modified: None,
            created: None,
        }
    }

    fn mkmeta(month: u32, day: u32) -> DateMetadata {
        DateMetadata {
            date: NaiveDate::from_ymd_opt(2026, month, day).unwrap(),
            year: 2026,
            month,
            day,
            source: MetadataSource::Exif,
            needs_review: false,
            raw: None,
        }
    }

    #[test]
    fn plan_multi_destination() {
        let files = vec![mkfile("a.jpg", 100), mkfile("b.jpg", 200)];
        let mut dates = BTreeMap::new();
        dates.insert(PathBuf::from("E:\\DCIM\\a.jpg"), mkmeta(3, 15));
        dates.insert(PathBuf::from("E:\\DCIM\\b.jpg"), mkmeta(3, 17));

        let dests = vec![
            Destination {
                id: "d1".into(),
                root: PathBuf::from("D:\\Dokumentasi"),
                activity: "Pelatihan".into(),
                suffix: Some("Hari 2".into()),
                enabled: true,
            },
            Destination {
                id: "d2".into(),
                root: PathBuf::from("E:\\Arsip"),
                activity: "Pelatihan".into(),
                suffix: None,
                enabled: true,
            },
        ];

        let plan = build_plan(&files, &dates, &dests).unwrap();
        assert_eq!(plan.items.len(), 4); // 2 file × 2 tujuan
        assert_eq!(plan.total_bytes, 600);
        assert_eq!(plan.folder_names.len(), 4); // 2 tanggal × 2 tujuan
    }

    #[test]
    fn plan_disabled_destination_excluded() {
        let files = vec![mkfile("a.jpg", 10)];
        let mut dates = BTreeMap::new();
        dates.insert(PathBuf::from("E:\\DCIM\\a.jpg"), mkmeta(3, 15));

        let dests = vec![Destination {
            id: "d1".into(),
            root: PathBuf::from("D:\\X"),
            activity: "Keg".into(),
            suffix: None,
            enabled: false,
        }];

        let plan = build_plan(&files, &dates, &dests).unwrap();
        assert!(plan.items.is_empty());
    }
}
