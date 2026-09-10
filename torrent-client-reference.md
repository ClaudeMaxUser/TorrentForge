# Torrent Client Project Reference

## 1. Project Vision

Build a production-quality, cross-platform BitTorrent desktop client designed to be a strong portfolio and systems-engineering showcase.

This should NOT be treated as a simple Electron wrapper around an existing torrent library. The core objective is to implement a custom BitTorrent engine and expose it through a polished desktop application.

Primary engineering themes:

- BitTorrent protocol implementation
- Peer-to-peer networking
- TCP and UDP networking
- DHT / Kademlia
- Concurrent I/O
- Piece scheduling
- Disk I/O and storage management
- Fault tolerance and crash recovery
- Security hardening
- Observability and diagnostics
- Automated testing
- Cross-platform desktop packaging

The final project should demonstrate substantially more than standard CRUD/full-stack development.

---

## 2. Recommended Product Positioning

Suggested positioning:

> A production-grade, cross-platform BitTorrent client with a custom torrent engine, modern desktop UI, peer/network management, observability, fault tolerance, and advanced download scheduling.

Possible project name can be decided separately.

The README should explain why the project exists:

> The purpose of the project is to implement the BitTorrent protocol from the wire level and explore distributed networking, peer coordination, DHT, concurrent I/O, disk scheduling, fault-tolerant systems, and desktop application engineering.

---

## 3. High-Level Architecture

Preferred architecture:

```text
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

### Architectural principle

Keep the torrent engine independent of Electron.

The engine should be usable independently, for example by a CLI or another frontend.

Conceptually:

```text
torrent-engine
      │
      ├── CLI
      ├── Electron GUI
      └── REST/WebSocket API (future possibility)
```

Electron should be the desktop presentation layer, not the place where the torrent protocol logic lives.

---

## 4. Technology Stack

### Desktop

- Electron
- React
- TypeScript

### Torrent engine

Preferred:

- Rust

Reason for choosing Rust:

- Strong networking ecosystem
- Memory safety
- Excellent concurrency primitives
- High performance
- Good fit for long-running networking workloads
- Demonstrates systems programming ability
- Keeps the project technically distinctive

Alternative for a faster first implementation:

- Node.js + TypeScript

However, if project quality and portfolio value are the priority, Rust should be preferred for the final engine.

### Communication

Electron renderer ↔ Electron main:

- Electron IPC

Electron ↔ torrent engine:

Preferred architecture:

- Local process communication
- JSON-RPC or another clearly defined IPC protocol

The exact transport can be selected during implementation.

### Storage

- Local filesystem
- Structured persistent session/state storage
- Resume metadata
- Optional disk cache

### UI

React + TypeScript.

Suggested UI style:

- Modern
- Dense but readable
- qBittorrent-inspired information density
- Developer-tool-like diagnostics
- Clear torrent status and networking information
- Dark/light theme support if practical

---

# 5. Core Domain Model

Important conceptual objects:

```text
TorrentEngine
Torrent
Peer
Tracker
DhtNode
PieceManager
StorageManager
SessionManager
ConnectionManager
```

Possible responsibilities:

### TorrentEngine

Owns global engine state and torrent lifecycle.

### Torrent

Represents one torrent session.

Responsibilities:

- Metadata
- State
- Progress
- Peer coordination
- Tracker coordination
- Piece scheduling

### Peer

Represents a remote peer.

Track:

- IP/port
- Client identification
- Connection state
- Choking state
- Interested state
- Bitfield
- Download rate
- Upload rate
- Latency
- Connection duration

### Tracker

Handles:

- HTTP/HTTPS trackers
- UDP trackers
- Announce requests
- Peer discovery
- Tracker failures/retries

### PieceManager

Handles:

- Piece availability
- Piece selection
- Block requests
- Verification
- Completion state
- Rarest-first scheduling
- End-game mode

### StorageManager

Handles:

- File allocation
- Piece-to-file mapping
- Random-access writes
- Reads
- Verification
- Resume state
- Disk cache
- Multi-file torrents

### DHT

Handles:

- Kademlia routing
- Node IDs
- Routing buckets
- Peer discovery
- UDP communication

---

# 6. Torrent Lifecycle

A typical lifecycle should eventually look like:

```text
Add .torrent / Magnet
        ↓
Parse metadata
        ↓
Determine info hash
        ↓
Discover peers
 ┌──────┴─────────┐
 │                │
Tracker          DHT
 │                │
 └──────┬─────────┘
        ↓
Connect to peers
        ↓
BitTorrent handshake
        ↓
