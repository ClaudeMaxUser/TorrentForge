//! Torrent engine core
//!
//! Core data structures and utilities for torrent management.

use crate::bencode::{Decoder, Encoder, Value};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;

/// Torrent metadata parsed from a .torrent file
#[derive(Debug, Clone)]
pub struct TorrentMetadata {
    /// Announce URL (primary tracker)
    pub announce: String,
    /// List of announce URLs (tier list)
    pub announce_list: Option<Vec<Vec<String>>>,
    /// Info hash (SHA-1 of info dictionary)
    pub info_hash: [u8; 20],
    /// Name of torrent
    pub name: String,
    /// Total size in bytes
    pub size: u64,
    /// Piece length in bytes
    pub piece_length: u64,
    /// SHA-1 hashes of each piece
    pub pieces: Vec<[u8; 20]>,
    /// Files in torrent
    pub files: Vec<FileInfo>,
    /// Creation date (Unix timestamp)
    pub creation_date: Option<i64>,
    /// Encoding (if specified)
    pub encoding: Option<String>,
    /// Comment
    pub comment: Option<String>,
    /// Created by
    pub created_by: Option<String>,
}

/// Information about a single file in a torrent
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Path components
    pub path: Vec<String>,
    /// Size in bytes
    pub length: u64,
}

/// Error parsing torrent
#[derive(Debug)]
pub enum TorrentParseError {
    BencodeParse(String),
    MissingField(String),
    InvalidFormat(String),
    InvalidHash(String),
}

impl std::fmt::Display for TorrentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentParseError::BencodeParse(msg) => write!(f, "Bencode parse error: {}", msg),
            TorrentParseError::MissingField(field) => write!(f, "Missing field: {}", field),
            TorrentParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            TorrentParseError::InvalidHash(msg) => write!(f, "Invalid hash: {}", msg),
        }
    }
}

impl std::error::Error for TorrentParseError {}

impl TorrentMetadata {
    /// Parse torrent from bencode bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, TorrentParseError> {
        let value = Decoder::decode(data)
            .map_err(|e| TorrentParseError::BencodeParse(e.to_string()))?;

        let dict = value.as_dict().ok_or(TorrentParseError::InvalidFormat(
            "Torrent must be a dictionary".to_string(),
        ))?;

        // Extract announce
        let announce = dict
            .get(&b"announce"[..])
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .ok_or(TorrentParseError::MissingField("announce".to_string()))?;

        // Extract announce-list (optional)
        let announce_list = dict.get(&b"announce-list"[..]).and_then(|v| {
            v.as_list().map(|list| {
                list.iter()
                    .filter_map(|tier| {
                        tier.as_list().map(|urls| {
                            urls.iter()
                                .filter_map(|url| url.as_string().map(|s| s.to_string()))
                                .collect()
                        })
                    })
                    .collect()
            })
        });

        // Extract info dictionary
        let info_value = dict
            .get(&b"info"[..])
            .ok_or(TorrentParseError::MissingField("info".to_string()))?;

        let info_dict = info_value.as_dict().ok_or(TorrentParseError::InvalidFormat(
            "info must be a dictionary".to_string(),
        ))?;

        // Calculate info hash
        let info_hash = Self::calculate_info_hash(info_value)?;

        // Extract name
        let name = info_dict
            .get(&b"name"[..])
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .ok_or(TorrentParseError::MissingField("info.name".to_string()))?;

        // Extract piece length
        let piece_length = info_dict
            .get(&b"piece length"[..])
            .and_then(|v| v.as_integer())
            .ok_or(TorrentParseError::MissingField("info.piece length".to_string()))?
            as u64;

        // Extract pieces
        let pieces_data = info_dict
            .get(&b"pieces"[..])
            .and_then(|v| v.as_bytes())
            .ok_or(TorrentParseError::MissingField("info.pieces".to_string()))?;

        if pieces_data.len() % 20 != 0 {
            return Err(TorrentParseError::InvalidHash(
                "pieces data must be multiple of 20 bytes".to_string(),
            ));
        }

        let mut pieces = Vec::new();
        for chunk in pieces_data.chunks(20) {
            let mut hash = [0u8; 20];
            hash.copy_from_slice(chunk);
            pieces.push(hash);
        }

        // Extract files
        let (files, size) = Self::parse_files(info_dict)?;

