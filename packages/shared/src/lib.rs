//! Shared types for IPC communication
//!
//! Types and structures shared between the Rust engine and Electron/UI.

pub mod event;
pub mod ipc;

pub use event::EngineEvent;
pub use ipc::{EngineMethod, JsonRpcError, JsonRpcRequest, JsonRpcResponse};

/// Torrent status
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum TorrentStatus {
    Downloading,
    Paused,
    Seeding,
    Completed,
    Error,
}

/// Torrent information sent over IPC
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