Exchange bitfields
        ↓
Peer scheduling
        ↓
Request blocks
        ↓
Receive blocks
        ↓
Assemble pieces
        ↓
SHA-1 verification
        ↓
Write to disk
        ↓
All pieces complete?
        ↓
       YES
        ↓
     Seeding
```

---

# 7. MVP

The MVP should focus on getting a real torrent downloaded through the BitTorrent protocol.

## MVP scope

### Metadata

- bencode decoder
- `.torrent` parsing
- Single-file torrent support initially
- Info hash calculation
- Piece metadata extraction

### Tracker

- HTTP tracker support
- Basic announce request
- Peer list parsing

### Networking

- TCP peer connections
- BitTorrent handshake
- Peer message parsing
- Peer state management

Support core messages:

- choke
- unchoke
- interested
- not interested
- have
- bitfield
- request
- piece
- cancel

### Pieces

- Piece/block requests
- Piece assembly
- SHA-1 verification
- Basic piece selection
- Download progress

### Storage

- Write completed data to disk
- Basic file handling
- Basic path validation

### Desktop

React + Electron UI showing:

- Add torrent
- Torrent list
- Download progress
- Download speed
- Upload speed
- Status
- ETA
- Basic logs

## MVP success criteria

A user should be able to:

1. Open the application
2. Add a valid `.torrent`
3. Discover peers through an HTTP tracker
4. Establish peer connections
5. Download pieces
6. Verify pieces
7. Write the final file
8. See live progress in the UI

Do not add DHT, magnet links, or advanced scheduling before this works reliably.

---

# 8. Version 1 - Functional Desktop Client

After MVP, make it usable as a proper torrent client.

## Features

### Torrent management

- Multiple simultaneous torrents
- Pause
- Resume
- Stop
- Remove torrent
- Remove torrent + data
- Recheck torrent
- Completed state
- Seeding state

### Files

- Multi-file torrent support
- File selection
- Download directory selection
- Path sanitization
- Existing file detection

### Resume

Application restart should not require restarting downloads from zero.

On restart:

```text
Load torrent state
      ↓
Inspect existing files
      ↓
Verify necessary pieces
      ↓
Restore state
      ↓
Reconnect to peers
      ↓
Resume download
```

### Seeding

After completion:

- Switch to seeding
- Accept peer connections
- Upload pieces
- Track upload statistics

### Rate limits

- Global download limit
- Global upload limit
- Per-torrent limits

### UI

Main dashboard:

```text
┌───────────────────────────────────────────────────────────────┐
│  Torrent Client                              ↓ 18.4 MB/s ↑ 2.1│
├────────────┬──────────────────────────────────────────────────┤
│ Torrents   │ Ubuntu ISO                                       │
│            │ ████████████████████░░░░ 82.4%                   │
│ + Add      │                                                  │
│            │ ↓ 18.4 MB/s       ETA 14m                        │
│ Active  3  │                                                  │
│ Seeding 2  │                                                  │
│ Completed  │                                                  │
│            ├──────────────────────────────────────────────────┤
│            │ Peers Pieces Files Trackers Logs                 │
└────────────┴──────────────────────────────────────────────────┘
```

---

# 9. Version 2 - Advanced BitTorrent Networking

This version should significantly increase the technical depth.

## Magnet links

Support:

```text
magnet:?xt=urn:btih:...
```

Flow:

```text
Magnet
  ↓
DHT / peer discovery
  ↓
Find peers
  ↓
Metadata extension
  ↓
Obtain torrent metadata
  ↓
Normal torrent lifecycle
```

## DHT

Implement Kademlia-style DHT support:

- UDP transport
- Node IDs
- Routing table
- Routing buckets
- ping
- find_node
- get_peers
- announce_peer

## UDP trackers

Support UDP tracker protocol.

## Advanced piece scheduling

Implement:

### Rarest-first

Track piece availability across peers.

Example:

```text
Piece availability

0   ██████████  10 peers
1   █████       5 peers
2   ██          2 peers  ← priority
3   ███████     7 peers
4   █           1 peer   ← priority
```

Also consider:

- Request pipelining
- End-game mode
- Duplicate request cancellation
- Peer scoring
- Choking/unchoking
- Piece availability tracking

---

# 10. Version 3 - Production Hardening

This version focuses on reliability, security, and observability.

## Crash recovery

Persist enough state that unexpected shutdowns are safe.

Requirements:

- Atomic state updates where practical
- Safe restart
- Partial download recovery
- Reverification after crash when required

## Storage improvements

Add:

- Disk cache
- Buffered writes
- Efficient piece-to-file mapping
- Sparse files where appropriate
- Disk throughput monitoring
- Pending write tracking

Example:

```text
Disk

