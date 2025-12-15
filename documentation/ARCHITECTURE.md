# TrinityChain Architecture

## Overview

TrinityChain is a geometric blockchain where value is represented as triangles. The codebase is organized into logical layers that separate concerns and enable optional functionality through feature flags.

## Core Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Tools (20+ binaries)                  │
├─────────────────────────────────────────────────────────────┤
│  Wallet Tools │ Mining │ Transactions │ Node Tools │ Utils  │
├─────────────────────────────────────────────────────────────┤
│                      TrinityChain Library                     │
├──────────────┬────────────────────┬─────────────────────────┤
│Core Modules  │ State Management   │ Integration Modules    │
│              │                    │ (Optional Features)    │
└──────────────┴────────────────────┴─────────────────────────┘
```

## Module Organization

### 1. Core Blockchain (4 modules)

The fundamental blockchain logic.

- **`blockchain.rs`** - Main chain validation, block storage, UTXO management
  - `Blockchain` - Central state machine
  - `validate_block()` - PoW and transaction validation
  - `apply_block()` - State transitions

- **`transaction.rs`** - Transaction types and operations
  - `Transaction` - Transfer, Subdivision, Coinbase variants
  - Input/output structure with signatures
  - Transaction serialization and hashing

- **`mempool.rs`** - Transaction memory pool
  - Pending transaction queue
  - Fee-based ordering and eviction
  - Double-spend detection

### 2. Geometric System (2 modules)

Triangle-based UTXO model using fixed-point arithmetic.

- **`geometry.rs`** - Triangle primitives and calculations
  - `Triangle` struct with I32F32 fixed-point coordinates
  - Shoelace formula for area calculation
  - Geometric validation and bounds checking

- **`fees.rs`** - Fee calculations (geometric)
  - Area-based transaction fees
  - Difficulty adjustment based on network metrics

### 3. Consensus & Mining (1 module)

Proof-of-Work mining implementation.

- **`miner.rs`** - SHA256 PoW mining
  - `mine_block()` - Find valid nonce
  - Difficulty management and retargeting
  - Mining reward calculation

### 4. Cryptography & Security (2 modules)

Cryptographic operations and security utilities.

- **`crypto.rs`** - Signatures and verification
  - secp256k1 ECDSA signing/verification
  - Public key derivation
  - Hash functions (SHA256, RIPEMD160)

- **`security.rs`** - Security utilities
  - Input validation
  - Timing attack prevention
  - Rate limiting utilities

### 5. State Management (4 modules)

Persistent storage and wallet operations.

- **`wallet.rs`** - Wallet operations
  - UTXO selection algorithms
  - Transaction building
  - Balance calculation

- **`hdwallet.rs`** - Hierarchical Deterministic (HD) wallets
  - BIP-39 mnemonic generation
  - BIP-32 key derivation
  - Account and change address management

- **`persistence.rs`** - SQLite database layer
  - Block storage and retrieval
  - Transaction history
  - UTXO set management
  - Schema migrations

- **`cache.rs`** - Performance caching
  - UTXO cache
  - Block metadata cache
  - Validation result caching

### 6. Networking (3 modules)

P2P networking and synchronization.

- **`network.rs`** - P2P networking
  - Peer connection management
  - Message broadcasting
  - Protocol handshakes
  - TCP and WebSocket transports

- **`discovery.rs`** - Peer discovery
  - DNS seed resolution
  - Peer address propagation
  - Bootstrap node management

- **`sync.rs`** - Chain synchronization
  - Block download coordination
  - Fork resolution
  - Initial sync strategy

### 7. Integration (Optional Feature: `api`)

REST API and web server.

- **`api.rs`** - REST API endpoints (feature-gated)
  - Requires: `features = ["api"]`
  - Provides: HTTP interface to blockchain operations
  - Axum web framework integration

### 8. Configuration & Utilities (3 modules)

Configuration, error handling, and utilities.

- **`config.rs`** - Configuration management
  - Network parameters (ports, timeouts)
  - Database settings
  - Consensus rules
  - Loaded from `config.toml`

- **`error.rs`** - Error types
  - `BlockchainError` - Core blockchain errors
  - `TransactionError` - Transaction validation errors
  - Error propagation and conversion

- **`cli.rs`** - CLI utilities
  - Command parsing helpers
  - Output formatting
  - User interaction utilities

- **`addressbook.rs`** - Address book management
  - Named address storage
  - Contact management
  - Export/import functionality

## Feature Flags

The project uses Cargo features to enable optional components:

### Default Features
```toml
[features]
default = ["cli"]
```

All CLI tools always build (essential for CLI-first architecture).

### Optional Features

| Feature | Purpose | Dependencies | Binaries |
|---------|---------|---|----------|
| `api` | REST API server | axum, tower-http | trinity-api, trinity-server |
| `telegram` | Telegram bot integration | teloxide | trinity-telegram-bot |
| `all` | All features | api + telegram | All binaries |

### Building with Features

```bash
# Default (CLI only)
cargo build --release

# With API server
cargo build --release --features api

# With Telegram bot
cargo build --release --features telegram

# All features
cargo build --release --features all

