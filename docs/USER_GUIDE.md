# 📘 Panduan Pengguna ArsipDok

> Sortir & pemindahan otomatis dokumentasi foto/video berbasis metadata.

Panduan ini menjelaskan cara instalasi, konfigurasi, dan penggunaan ArsipDok sehari-hari — dari memasang aplikasi sampai menyelesaikan transfer pertama Anda.

---

## Daftar Isi

1. [Instalasi](#1-instalasi)
2. [Persiapan](#2-persiapan)
3. [Alur Kerja Utama](#3-alur-kerja-utama)
4. [Menambah Folder Sumber](#4-menambah-folder-sumber)
5. [Mengatur Tujuan Arsip](#5-mengatur-tujuan-arsip)
6. [Preview & Seleksi File](#6-preview--seleksi-file)
7. [Override Tanggal Manual](#7-override-tanggal-manual)
8. [Menjalankan Transfer](#8-menjalankan-transfer)
9. [Verifikasi Integritas](#9-verifikasi-integritas)
10. [Log Audit](#10-log-audit)
11. [Pengaturan Lanjutan](#11-pengaturan-lanjutan)
12. [Tips & Trik](#12-tips--trik)

---

## 1. Instalasi

### Persyaratan Sistem

| Kebutuhan | Minimum |
|---|---|
| OS | Windows 10 (build 19041+) atau Windows 11, x64 |
| RAM | 4 GB |
| Ruang disk | 50 MB (instalasi) + ruang untuk arsip |
| Lainnya | WebView2 (bawaan Windows 10/11 terbaru) |

### Langkah Instalasi

1. Unduh installer dari [halaman Releases](https://github.com/agungwibouuo/arsipdok/releases):
   ```
   ArsipDok_0.1.0_x64-setup.exe
   ```
2. Klik dua kali file installer untuk memulai.
3. Ikuti wizard NSIS (Next → Install → Finish).
4. ArsipDok kini tersedia di Start Menu dan Desktop.

> ✅ **Catatan**: ArsipDok berjalan 100% offline — tidak ada data yang dikirim ke internet.

### Verifikasi Installer (Opsional)

Untuk memverifikasi keaslian file installer, bandingkan SHA-256:

```powershell
Get-FileHash .\ArsipDok_0.1.0_x64-setup.exe -Algorithm SHA256
```

Hash yang diharapkan untuk v0.1.0:
```
2BA095C527750BE4FB416078DA40B681C5258C5369F74AAED4D902BDDD7ED46C
```

---

## 2. Persiapan

Sebelum memulai, siapkan:

- **Sumber** — folder berisi foto/video yang akan diarsipkan:
  - SD card / memory card kamera
  - Flash disk / HDD eksternal
  - Folder lokal di komputer
  - NAS (mounted sebagai drive letter)
- **Tujuan** — folder arsip utama (misalnya `D:\Arsip\2026`)
- **Nama kegiatan** — nama acara/proyek untuk penamaan folder (misalnya `Rapat Koordinasi`)

### Format Folder Tujuan

ArsipDok membuat folder otomatis dengan format:

```text
MM-DD_Nama Kegiatan[_Suffix]
```

Contoh hasil:

```text
D:\Arsip\2026\
├── 03-15_Rapat Koordinasi\
│   ├── IMG_0001.jpg
│   ├── IMG_0002.jpg
│   └── VID_0001.mp4
├── 03-15_Rapat Koordinasi\Backup\    ← dengan suffix
└── 03-20_Kunjungan Lapangan\
```

**Bagaimana tanggal ditentukan?**

ArsipDok membaca metadata file dengan prioritas berikut:

| Prioritas | Sumber | Kapan digunakan |
|---|---|---|
| 1 | EXIF DateTimeOriginal | Foto JPG/PNG/TIFF/HEIC |
| 2 | QuickTime/MP4 creation_time | Video MP4/MOV/M4V |
| 3 | File creation time | Fallback bila metadata tidak ada |
| 4 | File modified time | Fallback terakhir |

> 📌 **Penting**: Tanggal folder **bukan** tanggal transfer, melainkan tanggal pengambilan gambar sesungguhnya.

---

## 3. Alur Kerja Utama

ArsipDok memiliki alur kerja 5 langkah:

```text
┌─────────┐    ┌──────────┐    ┌─────────┐    ┌──────────────┐    ┌─────────┐
│  Selamat │    │ Tambah   │    │ Atur    │    │ Preview &    │    │ Transfer│
│  datang  │───▶│ Sumber   │───▶│ Tujuan  │───▶│ Override     │───▶│ & Selesai│
└─────────┘    └──────────┘    └─────────┘    └──────────────┘    └─────────┘
```

1. **Tambah Sumber** — pilih folder yang berisi file
2. **Atur Tujuan** — tentukan folder arsip + nama kegiatan
3. **Preview** — lihat tabel file, sesuaikan bila perlu
4. **Transfer** — proses copy/move berjalan
5. **Selesai** — ringkasan hasil + log tersimpan

---

## 4. Menambah Folder Sumber

### Cara 1: Drag & Drop

Tarik folder dari Windows Explorer ke area **Source Queue** di aplikasi.

### Cara 2: Dialog Pemilihan

1. Klik tombol **+ Tambah Sumber**.
2. Pilih folder sumber di dialog.
3. Klik **Select Folder**.

### Hasil Scan

Setelah folder ditambahkan, ArsipDok melakukan scan otomatis:

- ✅ Membaca file rekursif (termasuk subfolder)
- ✅ Menghitung total file + ukuran
- ✅ Memisahkan foto vs video
- ✅ Menampilkan ringkasan

File yang didukung:

| Tipe | Ekstensi |
|---|---|
| Foto | `.jpg` `.jpeg` `.png` `.heic` `.heif` `.webp` `.tif` `.tiff` `.cr2` `.cr3` `.nef` `.arw` `.dng` |
| Video | `.mp4` `.mov` `.m4v` `.avi` `.mkv` `.mts` `.m2ts` `.3gp` `.wmv` `.flv` `.webm` `.mpg` `.mpeg` |

File lain (misalnya `.txt`, `.pdf`, `.thm`) **dilewati** secara otomatis.

### Menghapus Sumber

Klik ikon **×** di samping folder untuk menghapusnya dari antrian. Ini tidak menghapus file asli — hanya mengeluarkannya dari daftar pemrosesan.

---

## 5. Mengatur Tujuan Arsip

### Menambah Tujuan

1. Di panel **Destinasi**, klik **+ Tambah Tujuan**.
2. Isi field berikut:

| Field | Wajib | Contoh | Keterangan |
|---|---|---|---|
| **Root Folder** | ✅ | `D:\Arsip\2026` | Folder utama arsip |
| **Nama Kegiatan** | ✅ | `Rapat Koordinasi` | Menjadi bagian nama folder |
| **Suffix** | ⬜ | `Original` | Tambahan opsional di akhir nama |
| **Enabled** | ✅ | ✓ | Tujuan aktif / nonaktif |

3. Klik **Simpan**.

### Hasil Nama Folder

Contoh konfigurasi:

```text
Root: D:\Arsip\2026
Kegiatan: Rapat Koordinasi
Suffix: Original
```

Menghasilkan folder:

```text
D:\Arsip\2026\03-15_Rapat Koordinasi_Original\
```

### Multi-Tujuan

Anda bisa menambahkan beberapa tujuan sekaligus. Semua file akan disalin ke **semua tujuan yang enabled**:

```text
Tujuan 1: D:\Arsip\2026       (kegiatan: Rapat Koordinasi)
Tujuan 2: E:\Backup\2026      (kegiatan: Rapat Koordinasi, suffix: Backup)
```

Hasil:

```text
D:\Arsip\2026\03-15_Rapat Koordinasi\
E:\Backup\2026\03-15_Rapat Koordinasi_Backup\
```

### Sanitasi Nama

Nama kegiatan/suffix otomatis disanitasi agar valid di Windows:

- Karakter ilegal (`< > : " / \ | ? *`) diganti spasi
- Nama reserved Windows (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`) dihindari
- Spasi berlebih dirapikan
- Panjang maksimum segmen: 150 karakter

### Cek Ruang Disk

ArsipDok menampilkan indikator ruang disk untuk setiap tujuan. Bila ruang tidak cukup, akan muncul peringatan sebelum transfer dimulai.

---

## 6. Preview & Seleksi File

Sebelum transfer, tinjau file di tabel **Preview**:

| Kolom | Keterangan |
|---|---|
| ☑ | Checkbox untuk include/exclude file |
| Nama File | Nama file sumber |
| Tipe | Foto / Video |
| Ukuran | Dalam format human-readable (KB/MB/GB) |
| Tanggal | Hasil ekstraksi metadata |
| Sumber Tanggal | EXIF / Video / Creation / Modified / Manual |
| Status | Perlu review / OK |

### Filter

Gunakan filter di atas tabel untuk menyaring berdasarkan:

- Sumber tanggal (misalnya hanya yang "Perlu Review")
- Status
- Pencarian nama file

### Indikator "Perlu Review"

File dengan tanggal yang **kurang meyakinkan** (fallback ke creation/modified time, atau tanggal tidak wajar) ditandai **Perlu Review** dengan warna kuning/amber. Disarankan untuk:

1. Klik sel tanggal untuk override manual (lihat [bagian 7](#7-override-tanggal-manual))
2. Atau biarkan — file tetap diproses dengan tanggal fallback

### Seleksi Massal

- Klik header checkbox untuk select/deselect semua
- Shift+Klik untuk rentang
- Ctrl+Klik untuk individual

File yang **tidak dicentang** tidak akan ditransfer.

---

## 7. Override Tanggal Manual

Bila metadata tidak akurat (misalnya baterai kamera habis, reset tanggal), Anda bisa mengganti tanggal manual:

1. Di tabel Preview, klik **sel tanggal** pada file yang ingin diubah.
2. Date picker muncul — pilih tanggal yang benar.
3. Sumber tanggal berubah menjadi **Manual** (warna biru).
4. Folder tujuan otomatis diperbarui sesuai tanggal baru.

### Override Massal

1. Seleksi beberapa baris (Ctrl/Shift+Klik).
2. Klik kanan → **Override Tanggal**.
3. Pilih tanggal — semua baris terpilih diperbarui sekaligus.

> 💡 Override manual **tidak mengubah** file asli — hanya mempengaruhi penempatan folder di transfer ini.

---

## 8. Menjalankan Transfer

### Mode Transfer

| Mode | Perilaku | Kapan digunakan |
|---|---|---|
| **Copy** | Salin file ke tujuan, sumber tetap ada | Default, aman untuk pertama kali |
| **Move** | Salin → verifikasi → hapus sumber | Mengosongkan SD card setelah arsip aman |

> ⚠️ **Mode Move menghapus file sumber** — tapi hanya setelah verifikasi integritas sukses. Bila verifikasi gagal, sumber tetap ada.

### Konfirmasi Sebelum Mulai

Bila `confirm_before_start` aktif (default: on), dialog konfirmasi muncul berisi:

- Total file yang akan ditransfer
- Total ukuran
- Daftar tujuan
- Mode (Copy/Move)

Klik **Mulai Transfer** untuk melanjutkan, atau **Batal**.

### Monitoring Progres

Saat transfer berjalan, panel **Progress** menampilkan:

- Progress bar per tujuan
- File yang sedang diproses
- Statistik real-time: selesai / error / skip / total
- Total bytes yang telah ditransfer

### Membatalkan Transfer

Klik tombol **Batalkan** kapan saja:

- Transfer berhenti setelah file saat ini selesai
- File yang sudah ter-transfer **tetap ada** di tujuan
- Log tetap tersimpan dengan status `cancelled`

### Hasil Transfer (TransferReport)

Setelah selesai, ringkasan ditampilkan:

```text
✅ Transfer Selesai
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Berhasil    : 245 file (2.3 GB)
Duplikat    : 3 file (skip)
Error       : 0 file
Durasi      : 1 mnt 23 dtk
Log         : %APPDATA%\ArsipDok\logs\2026-03-15_143022.csv
```

### Penanganan Duplikat

Bila file dengan nama sama sudah ada di tujuan, ArsipDok **tidak menimpa**. File baru diberi suffix:

```text
IMG_0001.jpg        ← file lama
IMG_0001_1.jpg      ← duplikat baru, auto-rename
IMG_0001_2.jpg      ← duplikat berikutnya
```

Status di log: `Duplicate` — dihitung sukses, bukan error.

---

## 9. Verifikasi Integritas

Setelah setiap file disalin, ArsipDok memverifikasi hasilnya:

| Mode | Metode | Kecepatan | Akurasi |
|---|---|---|---|
| **Size + Mtime** | Bandingkan ukuran file | ⚡ Sangat cepat | Deteksi copy tidak lengkap |
| **BLAKE3** | Hash kriptografis penuh | 🐢 Lebih lambat | Deteksi korupsi bit-level |

Default: **Size + Mtime** (cukup untuk mayoritas kasus).

Aktifkan **BLAKE3** di Settings bila:

- Transfer via jaringan/NAS (rawan bit flip)
- Arsip jangka panjang (butuh jaminan integritas penuh)
- Disk tujuan pernah bermasalah sebelumnya

> Dalam mode **Move**, file sumber baru dihapus **setelah** verifikasi sukses. Bila verifikasi gagal, sumber dipertahankan dan item ditandai `Error`.

---

## 10. Log Audit

Setiap sesi transfer menghasilkan file CSV di:

```text
%APPDATA%\ArsipDok\logs\YYYY-MM-DD_HHMMSS.csv
```

### Kolom Log

| Kolom | Keterangan |
|---|---|
| timestamp | Waktu transfer |
| mode | copy / move |
| source_path | Path file sumber |
| dest_root | Root tujuan |
| dest_path | Path final file tujuan |
| size_bytes | Ukuran file |
| status | done / duplicate / skipped / error / partial |
| metadata_source | exif / video / file_creation / file_modified / manual |
| duration_ms | Durasi transfer file |
| error | Pesan error (bila ada) |

### Ekspor Log

1. Setelah transfer selesai, klik **Ekspor Log**.
2. Pilih lokasi penyimpanan.
3. File CSV tersalin ke lokasi tersebut.

### Membuka Folder Log

Klik **Buka Folder Log** di layar hasil untuk membuka `%APPDATA%\ArsipDok\logs\` di Explorer.

---

## 11. Pengaturan Lanjutan

Buka **Settings** (ikon gerigi di title bar) untuk opsi berikut:

| Pengaturan | Default | Keterangan |
|---|---|---|
| Mode Transfer | Copy | Default mode untuk sesi baru |
| Verifikasi | Size + Mtime | Metode verifikasi pasca-transfer |
| Paralel Workers | 4 | Jumlah thread transfer paralel |
| Hapus Folder Sumber Kosong | On | Mode Move: bersihkan folder kosong setelah transfer |
| Konfirmasi Sebelum Mulai | On | Tampilkan dialog konfirmasi |
| Bahasa | id | Bahasa UI (id / en) |

### Lokasi File Settings

Settings tersimpan di:

```text
%APPDATA%\ArsipDok\settings.json
```

Berisi: daftar tujuan tersimpan, riwayat root/kegiatan/suffix, preferensi mode, dan lainnya.

### Reset Settings

Hapus file `settings.json` lalu restart aplikasi — semua pengaturan kembali ke default.

---

## 12. Tips & Trik

### ✅ Do

- **Gunakan mode Copy dulu** untuk transfer pertama ke tujuan baru — verifikasi hasilnya, baru gunakan Move
- **Cek file "Perlu Review"** — file dengan tanggal fallback mungkin salah penempatan
- **Backup settings.json** — berisi riwayat kegiatan & tujuan yang sering dipakai
- **Ekspor log** untuk dokumentasi resmi kegiatan

### ❌ Jangan

- **Jangan cabut SD card** saat transfer berjalan — tunggu sampai selesai
- **Jangan edit file di tujuan** saat transfer berjalan di folder yang sama
- **Jangan jalankan dua instance** yang menulis ke tujuan sama bersamaan

### Performa

| Skenario | Estimasi |
|---|---|
| Scan 10.000 file | < 10 detik |
| Copy 2 GB (SSD → SSD) | ~30 detik |
| Copy 2 GB + BLAKE3 | ~60 detik |
| Copy 2 GB (SD card → HDD) | 2-5 menit (tergantung SD card) |

### Keyboard Shortcuts

| Shortcut | Aksi |
|---|---|
| `Ctrl+O` | Tambah sumber |
| `Ctrl+S` | Simpan settings |
| `Ctrl+Enter` | Mulai transfer |
| `Esc` | Batalkan transfer |
| `F5` | Refresh scan |

---

## Butuh Bantuan?

- 🐛 **Menemukan bug?** → [Buat issue di GitHub](https://github.com/agungwibouuo/arsipdok/issues)
- 🔧 **Masalah teknis?** → Baca [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- 📖 **Ingin kontribusi?** → Baca [DEVELOPER.md](DEVELOPER.md)

---

<p align="center">Dokumen ini bagian dari <a href="../README.md">ArsipDok</a> · v0.1.0</p>
