//! Peer connection and messaging
//!
//! Handles TCP connections to BitTorrent peers, handshake protocol,
//! and peer message serialization/deserialization.

use bytes::Bytes;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// BitTorrent peer connection
pub struct PeerConnection {
    stream: TcpStream,
    #[allow(dead_code)]
    addr: SocketAddr,
    /// Bitfield of pieces we have
    pub our_bitfield: Vec<bool>,
    /// Bitfield of pieces peer has
    pub peer_bitfield: Vec<bool>,
    /// Whether peer is choking us
    pub peer_choking: bool,
    /// Whether peer is interested in us
    pub peer_interested: bool,
    /// Whether we are choking peer
    pub our_choking: bool,
    /// Whether we are interested in peer
    pub our_interested: bool,
}

/// Peer message types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// keep-alive
    KeepAlive,
    /// 0: choke
    Choke,
    /// 1: unchoke
    Unchoke,
    /// 2: interested
    Interested,
    /// 3: not interested
    NotInterested,
    /// 4: have <piece index>
    Have(u32),
    /// 5: bitfield <bitfield>
    Bitfield(Bytes),
    /// 6: request <index> <begin> <length>
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    /// 7: piece <index> <begin> <block>
    Piece {
        index: u32,
        begin: u32,
        block: Bytes,
    },
    /// 8: cancel <index> <begin> <length>
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
}

impl Message {
    /// Serialize message to bytes
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Message::KeepAlive => vec![0, 0, 0, 0],
            Message::Choke => vec![0, 0, 0, 1, 0],
            Message::Unchoke => vec![0, 0, 0, 1, 1],
            Message::Interested => vec![0, 0, 0, 1, 2],
            Message::NotInterested => vec![0, 0, 0, 1, 3],
            Message::Have(index) => {
                let mut buf = vec![0, 0, 0, 5, 4];
                buf.extend_from_slice(&index.to_be_bytes());
                buf
            }
            Message::Bitfield(bf) => {
                let mut buf = vec![0];
                let len = (bf.len() as u32 + 1).to_be_bytes();
                buf.extend_from_slice(&len[1..]);
                buf.push(5);
                buf.extend_from_slice(bf);
                buf
            }
            Message::Request {
                index,
                begin,
                length,
            } => {
                let mut buf = vec![0, 0, 0, 13, 6];
                buf.extend_from_slice(&index.to_be_bytes());
                buf.extend_from_slice(&begin.to_be_bytes());
                buf.extend_from_slice(&length.to_be_bytes());
                buf
            }
            Message::Piece {
                index,
                begin,
                block,
            } => {
                let mut buf = vec![0];
                let len = (9 + block.len()) as u32;
                let len_bytes = len.to_be_bytes();
                buf.extend_from_slice(&len_bytes[1..]);
                buf.push(7);
                buf.extend_from_slice(&index.to_be_bytes());
                buf.extend_from_slice(&begin.to_be_bytes());
                buf.extend_from_slice(block);
                buf
            }
            Message::Cancel {
                index,
                begin,
                length,
            } => {
                let mut buf = vec![0, 0, 0, 13, 8];
                buf.extend_from_slice(&index.to_be_bytes());
                buf.extend_from_slice(&begin.to_be_bytes());
                buf.extend_from_slice(&length.to_be_bytes());
                buf
            }
        }
    }

    /// Deserialize message from bytes
    pub fn deserialize(data: &[u8]) -> Result<Option<Message>, MessageError> {
        if data.len() < 4 {
            return Ok(None);
        }

        let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if len == 0 {
            return Ok(Some(Message::KeepAlive));
        }

        if data.len() < len + 4 {
            return Ok(None);
        }

        let msg_type = data[4];
        let payload = &data[5..4 + len];

        let message = match msg_type {
            0 => Message::Choke,
            1 => Message::Unchoke,
            2 => Message::Interested,
            3 => Message::NotInterested,
            4 => {
                if payload.len() != 4 {
                    return Err(MessageError::InvalidFormat("have message must have 4 byte index".to_string()));
                }
                Message::Have(u32::from_be_bytes([
                    payload[0], payload[1], payload[2], payload[3],
                ]))
            }
            5 => Message::Bitfield(Bytes::copy_from_slice(payload)),
            6 => {
                if payload.len() != 12 {
                    return Err(MessageError::InvalidFormat(
                        "request message must have 12 bytes".to_string(),
                    ));
                }
                Message::Request {
                    index: u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]),
                    begin: u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]),
                    length: u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]),
                }
            }
            7 => {
                if payload.len() < 8 {
                    return Err(MessageError::InvalidFormat(
                        "piece message must have at least 8 bytes".to_string(),
                    ));
                }
                Message::Piece {
                    index: u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]),
                    begin: u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]),
                    block: Bytes::copy_from_slice(&payload[8..]),
                }
            }
            8 => {
                if payload.len() != 12 {
                    return Err(MessageError::InvalidFormat(
                        "cancel message must have 12 bytes".to_string(),
                    ));
                }
                Message::Cancel {
                    index: u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]),
                    begin: u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]),
                    length: u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]),
                }
            }
            _ => return Err(MessageError::UnknownType(msg_type)),
        };

        Ok(Some(message))
    }
}

