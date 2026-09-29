# Torrent Forge

A production-oriented, cross-platform BitTorrent client with a custom Rust networking engine, modern React/Electron desktop UI, and advanced peer coordination.

## Project Vision

Build a systems-engineering showcase that implements the BitTorrent protocol from the wire level, demonstrating distributed networking, peer coordination, DHT, concurrent I/O, disk scheduling, fault-tolerant systems, and desktop application engineering.

This is **not** a wrapper around an existing torrent library. The core objective is a custom BitTorrent engine.

## Architecture

```
┌─────────────────────────────────────────────┐
│             React + TypeScript UI           │
│                                             │
│ Dashboard │ Torrents │ Peers │ Speed │ Logs │
└──────────────────────┬──────────────────────┘
                       │ IPC
┌──────────────────────▼──────────────────────┐
│              Electron Shell                 │
│  Window management │ Tray │ Notifications   │
└──────────────────────┬──────────────────────┘
                       │
┌──────────────────────▼──────────────────────┐
│           Torrent Engine (Rust)             │
│                                             │
│  Torrent Manager                            │
│  ├── Tracker Engine                         │
│  ├── Peer Manager                           │
│  ├── Piece Scheduler                        │
│  ├── Disk Manager                           │
│  ├── DHT / UDP                              │
│  ├── Magnet Metadata                        │
│  ├── Choking / Rarest-first                 │
│  ├── Connection Manager                     │
│  └── Session Manager                        │
└──────────────────────┬──────────────────────┘
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
          TCP/UDP             Filesystem
```

## Technology Stack

- **Engine**: Rust
- **Desktop**: Electron + React + TypeScript
- **Communication**: JSON-RPC over IPC
- **Testing**: Rust integration + React testing
- **CI/CD**: GitHub Actions

## Development Status

### M0 - Application + Engine Foundation (Complete ✓)
- [x] Monorepo structure
- [x] Rust workspace setup
- [x] Electron shell with dev server
- [x] React UI scaffolding
- [x] Structured logging (tracing + JSON output)
- [x] Configuration system (file-based JSON config)
- [x] IPC abstraction (JSON-RPC protocol over IPC)
- [x] Shared event model (engine events)
- [x] Test infrastructure (unit + integration)
- [x] CI/CD pipeline (GitHub Actions)
- [x] Git repository & GitHub push

**Features working:**
- Structured JSON logging throughout
- Configuration management with validation
- Clean JSON-RPC 2.0 request/response protocol
- Type-safe IPC between Electron and engine
- Test skeletons for future implementation

### M1 - Download a Real Torrent (Complete ✓)
- [x] Bencode decoder/encoder
- [x] Torrent parser
- [x] Info hash calculation
- [x] HTTP tracker
- [x] TCP peer connection
- [x] BitTorrent handshake
- [x] Peer messaging protocol
- [x] Piece requests & verification
- [x] Disk write
- [x] Progress UI

**Features working:**
- Complete bencode encoding/decoding with type support
- .torrent file parsing (single and multi-file torrents)
- SHA-1 based info hash calculation
- HTTP tracker peer discovery and announce
- Async TCP peer connections with timeout protection
- BitTorrent handshake protocol implementation
- 8 core peer messages (choke, unchoke, have, bitfield, request, piece, cancel)
- Block assembly and SHA-1 piece verification
- Async disk I/O with path sanitization
- Real-time download progress UI with speed indicators
- 30 unit tests covering all components (100% pass rate)

### M2+ - Advanced Features
- See [torrent-client-reference.md](torrent-client-reference.md) for full roadmap

## Project Structure

```
torrent-forge/
├── apps/
│   ├── desktop/
│   │   ├── electron/       # Electron main process
│   │   └── renderer/       # React UI
│   └── torrent-engine/     # Rust BitTorrent engine
│
├── packages/
│   └── shared/             # Shared Rust types for IPC
│
├── tests/                  # Integration tests
├── docs/                   # Architecture, protocol, security docs
└── README.md
```

## Getting Started

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Node.js 18+ (for Electron/React)
- npm or yarn

### Development

1. **Install dependencies**
   ```bash
   # Rust workspace is ready
   cd apps/torrent-engine
   cargo check
   
   # Electron/React
   cd apps/desktop
   npm install
   ```

2. **Run development environment**
   ```bash
   cd apps/desktop
   npm run dev
   ```

3. **Run tests**
   ```bash
   cargo test --workspace
   npm test --prefix apps/desktop
   ```

### Build for Production

```bash
npm run build --prefix apps/desktop
cargo build --release
npm run dist --prefix apps/desktop
```

## Principles

1. **Correctness before optimization** — Get protocol behavior correct first
2. **Explicit state machines** — Clear lifecycle states for torrents, peers, trackers
3. **Failure is normal** — Network failures are expected and recoverable
4. **Bounded resources** — Never allow unbounded queues or allocations
5. **Timeouts everywhere** — Every network operation has a timeout
6. **Clear separation** — UI, IPC, protocol logic, storage kept distinct

## References

- [BitTorrent Specification (BEP 3)](http://www.bittorrent.org/beps/bep_0003.html)
- [DHT Specification (BEP 5)](http://www.bittorrent.org/beps/bep_0005.html)
- [Project Reference Document](torrent-client-reference.md)

## License

MIT