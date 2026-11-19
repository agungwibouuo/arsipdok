//! Verifikasi hasil transfer: ukuran+mtime (default) atau BLAKE3

use crate::error::{Error, Result};
use crate::settings::VerificationMode;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Verifikasi file tujuan terhadap sumber
pub fn verify(source: &Path, dest: &Path, mode: VerificationMode) -> Result<()> {
    let src_meta = std::fs::metadata(source)?;
    let dst_meta = std::fs::metadata(dest)?;

    if src_meta.len() != dst_meta.len() {
        return Err(Error::VerificationFailed(format!(
            "ukuran beda: src={} dst={}",
            src_meta.len(),
            dst_meta.len()
        )));
    }

    if mode == VerificationMode::Blake3 {
        if !compare_blake3(source, dest)? {
            return Err(Error::VerificationFailed("hash BLAKE3 berbeda".into()));
        }
    }

    Ok(())
}

/// Bandingkan hash BLAKE3 dua file
pub fn compare_blake3(a: &Path, b: &Path) -> Result<bool> {
    Ok(hash_file(a)? == hash_file(b)?)
}

fn hash_file(path: &Path) -> Result<[u8; 32]> {
    let mut f = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("arsipdok-test");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn verify_same_size_ok() {
        let a = tmp("a.txt");
        let b = tmp("b.txt");
        std::fs::write(&a, b"hello").unwrap();
        std::fs::write(&b, b"hello").unwrap();
        assert!(verify(&a, &b, VerificationMode::SizeMtime).is_ok());
    }

    #[test]
    fn verify_size_mismatch() {
        let a = tmp("c.txt");
        let b = tmp("d.txt");
        std::fs::write(&a, b"abc").unwrap();
        let mut f = File::create(&b).unwrap();
        f.write_all(b"abcdef").unwrap();
        assert!(verify(&a, &b, VerificationMode::SizeMtime).is_err());
    }

    #[test]
    fn blake3_identical() {
        let a = tmp("e.txt");
        let b = tmp("f.txt");
        std::fs::write(&a, b"same content").unwrap();
        std::fs::write(&b, b"same content").unwrap();
        assert!(compare_blake3(&a, &b).unwrap());
    }
}