/// Message deserialization error
#[derive(Debug)]
pub enum MessageError {
    InvalidFormat(String),
    UnknownType(u8),
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            MessageError::UnknownType(t) => write!(f, "Unknown message type: {}", t),
        }
    }
}

impl std::error::Error for MessageError {}

/// Peer connection error
#[derive(Debug)]
pub enum PeerError {
    ConnectionFailed(String),
    HandshakeFailed(String),
    ProtocolError(String),
    Timeout,
    IoError(String),
}

impl std::fmt::Display for PeerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            PeerError::HandshakeFailed(msg) => write!(f, "Handshake failed: {}", msg),
            PeerError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            PeerError::Timeout => write!(f, "Connection timed out"),
            PeerError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for PeerError {}

impl PeerConnection {
    /// Connect to a peer and perform handshake
    pub async fn connect(
        addr: SocketAddr,
        info_hash: &[u8; 20],
        peer_id: &[u8; 20],
        num_pieces: usize,
    ) -> Result<Self, PeerError> {
        // Connect with timeout
        let stream = timeout(Duration::from_secs(10), TcpStream::connect(addr))
            .await
            .map_err(|_| PeerError::Timeout)?
            .map_err(|e| PeerError::ConnectionFailed(e.to_string()))?;

        let mut conn = PeerConnection {
            stream,
            addr,
            our_bitfield: vec![false; num_pieces],
            peer_bitfield: vec![false; num_pieces],
            peer_choking: true,
            peer_interested: false,
            our_choking: true,
            our_interested: false,
        };

        // Perform handshake
        conn.handshake(info_hash, peer_id).await?;

        Ok(conn)
    }

    /// Perform BitTorrent handshake
    async fn handshake(&mut self, info_hash: &[u8; 20], peer_id: &[u8; 20]) -> Result<(), PeerError> {
        // Build handshake: 19 + "BitTorrent protocol" + 8 reserved bytes + info_hash + peer_id
        let mut handshake = Vec::new();
        handshake.push(19u8); // Protocol string length
        handshake.extend_from_slice(b"BitTorrent protocol");
        handshake.extend_from_slice(&[0u8; 8]); // Reserved bytes
        handshake.extend_from_slice(info_hash);
        handshake.extend_from_slice(peer_id);

        // Send handshake
        timeout(Duration::from_secs(10), self.stream.write_all(&handshake))
            .await
            .map_err(|_| PeerError::Timeout)?
            .map_err(|e| PeerError::HandshakeFailed(e.to_string()))?;

        // Receive handshake response
        let mut response = vec![0u8; 68];
        timeout(Duration::from_secs(10), self.stream.read_exact(&mut response))
            .await
            .map_err(|_| PeerError::Timeout)?
            .map_err(|e| PeerError::HandshakeFailed(e.to_string()))?;

        // Verify handshake response
        if response[0] != 19 {
            return Err(PeerError::HandshakeFailed(
                "Invalid protocol string length".to_string(),
            ));
        }

        if &response[1..20] != b"BitTorrent protocol" {
            return Err(PeerError::HandshakeFailed(
                "Invalid protocol string".to_string(),
            ));
        }

        let response_info_hash = &response[28..48];
        if response_info_hash != info_hash {
            return Err(PeerError::HandshakeFailed("Info hash mismatch".to_string()));
        }

        Ok(())
    }