Read        184 MB/s
Write        72 MB/s
Cache        64 MB
Pending       12 writes
```

## Security

Threat model should explicitly consider:

### Malformed torrent metadata

Avoid parser crashes and unreasonable allocations.

### Malicious paths

Prevent:

```text
../../../../important-file
```

from escaping the selected download directory.

### Disk exhaustion

Enforce:

- Available disk checks
- Maximum allocation safeguards
- Clear errors when storage is insufficient

### Peer/network abuse

Consider:

- Connection limits
- Message-size limits
- Timeouts
- Idle peer cleanup
- Resource exhaustion protection

### Malicious tracker responses

Validate data before use.

### Symlink/path attacks

Handle filesystem edge cases carefully.

---

# 11. Observability

Observability should be a first-class feature.

## Structured logs

Example:

```text
2026-09-01T16:42:31.421Z
peer.connected
peer=185.x.x.x
client=qBittorrent
latency=42ms
```

Useful event types:

- torrent.added
- torrent.started
- torrent.paused
- torrent.completed
- peer.connected
- peer.disconnected
- peer.handshake
- peer.unchoked
- piece.requested
- piece.received
- piece.verified
- piece.failed
- tracker.request
- tracker.failure
- dht.query

## Developer diagnostics panel

Show a live event stream:

```text
16:42:31  peer.connected
16:42:31  peer.handshake
16:42:32  peer.bitfield
16:42:32  peer.unchoked
16:42:32  piece.request
16:42:33  piece.received
16:42:33  piece.verified
```

---

# 12. Performance Dashboard

Expose meaningful engine metrics.

## Network

```text
↓ Download      18.42 MB/s
↑ Upload          2.17 MB/s
Connections           84
Active peers           51
DHT nodes           4,821
```

## Disk

```text
Read              21.4 MB/s
Write             18.1 MB/s
Cache               64 MB
```

## Torrent

```text
Pieces           2,481 / 3,921
Availability           8.4
ETA                 14m 32s
```

Live charts can show:

- Download speed
- Upload speed
- Peer count
- Disk throughput
- Piece completion

---

# 13. Testing Strategy

Testing should be a major part of the project.

Suggested structure:

```text
tests/
├── bencode/
├── protocol/
├── tracker/
├── dht/
├── pieces/
├── storage/
├── torrent/
└── integration/
```

## Unit tests

Test:

- Bencode parsing
- Bencode encoding
- Info hash generation
- Message parsing
- Message serialization
- Bitfield operations
- Piece selection
- Piece verification
- File mapping
- Path sanitization

## Integration tests

Create simulated peers.

```text
Test torrent
      │
      ├── Peer A
      ├── Peer B
      ├── Peer C
      └── Peer D
```

Simulate:

- Peer disconnects
- Slow peers
- Corrupt pieces
- Missing pieces
- Tracker failure
- DHT failure
- Network interruption
- Application crash
- Disk errors

The client should recover gracefully.

## Important test principle

Do not optimize for large numbers of superficial UI tests.

Prioritize tests around the networking engine, concurrency, protocol correctness, storage, and failure handling.

---

# 14. Benchmarking

Performance benchmarks should be included in the repository.

Potential benchmark categories:

### Piece scheduling

Measure scheduling overhead under different peer counts.

### Peer connections

Measure connection handling with:

- 10 peers
- 50 peers
- 100 peers
- 500 peers

### Disk

Measure:

- Sequential write throughput
- Random piece writes
- Verification throughput
- Cache effectiveness

### DHT

Measure:

- Query latency
- Routing-table operations
- Concurrent queries
- Node discovery

### Memory

Measure memory usage with:

- 1 torrent
- 10 torrents
- 100 torrents
- Large peer counts

Avoid inventing performance claims. All published metrics must come from reproducible benchmarks.

---

# 15. CI/CD

GitHub Actions should eventually perform:

```text
Push
 ↓
Lint
 ↓
Typecheck
 ↓
Unit tests
 ↓
Integration tests
 ↓
Rust tests
 ↓
Build engine
 ↓
Build Electron
 ↓
Package
 ↓
