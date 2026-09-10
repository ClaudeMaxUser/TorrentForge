//! Domain events
//!
//! Event types emitted by the torrent engine and sent to the UI.

use serde::{Deserialize, Serialize};

/// Engine events sent to the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EngineEvent {
    // Torrent events
    #[serde(rename = "torrent.added")]
    TorrentAdded { id: String, name: String },

    #[serde(rename = "torrent.started")]
    TorrentStarted { id: String },

    #[serde(rename = "torrent.paused")]
    TorrentPaused { id: String },

    #[serde(rename = "torrent.completed")]
    TorrentCompleted { id: String },

    #[serde(rename = "torrent.removed")]
    TorrentRemoved { id: String },

    #[serde(rename = "torrent.progress")]
    TorrentProgress {
        id: String,
        progress: f64,
        downloaded: u64,
        uploaded: u64,
    },

    // Peer events
    #[serde(rename = "peer.connected")]
    PeerConnected {
        torrent_id: String,
        peer_addr: String,
        client: Option<String>,
    },

    #[serde(rename = "peer.disconnected")]
    PeerDisconnected {
        torrent_id: String,
        peer_addr: String,
    },

    // Piece events
    #[serde(rename = "piece.verified")]
    PieceVerified { torrent_id: String, index: u32 },

    #[serde(rename = "piece.failed")]
    PieceFailed { torrent_id: String, index: u32 },

    // Tracker events
    #[serde(rename = "tracker.announce")]
    TrackerAnnounce { torrent_id: String, url: String },

    #[serde(rename = "tracker.error")]
    TrackerError {
        torrent_id: String,
        url: String,
        error: String,
    },

    // Engine events
    #[serde(rename = "engine.started")]
    EngineStarted,

    #[serde(rename = "engine.stopped")]
    EngineStopped,

    #[serde(rename = "engine.error")]
    EngineError { error: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_event_serialization() {
        let event = EngineEvent::TorrentAdded {
            id: "123".to_string(),
            name: "test.torrent".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let restored: EngineEvent = serde_json::from_str(&json).unwrap();
        match restored {
            EngineEvent::TorrentAdded { id, name } => {
                assert_eq!(id, "123");
                assert_eq!(name, "test.torrent");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
