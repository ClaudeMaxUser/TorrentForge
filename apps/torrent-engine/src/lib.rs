//! Torrent Engine
//!
//! Custom BitTorrent protocol implementation with peer coordination,
//! piece scheduling, DHT support, and fault tolerance.

pub mod bencode;
pub mod config;
pub mod error;
pub mod logging;
pub mod torrent;
pub mod tracker;

pub use error::{Error, Result};
pub use torrent::{TorrentMetadata, TorrentParseError};
pub use tracker::{HttpTracker, PeerInfo, TrackerResponse};

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
