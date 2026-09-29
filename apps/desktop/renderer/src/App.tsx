import React from "react";
import "./App.css";

interface TorrentInfo {
  id: string;
  name: string;
  progress: number;
  downloaded: number;
  uploaded: number;
  download_speed: number;
  upload_speed: number;
  status: "Downloading" | "Paused" | "Seeding" | "Completed" | "Error";
  total_size?: number;
}

interface Window {
  electronAPI?: any;
}

const App: React.FC = () => {
  const [torrents, setTorrents] = React.useState<TorrentInfo[]>([]);
  const [selectedId, setSelectedId] = React.useState<string | null>(null);

  React.useEffect(() => {
    // Fetch torrents on mount
    if ((window as any).electronAPI) {
      (window as any).electronAPI.getTorrents().then(setTorrents);
    }

    // Listen for progress updates
    if ((window as any).electronAPI) {
      (window as any).electronAPI.onProgressUpdate((torrent: TorrentInfo) => {
        setTorrents((prev) =>
          prev.map((t) => (t.id === torrent.id ? torrent : t))
        );
      });
    }
  }, []);

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const formatSpeed = (bytes: number): string => {
    return formatBytes(bytes) + "/s";
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case "Downloading":
        return "#3b82f6"; // blue
      case "Seeding":
        return "#10b981"; // green
      case "Paused":
        return "#f59e0b"; // amber
      case "Completed":
        return "#6366f1"; // indigo
      case "Error":
        return "#ef4444"; // red
      default:
        return "#6b7280"; // gray
    }
  };

  const selectedTorrent = torrents.find((t) => t.id === selectedId);

  return (
    <div className="App">
      <header className="App-header">
        <h1>Torrent Forge</h1>
        <p>Production-grade BitTorrent Client with Custom Rust Engine</p>
      </header>

      <div className="container">
        <nav className="sidebar">
          <h3>Torrents</h3>
          <div className="torrent-list">
            {torrents.length === 0 ? (
              <p className="empty-state">No torrents yet</p>
            ) : (
              torrents.map((t) => (
                <div
                  key={t.id}
                  className={`torrent-item ${selectedId === t.id ? "active" : ""}`}
                  onClick={() => setSelectedId(t.id)}
                >
                  <div className="torrent-name">{t.name}</div>
                  <div className="torrent-progress-mini">
                    <div className="progress-bar">
                      <div
                        className="progress-fill"
                        style={{
                          width: `${t.progress}%`,
                          backgroundColor: getStatusColor(t.status),
                        }}
                      />
                    </div>
                    <span className="progress-percent">{t.progress.toFixed(1)}%</span>
                  </div>
                </div>
              ))
            )}
          </div>
        </nav>

        <main className="main-content">
          {selectedTorrent ? (
            <div className="torrent-details">
              <h2>{selectedTorrent.name}</h2>

              {/* Status and Speed */}
              <div className="status-bar">
                <div className="status-item">
                  <span className="label">Status:</span>
                  <span
                    className="value"
                    style={{ color: getStatusColor(selectedTorrent.status) }}
                  >
                    {selectedTorrent.status}
                  </span>
                </div>
                <div className="status-item">
                  <span className="label">↓ Download:</span>
                  <span className="value">
                    {formatSpeed(selectedTorrent.download_speed)}
                  </span>
                </div>
                <div className="status-item">
                  <span className="label">↑ Upload:</span>
                  <span className="value">
                    {formatSpeed(selectedTorrent.upload_speed)}
                  </span>
                </div>
              </div>

              {/* Progress Bar */}
              <div className="progress-section">
                <div className="progress-header">
                  <span>Progress: {selectedTorrent.progress.toFixed(1)}%</span>
                  <span className="progress-details">
                    {formatBytes(selectedTorrent.downloaded)} /{" "}
                    {formatBytes(selectedTorrent.total_size || selectedTorrent.downloaded)}
                  </span>
                </div>
                <div className="progress-bar large">
                  <div
                    className="progress-fill"
                    style={{
                      width: `${selectedTorrent.progress}%`,
                      backgroundColor: getStatusColor(selectedTorrent.status),
                    }}
                  />
                </div>
              </div>

              {/* Stats */}
              <div className="stats-grid">
                <div className="stat-card">
                  <div className="stat-label">Downloaded</div>
                  <div className="stat-value">{formatBytes(selectedTorrent.downloaded)}</div>
                </div>
                <div className="stat-card">
                  <div className="stat-label">Uploaded</div>
                  <div className="stat-value">{formatBytes(selectedTorrent.uploaded)}</div>
                </div>
                <div className="stat-card">
                  <div className="stat-label">Down Speed</div>
                  <div className="stat-value">
                    {formatSpeed(selectedTorrent.download_speed)}
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-label">Up Speed</div>
                  <div className="stat-value">
                    {formatSpeed(selectedTorrent.upload_speed)}
                  </div>
                </div>
              </div>

              {/* Controls */}
              <div className="controls">
                {selectedTorrent.status === "Downloading" && (
                  <button className="btn btn-pause">Pause</button>
                )}
                {selectedTorrent.status === "Paused" && (
                  <button className="btn btn-resume">Resume</button>
                )}
                <button className="btn btn-remove">Remove</button>
              </div>
            </div>
          ) : (
            <div className="welcome-section">
              <h2>No Torrent Selected</h2>
              <p>Select a torrent from the list to view details</p>
              {torrents.length === 0 && (
                <div className="info-box">
                  <h3>M1 - Download a Real Torrent</h3>
                  <ul className="feature-list">
                    <li>✓ Bencode decoder/encoder</li>
                    <li>✓ Torrent parser and info hash</li>
                    <li>✓ HTTP tracker support</li>
                    <li>✓ TCP peer connections</li>
                    <li>✓ BitTorrent handshake protocol</li>
                    <li>✓ Peer messaging (choke, unchoke, have, request)</li>
                    <li>✓ Piece verification with SHA-1</li>
                    <li>✓ Disk write operations</li>
                    <li>✓ Progress tracking</li>
                  </ul>
                </div>
              )}
            </div>
          )}
        </main>
      </div>
    </div>
  );
};

export default App;
