//! Shared types for IPC communication
//!
//! Types and structures shared between the Rust engine and Electron/UI.

use serde::{Deserialize, Serialize};

/// Torrent status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TorrentStatus {
    Downloading,
    Paused,
    Seeding,
    Completed,
    Error,
}

/// Torrent information sent over IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub id: String,
    pub name: String,
    pub status: TorrentStatus,
    pub progress: f64,
    pub downloaded: u64,
    pub uploaded: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
}