        // Extract optional fields
        let creation_date = dict.get(&b"creation date"[..]).and_then(|v| v.as_integer());
        let encoding = dict
            .get(&b"encoding"[..])
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        let comment = dict
            .get(&b"comment"[..])
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        let created_by = dict
            .get(&b"created by"[..])
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());

        Ok(TorrentMetadata {
            announce,
            announce_list,
            info_hash,
            name,
            size,
            piece_length,
            pieces,
            files,
            creation_date,
            encoding,
            comment,
            created_by,
        })
    }

    /// Calculate info hash (SHA-1 of encoded info dictionary)
    fn calculate_info_hash(info_value: &Value) -> Result<[u8; 20], TorrentParseError> {
        let encoded = Encoder::encode(info_value);
        let mut hasher = Sha1::new();
        hasher.update(&encoded);
        let result = hasher.finalize();

        let mut hash = [0u8; 20];
        hash.copy_from_slice(&result[..]);
        Ok(hash)
    }

    /// Parse files from info dictionary
    fn parse_files(
        info_dict: &BTreeMap<Vec<u8>, Value>,
    ) -> Result<(Vec<FileInfo>, u64), TorrentParseError> {
        let mut total_size = 0;
        let mut files = Vec::new();

        if let Some(files_value) = info_dict.get(&b"files"[..]) {
            // Multi-file torrent
            let files_list = files_value.as_list().ok_or(TorrentParseError::InvalidFormat(
                "files must be a list".to_string(),
            ))?;

            for file_dict in files_list {
                let file = file_dict.as_dict().ok_or(TorrentParseError::InvalidFormat(
                    "file entry must be a dictionary".to_string(),
                ))?;

                let path_list = file
                    .get(&b"path"[..])
                    .and_then(|v| v.as_list())
                    .ok_or(TorrentParseError::MissingField("file.path".to_string()))?;

                let path: Result<Vec<String>, _> = path_list
                    .iter()
                    .map(|v| {
                        v.as_string()
                            .map(|s| s.to_string())
                            .ok_or(TorrentParseError::InvalidFormat(
                                "path component must be string".to_string(),
                            ))
                    })
                    .collect();

                let length = file
                    .get(&b"length"[..])
                    .and_then(|v| v.as_integer())
                    .ok_or(TorrentParseError::MissingField("file.length".to_string()))?
                    as u64;

                total_size += length;
                files.push(FileInfo {
                    path: path?,
                    length,
                });
            }
        } else {
            // Single-file torrent
            let length = info_dict
                .get(&b"length"[..])
                .and_then(|v| v.as_integer())
                .ok_or(TorrentParseError::MissingField("info.length".to_string()))?
                as u64;

            total_size = length;
            files.push(FileInfo {
                path: vec![
                    info_dict
                        .get(&b"name"[..])
                        .and_then(|v| v.as_string())
                        .map(|s| s.to_string())
                        .ok_or(TorrentParseError::MissingField("info.name".to_string()))?
                ],
                length,
            });
        }

        Ok((files, total_size))
    }

    /// Get info hash as hex string
    pub fn info_hash_hex(&self) -> String {
        self.info_hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    /// Get number of pieces
    pub fn num_pieces(&self) -> usize {
        self.pieces.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_torrent() -> Vec<u8> {
        // Create a minimal valid torrent for testing
        // Single-file torrent with 2 pieces
        let mut dict = BTreeMap::new();
        dict.insert(b"announce".to_vec(), Value::Bytes(b"http://tracker.example.com/announce".to_vec()));

        let mut info = BTreeMap::new();
        info.insert(b"name".to_vec(), Value::Bytes(b"test.txt".to_vec()));
        info.insert(b"length".to_vec(), Value::Integer(40));
        info.insert(b"piece length".to_vec(), Value::Integer(20));
        
        // Two SHA-1 hashes (40 bytes total)
        let mut pieces = vec![0u8; 40];
        pieces[0] = 0x01;
        pieces[20] = 0x02;
        info.insert(b"pieces".to_vec(), Value::Bytes(pieces));

        dict.insert(b"info".to_vec(), Value::Dict(info));

        Encoder::encode(&Value::Dict(dict))
    }

    #[test]
    fn test_parse_single_file_torrent() {
        let data = create_test_torrent();
        let metadata = TorrentMetadata::from_bytes(&data).unwrap();

        assert_eq!(metadata.name, "test.txt");
        assert_eq!(metadata.size, 40);
        assert_eq!(metadata.piece_length, 20);
        assert_eq!(metadata.num_pieces(), 2);
        assert!(!metadata.info_hash_hex().is_empty());
    }

    #[test]
    fn test_info_hash_consistency() {
        let data = create_test_torrent();
        let metadata1 = TorrentMetadata::from_bytes(&data).unwrap();
        let metadata2 = TorrentMetadata::from_bytes(&data).unwrap();

        assert_eq!(metadata1.info_hash, metadata2.info_hash);
    }
}

