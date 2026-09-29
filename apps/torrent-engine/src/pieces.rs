//! Piece management and storage
//!
//! Handles piece state tracking, block assembly, SHA-1 verification,
//! and disk I/O operations.

use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

const BLOCK_SIZE: usize = 16 * 1024; // 16 KB standard block size

/// State of a piece
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceState {
    /// Not requested yet
    Missing,
    /// Currently downloading blocks
    Downloading,
    /// All blocks downloaded, waiting for verification
    Downloaded,
    /// Verified and complete
    Verified,
    /// Verification failed
    Failed,
}

/// A single piece being downloaded
#[derive(Debug, Clone)]
pub struct Piece {
    /// Index of the piece
    pub index: u32,
    /// Length of this piece
    pub length: u64,
    /// Expected SHA-1 hash
    pub hash: [u8; 20],
    /// Current state
    pub state: PieceState,
    /// Downloaded blocks (key: begin offset)
    pub blocks: HashMap<u32, Vec<u8>>,
    /// Total bytes downloaded
    pub downloaded: u64,
}

impl Piece {
    /// Create a new piece
    pub fn new(index: u32, length: u64, hash: [u8; 20]) -> Self {
        Piece {
            index,
            length,
            hash,
            state: PieceState::Missing,
            blocks: HashMap::new(),
            downloaded: 0,
        }
    }

    /// Add a downloaded block
    pub fn add_block(&mut self, offset: u32, data: &[u8]) -> Result<(), PieceError> {
        if offset as u64 + data.len() as u64 > self.length {
            return Err(PieceError::BlockOutOfBounds);
        }

        self.blocks.insert(offset, data.to_vec());
        self.downloaded += data.len() as u64;

        // Check if piece is complete
        if self.is_complete() {
            self.state = PieceState::Downloaded;
        }

        Ok(())
    }

    /// Check if all blocks are downloaded
    pub fn is_complete(&self) -> bool {
        if self.blocks.is_empty() {
            return false;
        }

        let mut covered = 0u64;
        let mut offsets: Vec<_> = self.blocks.keys().copied().collect();
        offsets.sort();

        for offset in offsets {
            let block_size = self.blocks[&offset].len() as u64;
            if offset as u64 != covered {
                return false; // Gap in coverage
            }
            covered += block_size;
        }

        covered == self.length
    }

    /// Assemble and verify the piece
    pub fn verify(&mut self) -> Result<Vec<u8>, PieceError> {
        if self.state != PieceState::Downloaded && self.state != PieceState::Downloading {
            return Err(PieceError::InvalidState(format!(
                "Cannot verify piece in state: {:?}",
                self.state
            )));
        }

        // Assemble blocks into complete piece
        let mut data = Vec::with_capacity(self.length as usize);
        let mut offsets: Vec<_> = self.blocks.keys().copied().collect();
        offsets.sort();

        for offset in offsets {
            data.extend_from_slice(&self.blocks[&offset]);
        }

        if data.len() as u64 != self.length {
            self.state = PieceState::Failed;
            return Err(PieceError::IncompleteData);
        }

        // Calculate hash
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let calculated_hash: [u8; 20] = hasher.finalize().as_slice().try_into().unwrap();

        if calculated_hash != self.hash {
            self.state = PieceState::Failed;
            return Err(PieceError::HashMismatch);
        }

        self.state = PieceState::Verified;
        Ok(data)
    }
}

/// Piece manager error
#[derive(Debug)]
pub enum PieceError {
    BlockOutOfBounds,
    InvalidState(String),
    IncompleteData,
    HashMismatch,
    IoError(String),
}

impl std::fmt::Display for PieceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceError::BlockOutOfBounds => write!(f, "Block is out of bounds"),
            PieceError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            PieceError::IncompleteData => write!(f, "Piece data is incomplete"),
            PieceError::HashMismatch => write!(f, "Piece hash does not match"),
            PieceError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for PieceError {}

/// Storage manager for writing pieces to disk
pub struct StorageManager {
    /// Download directory
    download_dir: std::path::PathBuf,
    /// File handles (piece_index -> file_handle)
    #[allow(dead_code)]
    files: HashMap<u32, tokio::fs::File>,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new<P: AsRef<Path>>(download_dir: P) -> Self {
        StorageManager {
            download_dir: download_dir.as_ref().to_path_buf(),
            files: HashMap::new(),
        }
    }

