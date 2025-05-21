pub use thiserror::Error;
#[derive(Error, Debug)]
pub enum VsixHarvesterError {
    #[error("{0}")]
    InvalidArchitecture(String),

    #[error("Invalid extension identifier: {0}")]
    InvalidExtensionId(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to query marketplace API: {0}")]
    ApiError(String),

    #[error("Failed to download extension: {0}")]
    DownloadError(String),
}
pub type Result<T> = std::result::Result<T, VsixHarvesterError>;