Release artifacts
```

Target platforms:

- Windows
- Linux
- macOS

Possible release artifacts:

- Windows installer
- Linux package
- macOS application/package

---

# 16. Repository Structure

Suggested monorepo:

```text
torrent-client/
│
├── apps/
│   ├── desktop/
│   │   ├── electron/
│   │   │   ├── main.ts
│   │   │   ├── preload.ts
│   │   │   └── ipc/
│   │   │
│   │   └── renderer/
│   │       ├── React
│   │       └── UI
│   │
│   └── torrent-engine/
│       ├── bencode/
│       ├── tracker/
│       ├── peer/
│       ├── dht/
│       ├── torrent/
│       ├── pieces/
│       ├── storage/
│       └── magnet/
│
├── packages/
│   └── shared/
│
├── tests/
│
├── docs/
│   ├── architecture/
│   ├── protocol/
│   ├── security/
│   └── benchmarks/
│
└── README.md
```

The exact structure can change as implementation begins. Keep module boundaries based on responsibility rather than forcing this exact directory tree.

---

# 17. UI Feature Plan

## Main navigation

Potential sections:

- Torrents
- Downloads
- Seeding
- Completed
- Peers
- Trackers
- DHT
- Logs
- Settings
- Diagnostics

## Torrent details

Tabs:

- Overview
- Peers
- Pieces
- Files
- Trackers
- Speed
- Logs

## Overview

Show:

- Name
- State
- Progress
- Download speed
- Upload speed
- ETA
- Downloaded
- Uploaded
- Ratio
- Availability
- Active peers

## Peers

Show:

- IP
- Client
- Latency
- Download rate
- Upload rate
- Bitfield/piece availability
- Choking state
- Connection duration

## Pieces

Visualize piece completion and availability.

## Trackers

Show:

- Tracker URL
- Status
- Last announce
- Next announce
- Peer count
- Error state

## DHT

Show:

- Node count
- Queries
- Routing buckets
- Recent activity

---

# 18. Product Quality Principles

The project should follow these principles:

### Correctness before optimization

Get protocol behavior correct before optimizing throughput.

### Explicit state machines

Torrent, peer, tracker, and piece lifecycle states should be explicit.

### Failure is normal

Network failures should be expected and recoverable.

### Backpressure

Do not allow uncontrolled queues or unbounded memory usage.

### Cancellation

Long-running operations should support cancellation where practical.

### Timeouts

Every network operation that can hang should have appropriate timeout behavior.

### Resource limits

Bound:

- Connections
- Memory
- Message sizes
- Pending requests
- Disk operations

### Clear separation

Keep UI, IPC, protocol logic, storage, and orchestration separate.

---

# 19. Potential Advanced Features

These should only be considered after the core engine is reliable.

Possible future work:

- Peer exchange (PEX)
- Local peer discovery
- IPv6
- NAT traversal
- UPnP/NAT-PMP
- Sequential download mode
- Download priorities
- File-level priorities
- Bandwidth scheduling
- Queue management
- Ratio limits
- Seed ratio/time limits
- RSS torrent feeds
- Remote control API
- WebSocket monitoring API
- Headless mode
- CLI
- System tray mode
- Automatic updates
- Themes
- Accessibility improvements

Do not add features merely to increase the feature count. Each feature should have a clear technical or product reason.

---

# 20. Recommended Development Order

## Stage 0 - Foundation

- Repository
- Rust workspace
- Electron shell
- React application
- IPC boundary
- Logging
- Configuration
- Basic CI

## Stage 1 - MVP networking

- Bencode
- Torrent parser
- Info hash
- HTTP tracker
- TCP peer
- Handshake
- Core peer messages
- Piece requests
- Piece verification
- Disk writes

## Stage 2 - Usable client

- Multiple torrents
- Pause/resume
- Multi-file torrents
- Resume
- Seeding
- Rate limiting
- Recheck
- Better UI

## Stage 3 - Advanced networking

- Magnet links
- Metadata extension
- UDP tracker
- DHT
- Kademlia
- Rarest-first
- End-game mode
- Choking/unchoking
- Peer scoring

## Stage 4 - Production hardening

- Crash recovery
- Disk cache
- Storage optimizations
- Security hardening
- Resource limits
- Network resilience
- Better error handling

## Stage 5 - Showcase quality

- Observability
- Diagnostics
- Performance dashboards
- Benchmarks
- Integration test network
- Cross-platform builds
- Documentation
- Architecture diagrams
- Demo video/GIF
- Release binaries

---

# 21. Definition of Done

A feature should generally not be considered complete until:

- Implementation exists
- Error cases are handled
- Relevant tests exist
- Logging/diagnostics are sufficient
- Cancellation is considered
- Resource usage is bounded
- Documentation is updated
- UI state is correct where applicable
- CI passes
- Behavior has been tested against realistic failure scenarios

For networking features, test both successful and failure paths.

---

# 22. README Showcase Strategy

The README should make the technical depth obvious immediately.

Recommended structure:

```text
# Project Name