    /// Write a piece to disk
    pub async fn write_piece(
        &mut self,
        file_path: &std::path::Path,
        offset: u64,
        data: &[u8],
    ) -> Result<(), PieceError> {
        // Sanitize path to prevent directory traversal attacks
        if file_path.components().any(|c| c.as_os_str() == "..") {
            return Err(PieceError::IoError(
                "Path traversal detected".to_string(),
            ));
        }

        let full_path = self.download_dir.join(file_path);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| PieceError::IoError(e.to_string()))?;
        }

        // Open file for writing (create if doesn't exist)
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&full_path)
            .await
            .map_err(|e| PieceError::IoError(e.to_string()))?;

        // Seek to offset
        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|e| PieceError::IoError(e.to_string()))?;

        // Write data
        file.write_all(data)
            .await
            .map_err(|e| PieceError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Read piece data from disk
    pub async fn read_piece(
        &mut self,
        file_path: &std::path::Path,
        offset: u64,
        length: u64,
    ) -> Result<Vec<u8>, PieceError> {
        let full_path = self.download_dir.join(file_path);

        let mut file = OpenOptions::new()
            .read(true)
            .open(&full_path)
            .await
            .map_err(|e| PieceError::IoError(e.to_string()))?;

        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|e| PieceError::IoError(e.to_string()))?;

        let mut buf = vec![0u8; length as usize];
        file.read_exact(&mut buf)
            .await
            .map_err(|e| PieceError::IoError(e.to_string()))?;

        Ok(buf)
    }
}

/// Piece manager for coordinating piece downloads
pub struct PieceManager {
    /// All pieces in the torrent
    pieces: Vec<Piece>,
    /// Pieces by state
    #[allow(dead_code)]
    state_index: HashMap<PieceState, Vec<u32>>,
}

impl PieceManager {
    /// Create a new piece manager
    pub fn new(pieces_data: Vec<[u8; 20]>, piece_length: u64, total_size: u64) -> Self {
        let mut pieces = Vec::new();
        let mut state_index = HashMap::new();

        for (i, hash) in pieces_data.iter().enumerate() {
            let remaining = total_size.saturating_sub(i as u64 * piece_length);
            let piece_len = remaining.min(piece_length);

            pieces.push(Piece::new(i as u32, piece_len, *hash));
        }

        state_index.insert(PieceState::Missing, (0..pieces.len() as u32).collect());

        PieceManager {
            pieces,
            state_index,
        }
    }

    /// Get a piece by index
    pub fn get_piece(&mut self, index: u32) -> Option<&mut Piece> {
        if index as usize >= self.pieces.len() {
            return None;
        }
        Some(&mut self.pieces[index as usize])
    }

    /// Get the first missing piece that peer has
    pub fn get_next_piece_to_download(&self, peer_bitfield: &[bool]) -> Option<u32> {
        for piece in &self.pieces {
            if piece.state == PieceState::Missing
                && (piece.index as usize) < peer_bitfield.len()
                && peer_bitfield[piece.index as usize]
            {
                return Some(piece.index);
            }
        }
        None
    }

    /// Count completed pieces
    pub fn verified_count(&self) -> u32 {
        self.pieces
            .iter()
            .filter(|p| p.state == PieceState::Verified)
            .count() as u32
    }

    /// Total pieces
    pub fn total_count(&self) -> u32 {
        self.pieces.len() as u32
    }

    /// Progress percentage
    pub fn progress(&self) -> f32 {
        if self.pieces.is_empty() {
            return 0.0;
        }
        (self.verified_count() as f32 / self.total_count() as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_add_block() {
        let mut piece = Piece::new(0, 100, [0u8; 20]);
        let data = vec![1, 2, 3, 4, 5];
        piece.add_block(0, &data).unwrap();
        assert_eq!(piece.downloaded, 5);
    }

    #[test]
    fn test_piece_block_out_of_bounds() {
        let mut piece = Piece::new(0, 100, [0u8; 20]);
        let data = vec![0u8; 150];
        let result = piece.add_block(0, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_piece_is_complete() {
        let mut piece = Piece::new(0, 20, [0u8; 20]);
        assert!(!piece.is_complete());

        piece.add_block(0, &vec![1; 10]).unwrap();
        assert!(!piece.is_complete());

        piece.add_block(10, &vec![2; 10]).unwrap();
        assert!(piece.is_complete());
    }

    #[test]
    fn test_piece_verify() {
        let mut piece = Piece::new(0, 5, [0u8; 20]);
        let data = vec![1, 2, 3, 4, 5];

        // Calculate correct hash
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let correct_hash: [u8; 20] = hasher.finalize().as_slice().try_into().unwrap();

        piece.hash = correct_hash;
        piece.add_block(0, &data).unwrap();
        
        let verified = piece.verify();
        assert!(verified.is_ok());
        assert_eq!(piece.state, PieceState::Verified);
    }

    #[test]
    fn test_piece_manager_new() {
        let hashes = vec![[0u8; 20]; 10];
        let manager = PieceManager::new(hashes, 1000, 10000);
        assert_eq!(manager.total_count(), 10);
        assert_eq!(manager.verified_count(), 0);
    }
}
