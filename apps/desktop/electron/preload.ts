import { contextBridge, ipcRenderer } from "electron";

// Expose structured IPC API to renderer with JSON-RPC wrapper
contextBridge.exposeInMainWorld("electronAPI", {
  // Torrent management
  addTorrent: (filePath: string) =>
    ipcRenderer.invoke("torrent-add", filePath),
  removeTorrent: (id: string) =>
    ipcRenderer.invoke("torrent-remove", id),
  pauseTorrent: (id: string) =>
    ipcRenderer.invoke("torrent-pause", id),
  resumeTorrent: (id: string) =>
    ipcRenderer.invoke("torrent-resume", id),
  getTorrents: () =>
    ipcRenderer.invoke("torrent-list"),
  getTorrentInfo: (id: string) =>
    ipcRenderer.invoke("torrent-info", id),

  // Engine control
  getEngineStatus: () =>
    ipcRenderer.invoke("engine-status"),
  getEngineConfig: () =>
    ipcRenderer.invoke("engine-config"),
  shutdown: () =>
    ipcRenderer.invoke("engine-shutdown"),

  // Event listeners for engine events
  onEngineEvent: (callback: (event: any, data: any) => void) => {
    ipcRenderer.on("engine-event", callback);
  },

  onTorrentUpdate: (callback: (event: any, data: any) => void) => {
    ipcRenderer.on("torrent-update", callback);
  },

  onPeerUpdate: (callback: (event: any, data: any) => void) => {
    ipcRenderer.on("peer-update", callback);
  },
});

