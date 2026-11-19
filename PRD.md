# PRD — ArsipDok

**Aplikasi Desktop Windows untuk Sortir & Transfer Dokumentasi Foto/Video**

| Item | Keterangan |
|---|---|
| Nama Produk | ArsipDok |
| Versi Dokumen | 1.1 |
| Tanggal | 19 November 2026 |
| Platform | Windows 10/11 x64 |
| Teknologi | Tauri 2 + Rust (frontend web) |
| Status | Implementasi — Fase 0.5 Selesai (Scaffold Tauri 2 & Core Modules) |

---

## 1. Ringkasan Eksekutif

ArsipDok adalah aplikasi desktop Windows single-user (tanpa login) untuk **memindahkan atau menyalin** file dokumentasi (foto & video) dari sumber eksternal — SD card, NAS, atau drive lokal — ke folder arsip yang terstruktur.

Folder tujuan dibuat **otomatis** dengan format:

```text
MM-DD_Nama Kegiatan[_Suffix]
```

Tanggal diambil dari **metadata file** (EXIF untuk foto, creation time untuk video), bukan tanggal proses transfer. Transfer dilakukan oleh engine internal aplikasi (bukan dialog copy-paste Explorer) dengan verifikasi sebelum file sumber dihapus pada mode Move.

---

## 2. Latar Belakang & Problem Statement

### Kondisi saat ini
Operator dokumentasi memindahkan hasil foto/video dari SD card secara manual melalui Windows Explorer: buka drive, seleksi, copy, lalu rename folder satu per satu.

### Masalah
1. **Rawan salah tanggal** — folder dinamai berdasarkan tanggal proses, bukan tanggal pengambilan gambar.
2. **Penamaan tidak konsisten** — format folder berbeda antar operator/sesi.
3. **Risiko kehilangan data** — pada mode pindah manual, file bisa terhapus sebelum dipastikan tersalin utuh.
4. **Tidak ada jejak audit** — tidak ada catatan file apa yang dipindah ke mana dan kapan.
5. **Tidak efisien** — file dari beberapa tanggal harus dikelompokkan dan di-rename manual.

### Peluang
Satu alur kerja: **tarik sumber → preview hasil pengelompokan → eksekusi**, dengan penamaan otomatis dan verifikasi terjamin.

---

## 3. Tujuan & Metrik Keberhasilan

### Tujuan produk
| # | Tujuan |
|---|---|
| G-1 | Satu alur kerja untuk copy/move dokumentasi tanpa membuka Explorer. |
| G-2 | Penamaan dan pengelompokan folder 100% otomatis dan konsisten. |
| G-3 | Mode Move aman: file sumber hanya dihapus setelah terverifikasi. |
| G-4 | Menyediakan log audit per transaksi yang dapat diekspor. |
| G-5 | Mendukung beberapa sumber sekaligus (antrean) dalam satu sesi. |

### Metrik keberhasilan (target MVP)
| Metrik | Target |
|---|---|
| Kehilangan/korupsi data pada mode Move | **0 file** |
| Folder sesuai format `MM-DD_Kegiatan` | **100%** |
| Intervensi rename manual | **0** |
| Scan 10.000 file | ≤ 10 detik |
| Throughput transfer | ≥ 80% kecepatan baca/tulis disk |
| File dengan tanggal dari metadata | ≥ 95% (sisanya fallback ber-flag) |

---

## 4. Non-Tujuan (Out of Scope)

- Upload ke cloud / sinkronisasi online.
- Multi-user, login, role, atau server pusat.
- Editor/preview penuh foto & video (hanya info file).
- Mengubah/menulis metadata file.
- Auto-import otomatis saat SD card dicolok (kandidat fase lanjutan).
- Dukungan macOS/Linux (fokus Windows).
- Antarmuka multibahasa (fase ini Bahasa Indonesia saja).

---

## 5. Pengguna & Konteks Penggunaan

**Persona utama:** Staf/operator dokumentasi lapangan.

| Aspek | Deskripsi |
|---|---|
| Kemampuan teknis | Menengah ke bawah; terbiasa dengan File Explorer |
| Frekuensi | Harian/mingguan setelah kegiatan |
| Volume | Ratusan–ribuan file per sesi, 1–50 GB |
| Lingkungan | Laptop/PC Windows, kadang terhubung NAS kantor |
| Kebutuhan utama | Cepat, aman, tidak perlu mengingat aturan penamaan |

**Konteks:** Setelah kegiatan selesai, operator mencabut SD card dari kamera, mencolokkannya ke PC, lalu memindahkan seluruh dokumentasi ke arsip.

---

## 6. Lingkup Rilis

