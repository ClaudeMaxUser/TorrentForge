//! Configuration management
//!
//! Handles application configuration including download directories,
//! peer limits, rate limiting, and other settings.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Download directory for torrents
    pub download_dir: PathBuf,

    /// Maximum number of concurrent peer connections
    pub max_peers: usize,

    /// Global download rate limit in bytes/sec (0 = unlimited)
    pub download_rate_limit: u64,

    /// Global upload rate limit in bytes/sec (0 = unlimited)
    pub upload_rate_limit: u64,

    /// Maximum number of simultaneous torrents
    pub max_torrents: usize,

    /// Listen port for incoming peer connections
    pub listen_port: u16,

    /// Enable DHT support
    pub enable_dht: bool,

    /// Session file location for resume support
    pub session_file: PathBuf,

    /// Log level
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            max_peers: 100,
            download_rate_limit: 0,
            upload_rate_limit: 0,
            max_torrents: 50,
            listen_port: 6881,
            enable_dht: true,
            session_file: dirs::cache_dir()
                .map(|p| p.join("torrent-forge").join("session.json"))
                .unwrap_or_else(|| PathBuf::from("session.json")),
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from file, or use defaults if not found
    pub fn load(path: &std::path::Path) -> crate::Result<Self> {
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            let config: Config = serde_json::from_str(&contents)
                .map_err(|e| crate::Error::MetadataError(e.to_string()))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &std::path::Path) -> crate::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| crate::Error::MetadataError(e.to_string()))?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.max_peers == 0 {
            return Err(crate::Error::InvalidState(
                "max_peers must be > 0".to_string(),
            ));
        }
        if self.listen_port == 0 {
            return Err(crate::Error::InvalidState(
                "listen_port must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_peers, 100);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.max_peers = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.max_peers, config.max_peers);
    }
}