# Minimal (no CLI is unusual but possible)
cargo build --release --no-default-features
```

## CLI Binaries (20+)

Organized by functional category:

### Wallet Management
- `trinity-wallet` - Create, list, and manage wallets
- `trinity-wallet-backup` - Backup wallets with mnemonics
- `trinity-wallet-restore` - Restore wallets from mnemonics

### Mining
- `trinity-miner` - Continuous mining with reward tracking
- `trinity-mine-block` - Mine single blocks

### Transactions
- `trinity-send` - Create and broadcast transactions
- `trinity-balance` - Check wallet balances
- `trinity-history` - View transaction history

### Node & Networking
- `trinity-node` - Run a full node with TUI monitoring
- `trinity-connect` - Connect to peer nodes

### Utilities
- `trinity-addressbook` - Manage address contacts
- `trinity-guestbook` - Public message signing
- `trinity-user` - User profile management

### Optional Integrations
- `trinity-api` (requires `api` feature) - Standalone REST API server
- `trinity-server` (requires `api` feature) - Node + API combined
- `trinity-telegram-bot` (requires `telegram` feature) - Telegram interface

## Data Flow

### Mining Workflow
```
trinity-wallet (read balance)
    ↓
trinity-miner (mine blocks)
    ↓
database (persist blocks)
    ↓
trinity-node (sync peers)
```

### Transaction Workflow
```
trinity-wallet (create transaction)
    ↓
transaction module (validate)
    ↓
mempool (broadcast)
    ↓
miner (include in block)
    ↓
trinity-node (sync network)
```

### REST API Workflow (with `api` feature)
```
HTTP Client (trinity-api:3030)
    ↓
api.rs module
    ↓
blockchain + wallet (operations)
    ↓
HTTP Response
```

## Dependency Organization

### Core & Serialization (5 crates)
- `bincode` - Binary serialization
- `serde` - Serialization framework
- `serde_json` - JSON support

### Cryptography & Security (7 crates)
- `sha2` - SHA256 hashing
- `secp256k1` - ECDSA signatures
- `hex` - Hex encoding/decoding
- `zeroize` - Secure memory clearing

### Database & Persistence (2 crates)
- `rusqlite` - SQLite bindings

### Async & Networking (3 crates)
- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket support

### HTTP & Web - Optional (2 crates)
- `axum` - Web framework (feature: `api`)
- `tower-http` - HTTP utilities (feature: `api`)

### CLI & TUI (8 crates)
- `clap` - CLI argument parsing
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal control
- `indicatif` - Progress bars

### Logging & Tracing (4 crates)
- `log` / `env_logger` - Logging
- `tracing` / `tracing-subscriber` - Distributed tracing

### Integration - Telegram - Optional (1 crate)
- `teloxide` - Telegram bot framework (feature: `telegram`)

## Performance Characteristics

### Scalability
- **Block Size**: Configurable (default: 1MB)
- **Block Time**: ~10 minutes target
- **Network**: TCP + WebSocket P2P
- **Database**: SQLite with connection pooling

### Optimization
- UTXO cache reduces disk I/O
- Parallel transaction validation with Rayon
- Fixed-point arithmetic avoids floating-point errors
- Batch block verification

## Security Properties

### Consensus
- SHA256 PoW (standard Bitcoin-style)
- Selfish mining resistant
- 51% attack requires network majority

### Cryptography
- secp256k1 ECDSA (same as Bitcoin)
- HD wallets with BIP-39/BIP-32
- Deterministic signing

### Network
- Peer authentication via handshakes
- Message validation and rate limiting
- Eclipse attack mitigation

## Development Workflow

### Adding a New CLI Tool
1. Create `src/bin/trinity-mynew.rs`
2. Add `[[bin]]` section to `Cargo.toml`
3. Use existing modules from `src/lib.rs`
4. Test: `cargo build --release` / `cargo run --release --bin trinity-mynew`

### Adding a New Module
1. Create `src/mymodule.rs`
2. Add `pub mod mymodule;` to `src/lib.rs`
3. Document in module docstring (module-level docs)
4. Consider feature-gating if optional

### Enabling Optional Features
1. Add dependency with `optional = true` to `Cargo.toml`
2. Add to feature vector: `myfeature = ["dependency"]`
3. Guard imports with `#[cfg(feature = "myfeature")]`
4. Set `required-features` on dependent binaries

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test '*'
```

### Feature Testing
```bash
cargo test --features api
cargo test --features all
```

## Build Profiles

- **Debug** (`cargo build`): Fast compilation, slow execution, debug symbols
- **Release** (`cargo build --release`): Slow compilation, fast execution, optimized

## CI/CD Integration

The project is designed for:
- ✅ GitHub Actions (automated builds and tests)
- ✅ Docker deployment (containerized nodes)
- ✅ Cloud platforms (stateless REST API with feature flag)

## Future Enhancements

- [ ] Lightning network channels
- [ ] Taproot-style script contracts
- [ ] Zero-knowledge proofs for privacy
- [ ] Sharding for horizontal scaling
- [ ] WebAssembly UTXO validator

## References

- [Cargo Book - Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [BIP-39: Mnemonic Code for Generating Deterministic Keys](https://github.com/trezor/python-mnemonic/blob/master/vectors.txt)
- [BIP-32: Hierarchical Deterministic Wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [Bitcoin Yellow Paper - Simplified](https://bitcoin.org/en/developer-guide)
