use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CleanerError {
    #[error("I/O error at {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[allow(dead_code)]
    #[error("Unsupported or unrecognized image format for {path:?}")]
    UnsupportedFormat { path: PathBuf },

    #[error("Image error for {path:?}: {source}")]
    Image {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("Invalid arguments: {0}")]
    InvalidArgument(String),

    #[allow(dead_code)]
    #[error("General error: {0}")]
    Other(String),
}