    /// Send a message to the peer
    pub async fn send_message(&mut self, message: &Message) -> Result<(), PeerError> {
        let data = message.serialize();
        self.stream
            .write_all(&data)
            .await
            .map_err(|e| PeerError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Receive a message from the peer
    pub async fn receive_message(&mut self) -> Result<Option<Message>, PeerError> {
        let mut size_buf = [0u8; 4];
        
        match timeout(Duration::from_secs(30), self.stream.read_exact(&mut size_buf)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(PeerError::IoError(e.to_string())),
            Err(_) => return Err(PeerError::Timeout),
        }

        let len = u32::from_be_bytes(size_buf) as usize;

        if len == 0 {
            return Ok(Some(Message::KeepAlive));
        }

        // Read message (1 byte type + payload)
        let mut msg_buf = vec![0u8; len];
        match timeout(Duration::from_secs(30), self.stream.read_exact(&mut msg_buf)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(PeerError::IoError(e.to_string())),
            Err(_) => return Err(PeerError::Timeout),
        }

        let mut full_data = Vec::new();
        full_data.extend_from_slice(&size_buf);
        full_data.extend_from_slice(&msg_buf);

        Message::deserialize(&full_data)
            .map_err(|e| PeerError::ProtocolError(e.to_string()))
    }

    /// Update our bitfield
    pub fn set_piece(&mut self, index: usize) {
        if index < self.our_bitfield.len() {
            self.our_bitfield[index] = true;
        }
    }

    /// Check if peer has a piece
    pub fn peer_has_piece(&self, index: usize) -> bool {
        index < self.peer_bitfield.len() && self.peer_bitfield[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialize_keep_alive() {
        let msg = Message::KeepAlive;
        let data = msg.serialize();
        assert_eq!(data, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_message_serialize_choke() {
        let msg = Message::Choke;
        let data = msg.serialize();
        assert_eq!(data, vec![0, 0, 0, 1, 0]);
    }

    #[test]
    fn test_message_serialize_have() {
        let msg = Message::Have(42);
        let data = msg.serialize();
        assert_eq!(data[4], 4); // message type
        let index = u32::from_be_bytes([data[5], data[6], data[7], data[8]]);
        assert_eq!(index, 42);
    }

    #[test]
    fn test_message_round_trip() {
        let msg = Message::Request {
            index: 5,
            begin: 0,
            length: 16384,
        };
        let data = msg.serialize();
        let parsed = Message::deserialize(&data).unwrap();
        assert_eq!(parsed, Some(msg));
    }

    #[test]
    fn test_message_piece_round_trip() {
        let block = Bytes::copy_from_slice(b"test data");
        let msg = Message::Piece {
            index: 10,
            begin: 1024,
            block: block.clone(),
        };
        let data = msg.serialize();
        let parsed = Message::deserialize(&data).unwrap();
        
        if let Some(Message::Piece {
            index,
            begin,
            block: b,
        }) = parsed
        {
            assert_eq!(index, 10);
            assert_eq!(begin, 1024);
            assert_eq!(b, block);
        } else {
            panic!("Expected piece message");
        }
    }
}
