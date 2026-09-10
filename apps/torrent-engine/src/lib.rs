//! Torrent Engine
//!
//! Custom BitTorrent protocol implementation with peer coordination,
//! piece scheduling, DHT support, and fault tolerance.

pub mod error;
pub mod bencode;
pub mod torrent;

pub use error::{Error, Result};

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
