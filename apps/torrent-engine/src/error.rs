//! Error types for the torrent engine

use thiserror::Error;

/// Result type for torrent engine operations
pub type Result<T> = std::result::Result<T, Error>;

/// Torrent engine errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("Bencode error: {0}")]
    BencodeError(String),

    #[error("Torrent metadata error: {0}")]
    MetadataError(String),

    #[error("Tracker error: {0}")]
    TrackerError(String),

    #[error("Peer error: {0}")]
    PeerError(String),

    #[error("Piece error: {0}")]
    PieceError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
