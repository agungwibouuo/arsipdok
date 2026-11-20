# 🔧 Troubleshooting ArsipDok

> Panduan mengatasi masalah umum, error yang sering muncul, dan tips debugging.

---

## Daftar Isi

1. [Masalah Instalasi](#masalah-instalasi)
2. [Masalah Scan & Sumber](#masalah-scan--sumber)
3. [Masalah Transfer](#masalah-transfer)
4. [Masalah Verifikasi](#masalah-verifikasi)
5. [Masalah UI & Tampilan](#masalah-ui--tampilan)
6. [Error Log Reference](#error-log-reference)
7. [Debug Mode & Log](#debug-mode--log)
8. [FAQ](#faq)

---

## Masalah Instalasi

### ❌ Installer tidak mau berjalan / "Windows protected your PC"

**Gejala:** SmartScreen memblokir installer.

**Solusi:**
1. Klik **More info** pada dialog SmartScreen.
2. Klik **Run anyway**.

> Ini terjadi karena sertifikat code signing belum terpasang pada v0.1.0. Installer aman — verifikasi SHA-256 bila ragu (lihat [USER_GUIDE](USER_GUIDE.md#verifikasi-installer-opsional)).

---

### ❌ Error: "WebView2 Runtime is not installed"

**Gejala:** Aplikasi tidak terbuka setelah instalasi.

**Solusi:**
1. Unduh [WebView2 Runtime Evergreen](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) dari Microsoft.
2. Install runtime, lalu jalankan ulang ArsipDok.

> Windows 10 (build terbaru) & Windows 11 sudah menyertakan WebView2 secara default.

---

### ❌ Aplikasi crash saat startup

**Solusi:**
1. Hapus cache aplikasi: `%APPDATA%\ArsipDok\` (settings akan reset).
2. Jalankan ulang aplikasi.
3. Bila masih crash, [buat issue](https://github.com/agungwibouuo/arsipdok/issues) dengan lampiran log (lihat [Debug Mode](#debug-mode--log)).

---

## Masalah Scan & Sumber

### ❌ Folder sumber tidak terbaca / hasil scan 0 file

**Kemungkinan penyebab:**

| Penyebab | Solusi |
|---|---|
| Ekstensi tidak didukung | Cek daftar [ekstensi yang didukung](USER_GUIDE.md#hasil-scan) |
| Folder hanya berisi subfolder tersembunyi | Nonaktifkan hidden attribute di Explorer |
| Permission denied (folder sistem) | Jalankan ArsipDok sebagai Administrator |
| SD card belum ter-mount | Cek di Explorer apakah drive muncul |
| Path melebihi 260 karakter | ArsipDok sudah handle `\\?\` prefix — cek versi terbaru |

---

### ❌ Scan sangat lambat (> 1 menit untuk ribuan file)

**Penyebab umum:**
- Sumber adalah **network drive (NAS)** dengan latency tinggi.
- Disk HDD eksternal yang sedang sibuk.
- Folder berisi file sangat banyak (>50.000) dalam subfolder dalam.

**Solusi:**
- Untuk NAS: pertimbangkan copy dulu ke lokal, baru scan.
- Pastikan tidak ada antivirus yang scanning real-time folder yang sama.
- Batasi folder sumber ke subfolder spesifik, bukan root drive.

---

### ❌ File foto terdeteksi sebagai video (atau sebaliknya)

**Penyebab:** Klasifikasi berbasis ekstensi, bukan isi file.

**Solusi:** Rename ekstensi file ke yang benar (`IMG_0001.jpeg` → `IMG_0001.jpg` jika file memang JPG).

---

## Masalah Transfer

### ❌ Error: "No such file or directory" saat transfer

**Penyebab:** File sumber dipindah/dihapus setelah scan tapi sebelum transfer.

**Solusi:**
1. Jangan memodifikasi folder sumber antara scan dan transfer.
2. Klik **Re-scan** untuk memperbarui daftar file.
3. Jalankan ulang transfer.

---

### ❌ Error: "Disk full" / "There is not enough space"

**Solusi:**
1. Cek indikator ruang disk di panel Destinasi.
2. Hapus file tidak perlu di drive tujuan.
3. Atau ganti ke drive tujuan dengan ruang lebih besar.
4. Jalankan ulang transfer — file yang sudah ter-transfer akan terdeteksi sebagai `Duplicate` dan di-skip.

---

### ❌ Transfer terhenti di tengah, aplikasi crash

**Solusi:**
1. Buka kembali aplikasi.
2. Jalankan transfer ulang dengan konfigurasi yang sama.
3. File yang sudah berhasil ditransfer **tidak akan diduplicate** — sistem deteksi otomatis skip.
4. Untuk mode **Move**: file sumber yang belum terhapus masih ada (sistem hanya menghapus setelah verifikasi sukses).

---

### ❌ File duplikat muncul dengan suffix `_1`, `_2`, dst.

**Penjelasan:** Ini fitur, bukan bug. Bila file dengan nama sama sudah ada di tujuan dan **isinya berbeda**, ArsipDok tidak menimpa — file baru diberi suffix numerik.

**Bila file yang sudah ada itu identik:** status menjadi `Duplicate` dan dihitung sukses (tidak dicopy ulang).

---

### ❌ Mode Move: sumber tidak terhapus

**Penyebab & Solusi:**

| Penyebab | Perilaku | Solusi |
|---|---|---|
| Satu tujuan gagal verifikasi | Sumber sengaja **tidak dihapus** (safety) | Cek error di log, perbaiki, transfer ulang |
| File sumber sedang digunakan aplikasi lain | Windows lock | Tutup aplikasi lain, transfer ulang |
| Permission read-only pada sumber | Hapus gagal | Hapus manual via Explorer |

> Ini by design: data safety > kebersihan folder. Sumber hanya dihapus setelah **semua** tujuan sukses.

---

### ❌ Transfer sangat lambat

**Faktor yang mempengaruhi:**
- Kecepatan baca sumber (SD card class berbeda sangat jauh).
- Verifikasi BLAKE3 menggandakan waktu I/O.
- Drive tujuan HDD vs SSD.

**Tips:**
- Gunakan verifikasi `Size + Mtime` (default) bila tujuan lokal SSD.
- Hindari transfer ke USB 2.0 port (gunakan USB 3.0/USB-C).
- Pertimbangkan `parallel_workers` di Settings — untuk HDD tujuan, turunkan ke 2 untuk mengurangi seek thrashing.

---

## Masalah Verifikasi

### ❌ Error: "Verification failed: ukuran beda"

**Penyebab:** Ukuran file tujuan tidak sama dengan sumber setelah copy.

**Kemungkinan:**
- Transfer terputus (disk penuh, kabel lepas).
- Antivirus mengubah file saat scanning.
- Drive tujuan bermasalah (bad sector).

**Solusi:**
1. Cek kesehatan drive tujuan.
2. Jalankan transfer ulang — file rusak akan di-overwrite.
3. Tambahkan folder tujuan ke exclusion list antivirus.

---

### ❌ Error: "Hash BLAKE3 berbeda"

**Penyebab:** Isi file tujuan berbeda di bit-level dari sumber, meski ukurannya sama.

**Kemungkinan:**
- Bit rot / bad sector.
- NAS dengan ECC error.
- Kabel USB/optical drive menghasilkan corruption.

**Solusi:**
1. Transfer ulang file yang sama.
2. Bila berulang, ganti kabel/port.
3. Jalankan `chkdsk` pada drive terkait.

---

## Masalah UI & Tampilan

### ❌ Tema gelap/terang tidak berubah

**Solusi:**
1. Klik toggle tema di title bar → pilih **System**.
2. Restart aplikasi.
3. Bila tetap, hapus cache: `%APPDATA%\ArsipDok\settings.json` (semua pengaturan reset).

---

### ❌ Layout rusak / komponen tumpang tindih

**Penyebab:** Window di-resize terlalu kecil.

**Solusi:**
- Minimal lebar window adalah **620px** (dirancang untuk Windows Snap 2-side).
- Di bawah **860px**, layout otomatis stack vertical.
- Restore window ke ukuran default dengan double-click title bar.

---

### ❌ Preview table tidak menampilkan semua file

**Penyebab:** Filter aktif menyembunyikan sebagian baris.

**Solusi:** Klik tombol **Reset Filter** di atas tabel.

---

### ❌ Aplikasi terbuka dengan window kosong / putih

**Penyebab:** WebView2 crash atau GPU driver issue.

**Solusi:**
1. Tutup aplikasi dari Task Manager (cari `ArsipDok.exe`).
2. Jalankan ulang.
3. Bila berulang: update GPU driver atau jalankan dengan flag `--disable-gpu` (shortcut modification).

---

## Error Log Reference

Status yang muncul di log CSV dan artinya:

| Status | Arti | Perlu tindakan? |
|---|---|---|
| `done` | File berhasil ditransfer & diverifikasi | ✅ Tidak |
| `duplicate` | File identik sudah ada di tujuan (skip) | ✅ Tidak |
| `skipped` | File dilewati (user uncheck / cancel) | ✅ Tidak |
| `error` | Transfer gagal — lihat kolom `error` | ⚠️ Ya |
| `partial` | Copy terputus di tengah | ⚠️ Ya — transfer ulang |
| `source_deleted` | File sumber berhasil dihapus (mode Move) | ✅ Tidak |

---

## Debug Mode & Log

### Lokasi File Penting

| File/Folder | Lokasi | Isi |
|---|---|---|
| Settings | `%APPDATA%\ArsipDok\settings.json` | Konfigurasi & history |
| Log CSV | `%APPDATA%\ArsipDok\logs\YYYY-MM-DD_HHMMSS.csv` | Audit trail transfer |
| Log aplikasi | `%APPDATA%\ArsipDok\app.log` *(bila ada)* | Log internal debug |

### Menjalankan dengan Console Logging (Dev Mode)

```powershell
# Jalankan dari terminal untuk melihat log Rust
& "C:\Program Files\ArsipDok\ArsipDok.exe" 2>&1 | Tee-Object -FilePath console.log
```

### Mengumpulkan Info untuk Bug Report

Saat membuat issue, sertakan:

1. **Versi aplikasi** (di title bar / settings)
2. **Log CSV** dari sesi bermasalah
3. **Langkah reproduce** — langkah detail sampai error muncul
4. **Screenshot** error/pesannya
5. **Info sistem**:
   ```powershell
   systeminfo | findstr /B /C:"OS Name" /C:"OS Version"
   Get-PSDrive -PSProvider FileSystem
   ```

---

## FAQ

### Q: Apakah ArsipDok mengirim data ke internet?

**Tidak.** Aplikasi 100% offline. Tidak ada telemetri, analytics, atau koneksi keluar.

### Q: Apakah mode Move aman?

**Ya**, dengan safety mechanism: file sumber **hanya dihapus** setelah **semua tujuan** terverifikasi utuh. Bila satu tujuan gagal, sumber tetap ada.

### Q: Bisakah transfer ke beberapa drive sekaligus?

**Ya.** Tambahkan beberapa Destinasi di panel tujuan — file akan dicopy ke semua yang enabled.

### Q: Bagaimana jika saya menutup aplikasi saat transfer?

Transfer akan berhenti. File yang sudah selesai tetap ada di tujuan. Jalankan ulang dengan konfigurasi sama — file yang sudah ada akan di-skip sebagai duplicate.

### Q: Kenapa folder dinamai `03-15` bukan `2026-03-15`?

Format folder adalah `MM-DD_Nama Kegiatan`. Tahun diambil dari struktur parent folder yang Anda tentukan (misalnya root `D:\Arsip\2026\`). Ini menjaga nama folder pendek untuk path panjang Windows.

### Q: Apakah bisa mengubah format penamaan folder?

Di v0.1.0 format `MM-DD_Nama[_Suffix]` fixed. Custom format ada di roadmap.

### Q: Kenapa beberapa file ditandai "Perlu Review"?

File tersebut tidak punya EXIF/Video metadata valid, sehingga tanggalnya fallback ke filesystem time (creation/modified) yang mungkin tidak akurat (misalnya tanggal copy, bukan tanggal foto). Disarankan review dan override manual bila perlu.

---

<p align="center">Dokumen ini bagian dari <a href="../README.md">ArsipDok</a> · v0.1.0</p>
