// Electron IPC API types
declare global {
  interface Window {
    electronAPI: {
      // Torrent management
      addTorrent(filePath: string): Promise<{ success: boolean; id: number }>;
      removeTorrent(id: string): Promise<{ success: boolean }>;
      pauseTorrent(id: string): Promise<{ success: boolean }>;
      resumeTorrent(id: string): Promise<{ success: boolean }>;
      getTorrents(): Promise<{ torrents: any[] }>;
      getTorrentInfo(id: string): Promise<any>;

      // Engine control
      getEngineStatus(): Promise<{
        status: string;
        version: string;
        uptime: number;
      }>;
      getEngineConfig(): Promise<{
        downloadDir: string;
        maxPeers: number;
        listenPort: number;
      }>;
      shutdown(): Promise<void>;

      // Event listeners
      onEngineEvent(
        callback: (event: any, data: any) => void
      ): void;
      onTorrentUpdate(
        callback: (event: any, data: any) => void
      ): void;
      onPeerUpdate(
        callback: (event: any, data: any) => void
      ): void;
    };
  }
}

export {};
