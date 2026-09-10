//! Integration test framework
//!
//! Shared test utilities for integration testing the torrent engine.

pub struct TestTorrent {
    pub name: String,
    pub size: u64,
}

pub struct TestPeer {
    pub addr: String,
}

impl TestTorrent {
    pub fn new(name: &str, size: u64) -> Self {
        Self {
            name: name.to_string(),
            size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torrent_creation() {
        let torrent = TestTorrent::new("test.iso", 1024 * 1024 * 1024);
        assert_eq!(torrent.name, "test.iso");
        assert_eq!(torrent.size, 1024 * 1024 * 1024);
    }
}
