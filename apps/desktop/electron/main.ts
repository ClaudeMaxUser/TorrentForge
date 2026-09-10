import { app, BrowserWindow, ipcMain } from "electron";
import * as path from "path";

let mainWindow: BrowserWindow | null = null;

// Request/response ID counter for JSON-RPC
let requestId = 0;
function nextRequestId(): number {
  return ++requestId;
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      nodeIntegration: false,
      contextIsolation: true,
    },
  });

  const isDev = process.env.NODE_ENV === "development";
  const startUrl = isDev
    ? "http://localhost:3000"
    : `file://${path.join(__dirname, "../renderer/build/index.html")}`;

  mainWindow.loadURL(startUrl);

  if (isDev) {
    mainWindow.webContents.openDevTools();
  }

  mainWindow.on("closed", () => {
    mainWindow = null;
  });
}

app.on("ready", createWindow);

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  if (mainWindow === null) {
    createWindow();
  }
});

// Torrent Management
ipcMain.handle("torrent-add", async (event, filePath: string) => {
  const id = nextRequestId();
  const request = {
    jsonrpc: "2.0",
    method: "torrent.add",
    params: { path: filePath },
    id,
  };
  // TODO: Send to engine via IPC/socket
  console.log("[RPC Request]", request);
  return { success: true, id };
});

ipcMain.handle("torrent-remove", async (event, torrentId: string) => {
  const id = nextRequestId();
  const request = {
    jsonrpc: "2.0",
    method: "torrent.remove",
    params: { id: torrentId },
    id,
  };
  console.log("[RPC Request]", request);
  return { success: true };
});

ipcMain.handle("torrent-pause", async (event, torrentId: string) => {
  const id = nextRequestId();
  const request = {
    jsonrpc: "2.0",
    method: "torrent.pause",
    params: { id: torrentId },
    id,
  };
  console.log("[RPC Request]", request);
  return { success: true };
});

ipcMain.handle("torrent-resume", async (event, torrentId: string) => {
  const id = nextRequestId();
  const request = {
    jsonrpc: "2.0",
    method: "torrent.resume",
    params: { id: torrentId },
    id,
  };
  console.log("[RPC Request]", request);
  return { success: true };
});

ipcMain.handle("torrent-list", async (event) => {
  const id = nextRequestId();
  const request = {
    jsonrpc: "2.0",
    method: "torrent.list",
    params: {},
    id,
  };
  console.log("[RPC Request]", request);
  // Return empty list for M0
  return { torrents: [] };
});

// Engine Status
ipcMain.handle("engine-status", async (event) => {
  const id = nextRequestId();
  const request = {
    jsonrpc: "2.0",
    method: "engine.status",
    params: {},
    id,
  };
  console.log("[RPC Request]", request);
  return {
    status: "ready",
    version: "0.1.0",
    uptime: 0,
  };
});

ipcMain.handle("engine-config", async (event) => {
  const id = nextRequestId();
  const request = {
    jsonrpc: "2.0",
    method: "engine.config",
    params: {},
    id,
  };
  console.log("[RPC Request]", request);
  return {
    downloadDir: process.env.HOME || "/tmp",
    maxPeers: 100,
    listenPort: 6881,
  };
});
