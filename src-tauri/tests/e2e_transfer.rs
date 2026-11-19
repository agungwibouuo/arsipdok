use arsipdok_lib::{metadata, planner, scanner, settings, transfer};
use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::AtomicBool;

#[test]
fn test_end_to_end_pipeline() {
    let tmp = std::env::temp_dir().join(format!("arsipdok_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let src = tmp.join("source_card");
    let dest_a = tmp.join("nas_a");
    let dest_b = tmp.join("disk_b");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&dest_a).unwrap();
    fs::create_dir_all(&dest_b).unwrap();

    // 1. Buat file sumber (2 foto valid, 1 text yang harus di-skip)
    let img1 = src.join("DSC_0001.JPG");
    let img2 = src.join("VID_0002.mp4");
    let txt = src.join("notes.txt");
    fs::write(&img1, b"fake photo data 1").unwrap();
    fs::write(&img2, b"fake video data 2").unwrap();
    fs::write(&txt, b"ignore this").unwrap();

    // 2. Scan
    let scan = scanner::scan_source(&src).unwrap();
    assert_eq!(scan.files.len(), 2, "Hanya foto & video yang terpindai");
    assert_eq!(scan.skipped_count, 1, "notes.txt harus di-skip");

    // 3. Extract metadata
    let mut dates = BTreeMap::new();
    for f in &scan.files {
        let meta = metadata::extract_date(&f.path, f.created, f.modified);
        assert!(meta.needs_review, "Timestamp file harus ditandai review");
        dates.insert(f.path.clone(), meta);
    }

    // 4. Overwrite tanggal manual untuk salah satu file
    dates.insert(
        img1.clone(),
        metadata::DateMetadata {
            date: chrono::NaiveDate::from_ymd_opt(2026, 8, 17).unwrap(),
            year: 2026,
            month: 8,
            day: 17,
            source: metadata::MetadataSource::Manual,
            needs_review: false,
            raw: Some("2026-08-17".into()),
        },
    );

    // 5. Build plan dengan multi-destination
    let destinations = vec![
        settings::Destination {
            id: "d1".into(),
            root: dest_a.clone(),
            activity: "HUT RI".into(),
            suffix: Some("Kamera A".into()),
            enabled: true,
        },
        settings::Destination {
            id: "d2".into(),
            root: dest_b.clone(),
            activity: "Dokumentasi".into(),
            suffix: None,
            enabled: true,
        },
    ];

    let plan = planner::build_plan(&scan.files, &dates, &destinations).unwrap();
    assert_eq!(plan.total_files, 2);
    assert_eq!(plan.items.len(), 4, "2 file x 2 tujuan = 4 item");

    // Cek format nama folder
    // d1: "08-17_HUT RI_Kamera A"
    let d1_item = plan.items.iter().find(|i| i.source == img1 && i.dest_root == dest_a).unwrap();
    assert_eq!(
        d1_item.dest_path.parent().unwrap().file_name().unwrap().to_str().unwrap(),
        "08-17_HUT RI_Kamera A"
    );

    // 6. Eksekusi transfer (mode Copy + size_mtime)
    let cancel = std::sync::Arc::new(AtomicBool::new(false));
    let cfg = transfer::TransferConfig {
        mode: settings::TransferMode::Copy,
        verification: settings::VerificationMode::SizeMtime,
        buffer_size: 64 * 1024,
    };

    for item in &plan.items {
        let res = transfer::execute_item(item, &cfg, &cancel);
        assert_eq!(res.status, transfer::ItemStatus::Done);
        assert!(res.final_path.as_ref().unwrap().exists());
    }

    // Verifikasi sumber masih ada (mode copy)
    assert!(img1.exists());
    assert!(img2.exists());

    // 7. Uji duplikasi identik: jalankan ulang item yang sama → harus di-skip
    let res_dup = transfer::execute_item(&plan.items[0], &cfg, &cancel);
    assert_eq!(res_dup.status, transfer::ItemStatus::Duplicate);

    // 8. Uji mode Move + BLAKE3
    let cfg_move = transfer::TransferConfig {
        mode: settings::TransferMode::Move,
        verification: settings::VerificationMode::Blake3,
        buffer_size: 64 * 1024,
    };
    let dest_c = tmp.join("move_target");
    fs::create_dir_all(&dest_c).unwrap();
    let move_dests = vec![settings::Destination {
        id: "d3".into(),
        root: dest_c.clone(),
        activity: "Pindah".into(),
        suffix: None,
        enabled: true,
    }];
    let plan_move = planner::build_plan(&scan.files, &dates, &move_dests).unwrap();
    for it in &plan_move.items {
        let res = transfer::execute_item(it, &cfg_move, &cancel);
        assert_eq!(res.status, transfer::ItemStatus::Done);
    }
    // Hapus sumber setelah verifikasi semua tujuan sukses
    transfer::delete_source(&img1).unwrap();
    assert!(!img1.exists(), "Sumber harus terhapus setelah mode Move sukses");

    // Cleanup
    let _ = fs::remove_dir_all(&tmp);
}
