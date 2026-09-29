//! Tracker communication
//!
//! HTTP tracker client for peer discovery and session management.

use crate::bencode::{Decoder, Value};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::time::timeout;

/// Tracker response containing peer list and metadata
#[derive(Debug, Clone)]
pub struct TrackerResponse {
    /// Interval in seconds for next announce
    pub interval: u64,
    /// Minimum interval (optional)
    pub min_interval: Option<u64>,
    /// Complete (seeders)
    pub complete: u32,
    /// Incomplete (leechers)
    pub incomplete: u32,
    /// List of discovered peers
    pub peers: Vec<PeerInfo>,
}

/// Peer information from tracker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerInfo {
    /// Peer IP address
    pub ip: IpAddr,
    /// Peer port
    pub port: u16,
}

impl PeerInfo {
    /// Get socket address
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

/// Tracker client error
#[derive(Debug)]
pub enum TrackerError {
    HttpError(String),
    InvalidResponse(String),
    DecodingError(String),
    Timeout,
}

impl std::fmt::Display for TrackerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackerError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            TrackerError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            TrackerError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            TrackerError::Timeout => write!(f, "Tracker request timed out"),
        }
    }
}

impl std::error::Error for TrackerError {}

/// HTTP Tracker client
pub struct HttpTracker {
    announce_url: String,
    client: reqwest::Client,
}

impl HttpTracker {
    /// Create a new HTTP tracker client
    pub fn new(announce_url: String) -> Self {
        HttpTracker {
            announce_url,
            client: reqwest::Client::new(),
        }
    }

    /// Send announce request to tracker
    pub async fn announce(
        &self,
        info_hash: &[u8; 20],
        peer_id: &[u8; 20],
        port: u16,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        compact: bool,
    ) -> Result<TrackerResponse, TrackerError> {
        // URL encode the info_hash and peer_id
        let info_hash_str = Self::url_encode_bytes(info_hash);
        let peer_id_str = Self::url_encode_bytes(peer_id);

        // Build query
        let query = format!(
            "?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
            info_hash_str,
            peer_id_str,
            port,
            uploaded,
            downloaded,
            left,
            if compact { 1 } else { 0 }
        );

        let url = format!("{}{}", self.announce_url, query);

        // Send request with timeout
        let response = timeout(Duration::from_secs(30), self.client.get(&url).send())
            .await
            .map_err(|_| TrackerError::Timeout)?
            .map_err(|e| TrackerError::HttpError(e.to_string()))?;

        let body = response
            .bytes()
            .await
            .map_err(|e| TrackerError::HttpError(e.to_string()))?;

        // Decode bencode response
        let value = Decoder::decode(&body)
            .map_err(|e| TrackerError::DecodingError(e.to_string()))?;

        Self::parse_tracker_response(&value)
    }

    /// Parse bencode tracker response
    fn parse_tracker_response(value: &Value) -> Result<TrackerResponse, TrackerError> {
        let dict = value.as_dict().ok_or(TrackerError::InvalidResponse(
            "Response must be a dictionary".to_string(),
        ))?;

        // Check for failure reason
        if let Some(failure_reason) = dict.get(&b"failure reason"[..]) {
            if let Some(msg) = failure_reason.as_string() {
                return Err(TrackerError::InvalidResponse(format!("Tracker error: {}", msg)));
            }
        }

        // Extract interval
        let interval = dict
            .get(&b"interval"[..])
            .and_then(|v| v.as_integer())
            .ok_or(TrackerError::InvalidResponse(
                "Missing interval field".to_string(),
            ))? as u64;

        // Extract optional fields
        let min_interval = dict
            .get(&b"min interval"[..])
            .and_then(|v| v.as_integer())
            .map(|v| v as u64);

        let complete = dict
            .get(&b"complete"[..])
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u32;

        let incomplete = dict
            .get(&b"incomplete"[..])
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u32;

        // Parse peers
        let peers = Self::parse_peers(dict)?;

        Ok(TrackerResponse {
            interval,
            min_interval,
            complete,
            incomplete,
            peers,
        })
    }

    /// Parse peer list from tracker response
    fn parse_peers(dict: &std::collections::BTreeMap<Vec<u8>, Value>) -> Result<Vec<PeerInfo>, TrackerError> {
        let mut peers = Vec::new();

        // Try compact format first (peers as binary string)
        if let Some(peers_value) = dict.get(&b"peers"[..]) {
            if let Some(peers_data) = peers_value.as_bytes() {
                // Compact format: 6 bytes per peer (4 for IP, 2 for port)
                if peers_data.len() % 6 == 0 {
                    for chunk in peers_data.chunks(6) {
                        let ip = std::net::Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                        peers.push(PeerInfo {
                            ip: IpAddr::V4(ip),
                            port,
                        });
                    }
                    return Ok(peers);
                }
            }
        }

        // Try dictionary format (list of dictionaries with ip, port, peer id)
        if let Some(peers_value) = dict.get(&b"peers"[..]) {
            if let Some(peers_list) = peers_value.as_list() {
                for peer_dict in peers_list {
                    if let Some(peer_map) = peer_dict.as_dict() {
                        if let (Some(ip_str), Some(port)) = (
                            peer_map
                                .get(&b"ip"[..])
                                .and_then(|v| v.as_string()),
                            peer_map
                                .get(&b"port"[..])
                                .and_then(|v| v.as_integer()),
                        ) {
                            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                                peers.push(PeerInfo {
                                    ip,
                                    port: port as u16,
                                });
                            }
                        }
                    }
                }
            }
        }

        if peers.is_empty() {
            return Err(TrackerError::InvalidResponse(
                "No peers found in response".to_string(),
            ));
        }

        Ok(peers)
    }

    /// URL encode bytes for tracker requests
    fn url_encode_bytes(data: &[u8]) -> String {
        data.iter()
            .map(|b| {
                match b {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                        format!("{}", *b as char)
                    }
                    _ => format!("%{:02X}", b),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_url_encode_bytes() {
        let data = b"hello";
        let encoded = HttpTracker::url_encode_bytes(data);
        assert_eq!(encoded, "hello");

        let data_with_special = vec![0x00, 0x01, 0x02, 0xFF];
        let encoded = HttpTracker::url_encode_bytes(&data_with_special);
        assert_eq!(encoded, "%00%01%02%FF");
    }

    #[test]
    fn test_parse_compact_peers() {
        // Create a tracker response with compact peers
        let mut dict = BTreeMap::new();
        dict.insert(b"interval".to_vec(), Value::Integer(1800));
        dict.insert(b"complete".to_vec(), Value::Integer(10));
        dict.insert(b"incomplete".to_vec(), Value::Integer(5));

        // Compact peers: 127.0.0.1:6881
        let peer_data = vec![127, 0, 0, 1, 0x1A, 0xE1];
        dict.insert(b"peers".to_vec(), Value::Bytes(peer_data));

        let response = HttpTracker::parse_tracker_response(&Value::Dict(dict)).unwrap();
        assert_eq!(response.interval, 1800);
        assert_eq!(response.complete, 10);
        assert_eq!(response.incomplete, 5);
        assert_eq!(response.peers.len(), 1);
        assert_eq!(response.peers[0].port, 6881);
    }
}
