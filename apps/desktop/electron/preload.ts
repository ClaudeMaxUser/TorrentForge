import { contextBridge, ipcRenderer } from "electron";

// Expose IPC API to renderer
contextBridge.exposeInMainWorld("electronAPI", {
  getTorrents: () => ipcRenderer.invoke("get-torrents"),
  addTorrent: (file: string) => ipcRenderer.invoke("add-torrent", file),
  removeTorrent: (id: string) => ipcRenderer.invoke("remove-torrent", id),
  pauseTorrent: (id: string) => ipcRenderer.invoke("pause-torrent", id),
  resumeTorrent: (id: string) => ipcRenderer.invoke("resume-torrent", id),

  // Event listeners
  onTorrentUpdate: (callback: (data: any) => void) => {
    ipcRenderer.on("torrent-update", (_, data) => callback(data));
  },
});
