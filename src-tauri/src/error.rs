use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Format tanggal tidak valid: {0}")]
    InvalidDate(String),

    #[error("Metadata tidak ditemukan")]
    NoMetadata,

    #[error("Root tujuan tidak dapat diakses atau ditulis: {0}")]
    DestinationUnwritable(String),

    #[error("Ruang kosong tidak mencukupi di {destination}: butuh {required_bytes} byte, sisa {available_bytes} byte")]
    InsufficientSpace {
        destination: String,
        required_bytes: u64,
        available_bytes: u64,
    },

    #[error("Transfer dibatalkan oleh pengguna")]
    Cancelled,

    #[error("Verifikasi gagal: {0}")]
    VerificationFailed(String),

    #[error("File sedang dikunci atau digunakan proses lain")]
    FileLocked,

    #[error("Format path tidak didukung: {0}")]
    InvalidPath(String),

    #[error("Lainnya: {0}")]
    Other(String),
}

impl Error {
    /// Kode error singkat untuk frontend
    pub fn code(&self) -> &'static str {
        match self {
            Error::Io(_) => "io",
            Error::Json(_) => "json",
            Error::Csv(_) => "csv",
            Error::InvalidDate(_) => "invalid_date",
            Error::NoMetadata => "no_metadata",
            Error::DestinationUnwritable(_) => "destination_unwritable",
            Error::InsufficientSpace { .. } => "insufficient_space",
            Error::Cancelled => "cancelled",
            Error::VerificationFailed(_) => "verification_failed",
            Error::FileLocked => "file_locked",
            Error::InvalidPath(_) => "invalid_path",
            Error::Other(_) => "other",
        }
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Error", 3)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