Short one-line description

[Demo GIF / Screenshot]

## Features

## Architecture

## How BitTorrent Works in This Client

## Protocol Support

## Performance

## Security

## Testing

## Development

## Roadmap

## Why I Built This
```

Include an architecture diagram.

Include screenshots of:

1. Main torrent dashboard
2. Torrent details
3. Peer dashboard
4. Piece visualization
5. DHT diagnostics
6. Logs/diagnostics
7. Performance dashboard

---

# 23. Portfolio/Resume Value

The project should demonstrate:

- Distributed systems concepts
- Networking
- TCP/UDP
- Peer-to-peer protocols
- DHT/Kademlia
- Concurrent programming
- Rust
- TypeScript
- React
- Electron
- Filesystem/storage engineering
- Fault tolerance
- Security
- Testing
- Performance engineering
- Observability
- Cross-platform software delivery

The strongest portfolio story is not:

> "I built a torrent downloader."

It is:

> "I designed and implemented a cross-platform BitTorrent client with a custom Rust networking engine, peer scheduling, DHT-based discovery, concurrent piece transfer, resumable storage, crash recovery, observability, and a React/Electron desktop interface."

Only claim features that have actually been implemented and tested.

---

# 24. Copilot Instructions

When using GitHub Copilot, follow these project rules:

1. Prefer correctness and maintainability over short implementations.
2. Do not introduce an external torrent library to replace the custom torrent engine unless explicitly requested.
3. Keep the Rust engine independent of Electron.
4. Keep UI concerns out of the torrent engine.
5. Avoid global mutable state unless justified.
6. Use explicit state machines for protocol/session state.
7. Bound queues and resource usage.
8. Add timeouts to network operations.
9. Handle peer disconnects gracefully.
10. Validate all external/network input.
11. Never trust torrent file paths.
12. Avoid unbounded memory allocation from remote data.
13. Prefer structured errors over generic strings.
14. Add tests for protocol logic.
15. Add failure-path tests, not only happy-path tests.
16. Keep concurrency safe and explicit.
17. Document non-obvious protocol decisions.
18. Do not optimize based on assumptions. Benchmark first.
19. Do not claim a benchmark result without actually measuring it.
20. Preserve module boundaries.
21. Prefer small, composable components.
22. Keep public APIs minimal.
23. Do not silently swallow errors.
24. Make cancellation and shutdown behavior explicit.
25. Update documentation when architecture or protocol behavior changes.

---

# 25. Copilot Development Workflow

For every significant task:

```text
1. Understand the existing architecture
2. Identify the responsible module
3. Define the state/data flow
4. Implement the smallest correct change
5. Add unit tests
6. Add integration/failure tests where appropriate
7. Run formatting/lint/type checks
8. Run relevant tests
9. Review resource usage
10. Update documentation if behavior changed
```

Avoid asking Copilot to generate the entire torrent client at once.

Use small milestones such as:

> Implement the bencode decoder with support for integers, byte strings, lists, and dictionaries. Add malformed-input tests and ensure parser allocations are bounded.

Then:

> Implement torrent metadata parsing using the existing bencode module. Add info-hash calculation and tests against known torrent metadata.

Then:

> Implement the BitTorrent handshake state machine. Do not implement piece downloading yet.

This approach keeps the codebase understandable and makes it easier to verify correctness.

---

# 26. First Milestone

The first concrete milestone should be:

## "M0 - Application + Engine Foundation"

Deliver:

- Monorepo
- Rust engine
- Electron shell
- React UI
- IPC abstraction
- Shared event model
- Structured logging
- Configuration system
- Basic error handling
- CI
- Unit test setup
- Integration test setup

The second milestone should be:

## "M1 - Download a Real Torrent"

Deliver:

- Bencode
- Torrent parser
- Info hash
- HTTP tracker
- TCP peer connection
- Handshake
- Bitfield
- Interested/unchoke flow
- Piece requests
- Piece verification
- Disk write
- Basic progress UI

Only after M1 works reliably should advanced functionality begin.

---

# 27. Guiding Principle

Build this as a **systems project with a desktop UI**, not as a desktop UI project with torrent functionality.

The torrent engine is the core product.

Electron + React provides the user experience.

Rust provides the networking/system layer.

The portfolio value comes from the engineering depth: protocol implementation, concurrency, distributed networking, storage, reliability, security, observability, testing, and measurable performance.