| Fitur | MVP (v1.0) | Fase Lanjutan |
|---|---|---|
| Drag & drop multi-sumber + browse | ✅ | — |
| Scan rekursif, filter foto/video | ✅ | — |
| Tanggal dari EXIF (JPEG/TIFF/PNG/WebP) | ✅ | — |
| Tanggal dari video MP4/MOV (mvhd) | ✅ | — |
| Fallback tanggal + flag review | ✅ | — |
| Koreksi tanggal manual per baris | ✅ | — |
| Grouping per tanggal (multi-folder) | ✅ | — |
| Nama kegiatan bebas + suffix | ✅ | Preset kegiatan |
| Copy / Move dengan verifikasi | ✅ | — |
| Verifikasi checksum (BLAKE3) opsional | ✅ | — |
| Penanganan konflik & duplikat | ✅ | Deteksi duplikat lintas arsip |
| Log tabel + ekspor CSV | ✅ | — |
| Antrean multi-sumber | ✅ | Persist antrean antar sesi |
| EXIF untuk HEIC/CR3 (BMFF) | ⚠️ fallback | ✅ parser khusus |
| Pertahankan struktur subfolder sumber | ❌ (ratakan) | ✅ opsi |
| Auto-eject SD setelah selesai | ❌ | ✅ |
| Multi-tujuan simultan (>1 tujuan per sesi) | ✅ (N baris tujuan) | — |
| Modal konfirmasi sebelum eksekusi | ✅ | — |
| Tooltip/editor tanggal manual inline | ✅ | — |
| Riwayat kegiatan & suffix (autocomplete) | ✅ | Preset tersimpan |

---

## 7. Alur Pengguna (User Flow)

```text
[Mulai]
   │
   ▼
1. Pilih/tarik sumber (SD card / folder / NAS / D:)  → tampil di antrean
   │
   ▼
2. Aplikasi scan otomatis → filter foto & video → baca metadata tanggal
   │
   ▼
3. Preview tabel file + tanggal + sumber tanggal + folder tujuan
   │      └─ baris fallback ditandai "Perlu Review" (bisa diedit)
   │
   ▼
4. Isi: Nama Kegiatan (wajib) + Suffix (opsional)
   │      └─ bisa TAMBAH beberapa tujuan: tiap baris = root + kegiatan + suffix
   │
   ▼
5. Pilih mode: Copy / Move  (berlaku untuk seluruh sesi)
   │
   ▼
6. Review ringkasan grup folder (jumlah file, total ukuran) per tujuan
   │
   ▼
6b. Klik "Mulai" → MUNCUL MODAL KONFIRMASI
   │      └─ ringkasan: jumlah file, total ukuran, jumlah folder, mode, verifikasi
   │      └─ mode Move: wajib centang "Saya memahami risiko"
   │
   ▼
7. Konfirmasi "Mulai Transfer" → transfer + verifikasi + (Move: hapus sumber)
   │
   ▼
8. Ringkasan hasil + log tabel + ekspor CSV
   │
   ▼
[Selesai]
```

**Prinsip UX:** sumber bisa masuk satu per satu atau sekaligus; tidak ada aksi destruktif tanpa konfirmasi; semua hasil terlihat sebelum eksekusi.

---

## 8. Requirement Fungsional

