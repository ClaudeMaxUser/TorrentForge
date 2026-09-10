# Development Guide

## Prerequisites

- Rust 1.70+ (from [rustup.rs](https://rustup.rs))
- Node.js 18+ (for Electron/React)
- npm or yarn

## Setup

1. Clone the repository
2. Install dependencies:
   ```bash
   # Rust dependencies
   cd apps/torrent-engine
   cargo check
   
   # Desktop dependencies
   cd ../desktop
   npm install
   ```

## Development Workflow

### Working on the Torrent Engine

```bash
cd apps/torrent-engine

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Build debug binary
cargo build

# Build release
cargo build --release
```

### Working on the Desktop App

```bash
cd apps/desktop

# Start development server
npm run dev

# Run tests
npm test

# Build for production
npm run build
```

### Running Both

```bash
cd apps/desktop
npm run dev
```

This concurrently runs:
- React dev server on http://localhost:3000
- Electron pointing to the dev server
- Typescript watching electron/

## Code Organization

### Rust Engine (`apps/torrent-engine/`)

- `src/lib.rs` — Main engine exports
- `src/error.rs` — Error types
- `src/bencode/` — Bencode encoding/decoding
- `src/torrent/` — Torrent management
- `tests/` — Integration tests

### Desktop (`apps/desktop/`)

- `electron/main.ts` — Electron main process
- `electron/preload.ts` — IPC bridge
- `renderer/` — React UI application

### Shared Types (`packages/shared/`)

- Types used in IPC communication between Rust and Electron

## Testing

### Rust Tests

```bash
cargo test --workspace
cargo test --package torrent-engine
cargo test bencode  # Run specific module tests
```

### React Tests

```bash
npm test --prefix apps/desktop
```

## Git Workflow

1. Create a feature branch: `git checkout -b feature/my-feature`
2. Make changes and commit: `git commit -m "feat: describe change"`
3. Push: `git push origin feature/my-feature`
4. Create a pull request on GitHub

## Debugging

### Rust

Enable debug logging:
```bash
RUST_LOG=debug cargo run
RUST_LOG=torrent_engine=trace cargo test -- --nocapture
```

### Electron

Press `Ctrl+Shift+I` to open DevTools.

## Performance Profiling

### Rust

Use `cargo bench` for benchmarks (when available).

### Electron/React

Use Chrome DevTools Performance tab.

## Building for Release

```bash
cd apps/desktop
npm run build
npm run dist
```

This creates platform-specific installers in `out/`.