### FR-01 — Input Sumber
- Mendukung **drag & drop** folder, drive (mis. `E:\`), atau beberapa sekaligus ke dropzone.
- Mendukung tombol **Pilih Folder** (dialog native).
- Setiap sumber tampil sebagai item antrean (nama, path, status), bisa dihapus sebelum scan/eksekusi.
- Duplikasi path sumber yang sama diabaikan.

### FR-02 — Scan & Filter
- Scan **rekursif** seluruh subfolder sumber.
- Hanya file dengan ekstensi foto/video yang didaftarkan (lihat Lampiran A).
- Abaikan file hidden/system dan file 0 byte (dicatat di log sebagai *skipped*).
- Progres scan ditampilkan (jumlah file ditemukan, folder aktif).
- Scan dapat dibatalkan.

### FR-03 — Ekstraksi Tanggal
Prioritas sumber tanggal:

| Urutan | Sumber | Label di UI |
|---|---|---|
| 1 | EXIF `DateTimeOriginal` (foto) | `EXIF` |
| 2 | EXIF `DateTimeDigitized` | `EXIF` |
| 3 | EXIF `DateTime` | `EXIF` |
| 4 | Video `mvhd creation_time` (MP4/MOV) | `Video` |
| 5 | File creation time (Windows) | `File (fallback)` |
| 6 | File modified time | `File (fallback)` |

- Nilai tidak valid (0, tahun < 1970, tahun > 2100) dilewati ke prioritas berikutnya.
- Baris yang memakai fallback diberi **badge kuning "Perlu Review"**.
- Jika seluruh sumber gagal, file tetap ditampilkan dengan tanggal proses + badge merah.

### FR-04 — Koreksi Tanggal Manual
- User dapat mengubah tanggal pada baris mana pun melalui date picker di tabel.
- Perubahan langsung memengaruhi pengelompokan folder (real-time).
- Kolom sumber tanggal berubah menjadi `Manual` setelah diedit.

### FR-05 — Pengelompokan Folder & Multi-Tujuan
- File dikelompokkan berdasarkan **tanggal kalender** (tahun-bulan-hari), meskipun jam berbeda.
- **Multi-tujuan**: user dapat menambahkan N baris tujuan; setiap baris = root folder + nama kegiatan + suffix.
  - Contoh: `D:\Dokumentasi` + `Pelatihan Petani` + `Hari 2` → folder `03-15_Pelatihan Petani_Hari 2` di `D:\Dokumentasi`.
  - Contoh: `E:\Arsip Lapangan` + `Pelatihan Petani` + `Hari 2` → folder `03-15_Pelatihan Petani_Hari 2` di `E:\Arsip Lapangan`.
- **Setiap file dikirim ke SEMUA tujuan yang aktif** (duplikat di N lokasi).
- Satu sesi dengan N tujuan dapat menghasilkan **N × M folder** (N tujuan × M tanggal).
- Formula nama folder: `MM-DD_Nama Kegiatan[_Suffix]` (lihat §9).

### FR-06 — Sanitasi Nama
- Karakter terlarang Windows `\ / : * ? " < > |` diganti/dihapus.
- Spasi berlebih dirapikan; spasi/titik di awal-akhir dipangkas.
- Nama reserved Windows (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`) dihindari.
- Panjang nama folder dibatasi (≤ 150 karakter) untuk keamanan path.
- Jika sanitasi mengubah nama, hasil akhir ditampilkan di preview (tidak silent).

### FR-07 — Preview Tabel
Kolom tabel:

| Kolom | Keterangan |
|---|---|
| ☑ | Sertakan/keluarkan file dari transfer |
| Nama file | Nama + ekstensi |
| Tipe | Foto / Video |
| Ukuran | Human-readable |
| Tanggal | Tanggal efektif (dapat diedit) |
| Sumber tanggal | EXIF / Video / File (fallback) / Manual |
| Folder tujuan | Hasil nama folder relatif |

- Ringkasan grup: jumlah file + total ukuran per folder tujuan.
- Baris fallback disorot dan dapat difilter ("Tampilkan hanya Perlu Review").

### FR-08 — Mode Transfer
- Pilihan radio: **Copy** (default) dan **Move**.
- Memilih **Move** menampilkan peringatan eksplisit bahwa file sumber akan dihapus setelah verifikasi.
- Mode berlaku untuk seluruh sesi.

### FR-09 — Validasi Sebelum Mulai
Tombol "Mulai" hanya aktif jika:
- Minimal 1 file terpilih.
- Minimal 1 baris tujuan aktif.
- Semua baris tujuan aktif: Nama Kegiatan tidak kosong setelah sanitasi, root ada dan writable.
- Estimasi ruang kosong **setiap** root tujuan ≥ total ukuran file (dengan margin 5%).
- Tidak ada duplikasi kombinasi (root + kegiatan + suffix) antar baris tujuan.
- Tidak ada konflik nama folder yang tidak dapat diselesaikan.

### FR-09b — Modal Konfirmasi sebelum Mulai
- Menekan **Mulai** tidak langsung mengeksekusi; tampil modal konfirmasi.
- Isi modal:
  - Jumlah file & total ukuran yang akan ditransfer.
  - Jumlah folder tujuan yang akan dibuat/digabung.
  - Daftar tujuan ringkas (root + nama folder yang akan dibuat).
  - Mode aktif (Copy/Move) dan metode verifikasi.
- Jika mode **Move**: tampil blok peringatan amber + **checkbox wajib** "Saya memahami risiko dan siap melanjutkan"; tombol Mulai tetap disabled sampai dicentang.
- Tombol modal: **Batalkan** (menutup modal, kembali ke preview) dan **Mulai Transfer**.
- `Esc` menutup modal tanpa memulai. `Enter` memulai (jika valid).
- Modal menerima fokus keyboard penuh (focus trap).

### FR-10 — Engine Copy
- Copy berbasis stream chunked (buffer 1–4 MB), bukan load ke memori.
- Tulis ke file sementara `nama.ext.part`, lalu rename menjadi nama final setelah selesai.
- Setelah copy, timestamp modifikasi tujuan disamakan dengan sumber.
- Concurrency: default 4 worker (dapat dikonfigurasi 1–8).
- Progres (persen, byte, kecepatan, sisa waktu) dikirim ke UI maksimal tiap 100 ms agar UI tetap halus.
- Mendukung path panjang (> 260 karakter) dengan prefix `\\?\`.

### FR-11 — Verifikasi
- Default: **ukuran file sumber = tujuan** dan **timestamp modifikasi** sama.
- Opsional: **checksum BLAKE3** (lebih lambat; diaktifkan di pengaturan).
- File gagal verifikasi ditandai `error`, tidak dianggap selesai, dan tidak dihapus pada mode Move.
- Tersedia retry per file / untuk semua file gagal.

### FR-12 — Semantik Move
- Urutan wajib per file: **copy → verifikasi lulus → hapus sumber**.
- Jika verifikasi gagal atau copy terputus, file sumber **tidak disentuh**.
- Folder sumber yang menjadi kosong setelah move dapat dihapus (opsi, default: ya untuk subfolder kosong).
- Mode Move tidak pernah menggunakan operasi rename/move lintas volume secara langsung.

### FR-13 — Konflik & Duplikat
| Kasus | Perilaku |
|---|---|
| Folder tujuan sudah ada | Gabung (merge), tidak menghapus isi yang ada |
| File tujuan nama sama, isi identik (size + hash/mtime) | Lewati sebagai duplikat, dicatat di log |
| File tujuan nama sama, isi berbeda | Rename otomatis `nama (1).ext`, `nama (2).ext`, dst.; dicatat sebagai konflik |
| Nama folder bentrok akibat sanitasi | Tampilkan di preview, minta konfirmasi |

### FR-14 — Pembatalan
- Tombol **Batal** tersedia selama scan dan transfer.
- Pembatalan bersifat kooperatif: file aktif diselesaikan atau dihentikan dengan aman.
- File `.part` yang tertinggal dibersihkan.
- Pada mode Move, file yang belum terverifikasi tetap berada di sumber.

### FR-15 — Log & Audit
- Setiap sesi menghasilkan log terstruktur berisi **daftar file yang dipilih** beserta hasilnya.
- Log ditampilkan sebagai tabel di aplikasi setelah selesai.
- Ekspor ke **CSV** dan **TXT** (ringkasan).
- Log tersimpan di `%APPDATA%\ArsipDok\logs\YYYY-MM-DD_HHmmss.csv`.
- Kolom log: `timestamp, mode, source_path, dest_path, size_bytes, status, metadata_source, duration_ms, error`.

### FR-16 — Pengaturan
- Riwayat root tujuan (autocomplete) + daftar tujuan terakhir.
- Riwayat Nama Kegiatan dan Suffix (autocomplete).
- Mode verifikasi (size+mtime / BLAKE3).
- Jumlah worker (1–8).
- Tersimpan di `%APPDATA%\ArsipDok\settings.json` dan bertahan antar sesi.

### FR-17 — Multi-Sumber
- Beberapa sumber diproses **berurutan** dalam satu sesi.
- Satu log gabungan per sesi, dengan penanda sumber asal per file.
- Jika satu sumber gagal total (mis. card dicabut), sumber lain tetap dapat diproses.

### FR-18 — Multi-Tujuan Simultan
- Pengguna dapat menentukan lebih dari satu tujuan arsip dalam satu sesi transfer.
- Setiap baris tujuan memiliki:
  - **Root folder**: path absolut lokal atau UNC (mis. `D:\Dokumentasi`, `\\nas\arsip`).
  - **Nama Kegiatan**: teks bebas (wajib per baris).
  - **Suffix**: teks bebas opsional (mis. `Hari 2`, `Desa Sukamaju`).
- Tombol **+ Tambah Tujuan**: menambahkan baris baru dengan nilai default dari baris pertama.
- Tombol **Hapus**: menghapus baris (minimal tersisa 1 baris).
- Pada saat eksekusi, engine menduplikasi setiap file ke seluruh tujuan yang terdaftar.
- Pada mode **Move**: file sumber **hanya dihapus setelah berhasil terverifikasi di SEMUA tujuan**. Jika salah satu tujuan gagal, file sumber tidak dihapus.
- Progres transfer menampilkan status per tujuan dan status agregat.

### FR-19 — Tooltip & Editor Tanggal Inline
- Mengklik badge/kolom tanggal pada tabel preview membuka popup/tooltip editor kecil tepat di atas sel yang bersangkutan.
- Menampilkan:
  - Alasan fallback (mis. "EXIF tidak ada · menggunakan waktu file").
  - Date input (kalender mini).
  - Opsi cakupan: `(•) File ini saja` atau `( ) Semua file berstatus review`.
  - Tombol **Batal** dan **Terapkan**.
- Menerapkan tanggal langsung memperbarui kalkulasi folder tujuan file tersebut secara real-time.

---

## 9. Aturan Penamaan & Sanitasi

### Formula
```text
MM-DD_Nama Kegiatan[_Suffix]
```

| Komponen | Aturan |
|---|---|
| `MM` | Bulan 2 digit, **zero-pad** (`01`–`12`) |
| `DD` | Tanggal 2 digit, **zero-pad** (`01`–`31`) |
| `_` | Pemisah tetap |
| Nama Kegiatan | Teks bebas, wajib, disanitasi |
| Suffix | Opsional, teks bebas (mis. `Hari 2`, `Desa Sukamaju`, `Lokasi B`) |

### Contoh
| Input | Hasil Folder |
|---|---|
| 15 Maret, kegiatan "Rapat Koordinasi", tanpa suffix | `03-15_Rapat Koordinasi` |
| 15 Maret, kegiatan "Rapat Koordinasi", suffix "Hari 2" | `03-15_Rapat Koordinasi_Hari 2` |
| 2 Januari, kegiatan "Kunjungan", suffix "Desa Sukamaju" | `01-02_Kunjungan_Desa Sukamaju` |
| File dari 31 Mar & 1 Apr, kegiatan "Pelatihan" | `03-31_Pelatihan` dan `04-01_Pelatihan` |

### Sanitasi
1. Ganti karakter `\ / : * ? " < > |` dengan `-` (atau hapus).
2. Collapse spasi ganda menjadi satu.
3. Trim spasi dan titik di awal/akhir segmen.
4. Hindari nama reserved Windows.
5. Potong hingga ≤ 150 karakter, utamakan mempertahankan prefix tanggal.
6. Jika hasil kosong setelah sanitasi → tolak dengan pesan validasi.

---

## 10. Spesifikasi Metadata Tanggal

| Format | Metode | Tag/Field | Catatan |
|---|---|---|---|
| JPEG, TIFF, PNG, WebP | `kamadak-exif` | `DateTimeOriginal` → `DateTimeDigitized` → `DateTime` | Format `YYYY:MM:DD HH:MM:SS` |
| MP4, MOV, M4V | Parser container ISO BMFF | `mvhd.creation_time` | Epoch 1904-01-01, dalam detik |
| HEIC, HEIF, CR3 | — (MVP) | — | EXIF berada di dalam BMFF; MVP memakai fallback + flag review |
| AVI, MKV, MTS, lainnya | — | — | Fallback ke timestamp file + flag review |

### Kebijakan waktu
- Nilai metadata diperlakukan sebagai **waktu lokal apa adanya** (kamera umumnya menulis waktu lokal tanpa timezone).
- Nilai `0`, tanggal sebelum 1970, atau setelah 2100 dianggap tidak valid → lanjut ke fallback.
- Video dari kamera ponsel yang menulis UTC dapat berbeda beberapa jam; koreksi manual tersedia (FR-04).

---

## 11. Spesifikasi UI

### Layout utama

```text
┌──────────────────────────────────────────────────────────────────────┐
│  ArsipDok                                             [Pengaturan]   │
├──────────────────────────────────────────────────────────────────────┤
│  Sumber (antrean)                 Tujuan (bisa >1)                  │
│  ┌──────────────────────────┐     Folder     Kegiatan     Suffix  Hapus│
│  │  Tarik folder/SD card    │     [D:\Dok..] [Pelatihan]  [Hari 2]  🗑 │
│  │  ke sini (atau [Pilih])  │     [D:\Dok..] [Pelatihan]  [Desa..]  🗑 │
│  └──────────────────────────┘     [E:\Arsip] [Pelatihan]  [Hari 2]  🗑 │
│  • E:\DCIM\100ANDRO      [x]      [+ Tambah Tujuan]                  │
│  • \\nas\kamera\2026     [x]      Setiap file dikirim ke SEMUA tujuan │
│                                   Mode: (•) Copy   ( ) Move          │
├──────────────────────────────────────────────────────────────────────┤
│  Tabel Preview (☑ • nama • tipe • ukuran • tanggal ✎ • sumber •     │
│  folder tujuan) + filter [Semua] [Perlu Review] + pencarian          │
│  └─ klik tanggal → tooltip editor tanggal inline (FR-19)             │
├──────────────────────────────────────────────────────────────────────┤
│  Ringkasan: 3 folder • 6 file • 7.9 GB          [ Mulai ]            │
└──────────────────────────────────────────────────────────────────────┘
```

### Overlay & Dialog
| Overlay | Isi | Referensi |
|---|---|---|
| Pengaturan — Umum | Root default, riwayat tujuan, jumlah worker, bahasa | FR-16 |
| Pengaturan — Ekstensi | Daftar ekstensi foto/video yang diikutkan | §21.A |
| Pengaturan — Verifikasi | Mode size+mtime / BLAKE3, retry | FR-11 |
| Pengaturan — Lanjutan | Threshold timeout, hapus subfolder kosong, long-path | FR-12, FR-16 |
| **Modal Konfirmasi Mulai** | Ringkasan file/ukuran/folder/mode/verifikasi + checklist risiko Move | FR-09b |
| **Tooltip Edit Tanggal** | Popup di atas sel tanggal: alasan fallback, date picker, cakupan, Batal/Terapkan | FR-19 |

### State layar
| State | Tampilan |
|---|---|
| Kosong | Dropzone besar, tombol Mulai disabled |
| Scanning | Progress scan + jumlah file ditemukan |
| Ready | Tabel preview + ringkasan grup + panel tujuan multi-baris |
| Confirm | Modal konfirmasi sebelum mulai (overlay di atas Ready) |
| Running | Progress bar, file aktif, kecepatan, tombol Batal |
| Done | Ringkasan sukses/gagal, tabel log, ekspor CSV |
| Error | Pesan jelas + langkah pemulihan |

### Prinsip UI
- Bahasa Indonesia, istilah konsisten.
- Validasi inline (nama kegiatan wajib per baris tujuan, root tujuan writable).
- Aksi destruktif (Move) memakai warna peringatan + modal konfirmasi dengan checklist wajib.
- Status per file jelas: `Menunggu → Menyalin → Verifikasi → Selesai / Gagal`.
- Dapat dioperasikan dengan keyboard (tab order, Enter untuk mulai, Esc untuk batal).

### Identitas Visual
- Nama aplikasi: **ArsipDok** (wordmark "Arsip" gelap `#0F172A` + "Dok" biru `#0F62FE`).
- Ikon aplikasi: squircle biru dengan bentuk folder putih berisi siluet foto + badge tanggal `MM-DD`.
- Varian ikon yang disiapkan: **240px** (utama), **128px** (taskbar), **32px** (tray), **monokrom** (tema gelap).
- Palet utama: `#0F62FE` (primary), `#0066CC` (primary dark), `#EFF6FF` (tint), `#0F172A` (ink), `#64748B` (muted), `#E2E8F0` (border).
- Font: **Inter**.

---

## 12. Requirement Non-Fungsional

| Kategori | Requirement |
|---|---|
| Platform | Windows 10/11 x64, WebView2 (sudah tersedia default) |
| Distribusi | Exe portable; installer MSI opsional |
| Ukuran aplikasi | < 20 MB |
| RAM idle | < 100 MB |
| Performa scan | 10.000 file ≤ 10 detik (drive lokal) |
| Performa transfer | ≥ 80% throughput disk |
| Responsivitas | UI tidak freeze; operasi berat di background thread |
| Keandalan | Copy atomik via `.part`; tidak ada file setengah jadi dengan nama final |
| Crash recovery | File `.part` dibersihkan saat start; log sesi terakhir dipertahankan |
| Path | Mendukung long path (> 260 karakter) |
| Keamanan | Tidak ada telemetri/network selain akses filesystem & UNC |
| Kredensial NAS | Memakai Windows Credential Manager (bukan menyimpan password plaintext) |
| Aksesibilitas | Kontras memadai, label jelas, navigasi keyboard |
| Lokalisasi | Bahasa Indonesia |

---

## 13. Arsitektur Teknis

### Stack
- **Shell/UI:** Tauri 2 (frontend web ringan: Svelte atau React).
- **Core:** Rust (async `tokio`), dijalankan di background.
- **Packaging:** build Windows x64, exe portable.

### Modul Rust
| Modul | Tanggung jawab |
|---|---|
| `scanner` | Traversal rekursif, filter ekstensi, deteksi file |
| `metadata` | EXIF foto, `mvhd` video, validasi & fallback |
| `planner` | Grouping tanggal, sanitasi nama, deteksi konflik, estimasi ruang |
| `transfer` | Copy chunked, `.part` → rename, worker pool, event progres |
| `verifier` | Size+mtime, checksum BLAKE3 opsional |
| `logger` | Log terstruktur, ekspor CSV/TXT |
| `settings` | Baca/tulis konfigurasi JSON |
| `paths` | Long-path helper, validasi root tujuan, free space |

### Crates utama
| Kebutuhan | Crate |
|---|---|
| Metadata foto | `kamadak-exif` |
| Container video | `mp4parse` (fallback parser manual bila perlu) |
| Traversal file | `walkdir` |
| Async runtime | `tokio` |
| Hash opsional | `blake3` |
| Tanggal/waktu | `chrono` |
| Serialisasi | `serde`, `serde_json` |
| CSV | `csv` |
| Drive/free space | `sysinfo` |
| Dialog native | `tauri-plugin-dialog` |

### Batas proses
```text
Frontend (UI)  ──invoke(command)──▶  Rust core
      ▲                                  │
      └────── event(progress/log) ───────┘
```

### Kontrak perintah & event
**Commands:** `scan_sources`, `plan_transfer`, `start_transfer`, `cancel_transfer`, `export_log`, `get_settings`, `save_settings`.

**Events:** `scan://progress`, `scan://complete`, `transfer://progress`, `transfer://file`, `transfer://error`, `transfer://complete`.

### Model konkurensi
- Scan: paralel terbatas (rayon).
- Transfer: `tokio` + semaphore (default 4 worker).
- Progress di-throttle 100 ms.
- Cancellation memakai `CancellationToken`.

---

## 14. Model Data & Konfigurasi

### `settings.json`
```json
{
  "version": 2,
  "destinations": [
    {
      "id": "d1",
      "root": "D:\\Dokumentasi",
      "activity": "Pelatihan Petani",
      "suffix": "Hari 2",
      "enabled": true
    },
    {
      "id": "d2",
      "root": "\\\\nas\\arsip\\dokumentasi",
      "activity": "Pelatihan Petani",
      "suffix": "Desa Sukamaju",
      "enabled": true
    }
  ],
  "rootHistory": [
    "D:\\Dokumentasi",
    "\\\\nas\\arsip\\dokumentasi"
  ],
  "activityHistory": ["Rapat Koordinasi", "Pelatihan Petani"],
  "suffixHistory": ["Hari 1", "Desa Sukamaju"],
  "transferMode": "copy",
  "verification": "size_mtime",
  "parallelWorkers": 4,
  "removeEmptySourceDirs": true,
  "confirmBeforeStart": true,
  "language": "id"
}
```

| Field | Tipe | Keterangan |
|---|---|---|
| `destinations` | array | Daftar tujuan aktif; setiap entri = satu baris panel Tujuan |
| `destinations[].root` | string | Path root tujuan (lokal/UNC) |
| `destinations[].activity` | string | Nama kegiatan (belum disanitasi) |
| `destinations[].suffix` | string \| null | Suffix opsional |
| `destinations[].enabled` | bool | Baris ikut dieksekusi atau tidak |
| `rootHistory` | array | Riwayat root untuk autocomplete |
| `removeEmptySourceDirs` | bool | Hapus subfolder kosong setelah Move |
| `confirmBeforeStart` | bool | Selalu tampilkan modal konfirmasi (default `true`, tidak bisa dimatikan untuk mode Move) |

### Skema log (CSV)
| Field | Tipe | Keterangan |
|---|---|---|
| `timestamp` | ISO-8601 | Waktu kejadian |
| `mode` | enum | `copy` / `move` |
| `source_path` | string | Path asal lengkap |
| `dest_root` | string | Root tujuan tempat file ini ditulis |
| `dest_path` | string | Path tujuan lengkap |
| `size_bytes` | integer | Ukuran file |
| `status` | enum | `copied`, `verified`, `deleted`, `skipped`, `duplicate`, `error` |
| `metadata_source` | enum | `exif`, `video`, `file`, `manual` |
| `duration_ms` | integer | Durasi per file |
| `error` | string | Pesan error bila ada |

---

## 15. Penanganan Error & Edge Case

| Skenario | Perilaku |
|---|---|
| SD card dicabut saat transfer | Hentikan transfer sumber tsb, tandai file aktif `error`, sumber lain tetap lanjut; tampilkan instruksi sambungkan ulang |
| Ruang tujuan habis | Pre-check sebelum mulai; jika habis di tengah, hentikan dengan pesan jelas |
| NAS offline/timeout | Retry terbatas (mis. 3×), lalu gagal dengan pesan; tidak menghapus sumber |
| Tujuan FAT32 & file > 4 GB | Deteksi sebelum mulai, tolak file tersebut dengan pesan spesifik |
| Nama file terlalu panjang / invalid | Sanitasi + truncate, catat di log |
| File 0 byte / korup | Lewati (skipped), catat di log |
| Jam kamera salah (1970/2038) | Anggap tidak valid, fallback + badge |
| Dua file berbeda dengan nama sama | Rename otomatis `(1)`, `(2)` |
| Folder tujuan sudah ada | Merge tanpa menghapus |
| Akses ditolak (permission) | Tandai `error`, lanjut file lain |
| File dikunci antivirus | Retry 2× dengan jeda, lalu `error` |
| Path > 260 karakter | Gunakan long-path prefix |
| Nama kegiatan kosong/invalid | Blok tombol Mulai + pesan validasi per baris tujuan |
| Root tujuan tidak ada | Tawarkan buat folder atau pilih ulang |
| Satu tujuan gagal (mode Move) | File sumber TIDAK dihapus; tujuan lain yang berhasil tetap valid |
| Kombinasi tujuan duplikat (root+keg+sfx sama) | Blok validasi + pesan tujuan ganda |
| Ruang tidak cukup di salah satu tujuan | Pre-check per tujuan; informasikan tujuan mana yang kekurangan |

---

## 16. Kriteria Penerimaan (Acceptance Criteria)

**AC-01 — Grouping multi-tanggal**
> Diberikan sumber berisi file tanggal 15 dan 17 Maret, ketika kegiatan "Rapat" diisi dan transfer dijalankan, maka terbentuk dua folder: `03-15_Rapat` dan `03-17_Rapat`.

**AC-02 — Zero-pad**
> Diberikan file tanggal 2 Januari, maka folder bernama `01-02_...` (bukan `1-2_...`).

**AC-03 — Tanggal dari metadata**
> Diberikan file JPEG dengan EXIF `DateTimeOriginal = 2026:03:15`, maka tanggal efektif adalah 15 Maret 2026 meskipun file disalin hari ini.

**AC-04 — Fallback ber-flag**
> Diberikan video tanpa field `mvhd` valid, maka tanggal diambil dari file time, baris ditandai "Perlu Review", dan dapat diedit.

**AC-05 — Copy + verifikasi**
> Setelah copy, ukuran dan timestamp tujuan sama dengan sumber; status file `verified`.

**AC-06 — Move aman**
> Pada mode Move, file sumber hanya terhapus setelah status `verified`; jika verifikasi gagal, file sumber tetap ada.

**AC-07 — Duplikat**
> Diberikan file tujuan dengan nama sama dan isi identik, maka file dilewati sebagai `duplicate`, dan tidak ada duplikasi baru.

**AC-08 — Konflik nama**
> Diberikan file tujuan nama sama dengan isi berbeda, maka file baru disimpan sebagai `nama (1).ext` dan dicatat `conflict` di log.

**AC-09 — Sanitasi**
> Nama kegiatan `Laporan: Q1?` menjadi `Laporan- Q1` (karakter terlarang hilang) dan lolos validasi.

**AC-10 — Batal**
> Menekan Batal menghentikan transfer, menghapus file `.part`, dan tidak menghapus sumber yang belum terverifikasi.

**AC-11 — Ekspor log**
> Setelah sesi, tombol ekspor menghasilkan CSV berisi semua file terpilih beserta statusnya.

**AC-12 — Multi-sumber**
> Menjatuhkan dua sumber sekaligus memproses keduanya berurutan dalam satu sesi dan satu log gabungan.

**AC-13 — Multi-tujuan simultan**
> Diberikan 2 baris tujuan aktif (`D:\Dokumentasi` + kegiatan "Pelatihan Petani" + suffix "Hari 2", dan `E:\Arsip Lapangan` + kegiatan "Pelatihan Petani" + suffix "Desa Sukamaju"), ketika transfer 6 file tanggal 15 Maret berhasil, maka file tersalin ke `D:\Dokumentasi\03-15_Pelatihan Petani_Hari 2` dan `E:\Arsip Lapangan\03-15_Pelatihan Petani_Desa Sukamaju`, dan log memuat entri per pasangan file × tujuan.

**AC-14 — Modal konfirmasi sebelum mulai**
> Menekan tombol Mulai selalu menampilkan modal konfirmasi; transfer tidak berjalan sebelum tombol "Mulai Transfer" ditekan. Pada mode Move, tombol "Mulai Transfer" tetap disabled sampai checkbox "Saya memahami risiko" dicentang.

**AC-15 — Move multi-tujuan aman**
> Pada mode Move dengan 2 tujuan, jika tujuan A berhasil diverifikasi dan tujuan B gagal (mis. NAS timeout), maka file sumber **tidak dihapus** dan status file dicatat `partial`/`error` di log.

**AC-16 — Tooltip tanggal manual**
> Mengklik kolom tanggal baris fallback membuka tooltip editor tepat di atas sel; memilih tanggal baru lalu "Terapkan" mengubah tanggal efektif dan nama folder tujuan file tersebut secara langsung, serta label sumber tanggal menjadi `Manual`.

---

## 17. Rencana Pengujian (Ringkas)

| Jenis | Fokus |
|---|---|
| Unit test (Rust) | Sanitasi nama, parsing tanggal, grouping, logika konflik |
| Integration test | Copy/move pada folder temp, verifikasi, simulasi file rusak |
| Test metadata | Set sampel JPEG/TIFF/PNG/WebP/MP4/MOV (termasuk tanpa metadata) |
| Test UI | State kosong → ready → running → done; validasi form |
| Test beban | 10.000 file kecil + beberapa file besar (multi-GB) |
| Test kegagalan | Cabut sumber (simulasi), tujuan read-only, ruang penuh |
| UAT | Operator dokumentasi menjalankan 1 sesi nyata |

---

## 18. Risiko & Mitigasi

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Timezone metadata tidak konsisten | Folder tanggal bergeser | Edit manual per baris + preview sebelum eksekusi |
| SD card dicabut di tengah proses | Transfer gagal sebagian | Copy `.part`, verifikasi, sumber tidak dihapus |
| Parsing HEIC/CR3 belum ada di MVP | Tanggal fallback | Badge review + rencana fase 2 |
| Nama bentrok massal | File tertimpa | Rename otomatis + log, tidak pernah overwrite |
| Folder tujuan FAT32 | File > 4 GB gagal | Deteksi awal + pesan spesifik |
| Antivirus memperlambat | Throughput turun | Retry + info kecepatan real-time |
| Path panjang | Error I/O | Long-path prefix + sanitasi panjang nama |
| Pengguna salah pilih Move | Data hilang | Konfirmasi eksplisit + hapus hanya setelah verifikasi |

---

## 19. Roadmap

| Fase | Isi | Keluaran |
|---|---|---|
| **Fase 0** ✅ | Desain UI lengkap di FigCraft (7 state utama + 4 tab Pengaturan + modal konfirmasi + tooltip tanggal + logo/ikon) | `design/ArsipDok.figcraft` |
| **Fase 0.5** ✅ | Setup project Tauri 2 + Svelte 5, struktur modul Rust (scanner, metadata, planner, transfer, verifier, logger, settings, paths, error, commands), 19 unit test lulus | Kerangka aplikasi berjalan |
| **Fase 1 (MVP)** | Scan, metadata, grouping, preview, multi-tujuan, copy/move, verifikasi, log | Exe fungsional v1.0 |
| **Fase 2** | EXIF HEIC/CR3, opsi pertahankan subfolder, auto-eject, preset kegiatan | v1.1 |
| **Fase 3** | Deteksi duplikat lintas arsip, profil tujuan tersimpan, persist antrean | v1.2 |

---

## 20. Pertanyaan Terbuka

| # | Pertanyaan | Default sementara |
|---|---|---|
| Q-1 | Struktur subfolder sumber dipertahankan atau diratakan? | **Diratakan** (semua file langsung di folder tanggal) |
| Q-2 | Suffix berlaku global per sesi atau bisa per grup tanggal? | **Per baris tujuan** (mengikuti desain multi-tujuan) |
| Q-3 | Perlu auto-eject SD card setelah selesai? | Fase 2 |
| Q-4 | Perlu preset daftar kegiatan? | Fase 2 (opsional) |
| Q-5 | Apakah folder sumber kosong setelah Move dihapus? | Ya (subfolder kosong) |
| Q-6 | Installer MSI diperlukan atau cukup exe portable? | Exe portable dulu |
| Q-7 | Batas jumlah baris tujuan per sesi? | **Tanpa batas** (rekomendasi ≤ 10) |
| Q-8 | Mode Move multi-tujuan: eksekusi berurutan per tujuan atau paralel? | **Berurutan** (aman untuk SD card sumber) |

---

## 21. Lampiran

### A. Ekstensi yang didukung

**Foto:** `.jpg`, `.jpeg`, `.png`, `.webp`, `.tif`, `.tiff`, `.heic`, `.heif`, `.dng`, `.cr2`, `.cr3`, `.nef`, `.arw`, `.rw2`, `.raf`, `.orf`, `.pef`, `.srw`

**Video:** `.mp4`, `.mov`, `.m4v`, `.mkv`, `.avi`, `.mts`, `.m2ts`, `.mpg`, `.mpeg`, `.wmv`, `.3gp`, `.braw`, `.r3d`

> Catatan: semua ekstensi di atas ikut ditransfer; dukungan pembacaan metadata tanggal berbeda per format (lihat §10).

### B. Glosarium

| Istilah | Arti |
|---|---|
| EXIF | Metadata tertanam pada file foto (termasuk waktu pengambilan) |
| `mvhd` | Box header video MP4/MOV yang menyimpan creation time |
| Fallback | Sumber tanggal cadangan saat metadata tidak tersedia |
| `.part` | File sementara selama proses copy |
| Verifikasi | Pengecekan kecocokan file sumber dan tujuan |
| Root tujuan | Folder induk tempat semua folder tanggal dibuat |
| Multi-tujuan | Kemampuan mengirim satu sesi transfer ke lebih dari satu root tujuan sekaligus |
| Suffix | Tambahan nama opsional setelah nama kegiatan |

### C. Referensi
- EXIF 2.32 — tag `DateTimeOriginal` (0x9003)
- ISO/IEC 14496-12 (ISO Base Media File Format) — `mvhd.creation_time`
- Microsoft — Naming Files, Paths, and Namespaces
- Tauri 2 documentation

---

**Akhir dokumen.** Perubahan requirement wajib melalui revisi versi dokumen ini.
