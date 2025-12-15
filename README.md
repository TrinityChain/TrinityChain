<div align="center">

<img src="documentation/assets/logo.png" alt="TrinityChain Logo" width="200"/>

![Trinity Chain](documentation/assets/text.png)

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust 1.70+](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

**A blockchain where value is geometric area**

[Features](#features) • [Quick Start](#quick-start) • [Architecture](#architecture) • [API](#api) • [Contributing](#contributing)

</div>

---

## Overview

TrinityChain implements a proof-of-work blockchain where every UTXO is a triangle with real coordinates. The spendable value of each UTXO equals its geometric area, calculated using the Shoelace formula.

```
        △
       /│\
      / │ \     Area = 100.0 TRC
     /  │  \    
    /___│___\   
        │
        ↓
   Subdivide (Sierpiński)
        │
    ┌───┴───┐
    △   △   △
   
   33.3 + 33.3 + 33.3 = 100.0 TRC
```

**Current Status:** Functional CLI implementation with full mining, wallet, and transaction support.
This project is actively maintained as a command-line blockchain interface.

---

## Features

### Triangle-Based UTXO Model

| Operation | Input | Output | Mechanism |
|-----------|-------|--------|-----------|
| **Coinbase** | `∅` | `1△` | Mining creates new triangle |
| **Transfer** | `1△` | `1△` | Ownership change with geometric fee |
| **Subdivision** | `1△` | `3△` | Sierpiński fractal split |

### Geometric Fee Structure

Fees are deducted by reducing a triangle's area while preserving its coordinate identity:

```rust
// Fee: 0.1% of triangle area
Triangle { area: 100.0, vertices: [(0,0), (10,0), (5,10)] }
  ↓ [fee deduction]
Triangle { area: 99.9,  vertices: [(0,0), (10,0), (5,10)] }
```

### Technical Stack

| Component | Implementation |
|-----------|----------------|
| Consensus | SHA-256 PoW + Difficulty |
| Crypto | secp256k1 + BIP-39/BIP-32 |
| Storage | SQLite (deterministic) |
| Networking | TCP P2P + WebSocket bridge |
| Precision | I32F32 fixed-point (consensus) |

---

## Quick Start

### Prerequisites

```bash
# Install Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify
rustc --version
```

### Build

```bash
git clone https://github.com/TrinityChain/TrinityChain.git
cd TrinityChain

cargo build --release
cargo test --lib
```

### Run Tests

```bash
# Run all tests
cargo test

# Run wallet and transaction tests
cargo test --test wallet_and_transactions

# Run with output
cargo test -- --nocapture
```

**Test Coverage:**
- Wallet creation and persistence
- Transaction creation and validation
- Blockchain initialization
- Multi-wallet isolation
- Fee calculations
- Alice-to-Bob transfer demo

### Create Wallet

```bash
# Generate new wallet
./target/release/trinity-wallet new

# Show wallet address
./target/release/trinity-wallet address

# List all wallets
./target/release/trinity-wallet list

# Output example:
# Address: e54369c2ef44435ba34ef6ee881f33b2fa3126c0bf0e96a806ddc39ab91c046d
# Created: 2025-12-15T17:03:28.761074746+00:00
```

### CLI Commands

#### Node Operations

```bash
# Start a network node with TUI interface
./target/release/trinity-node

# View node help
./target/release/help
```

#### Wallet Management

```bash
# Create new wallet
cargo run --bin trinity-wallet -- new

# Get wallet address
cargo run --bin trinity-wallet -- address

# List all wallets
cargo run --bin trinity-wallet -- list

# Backup wallet
cargo run --bin trinity-wallet -- backup <wallet_name>

# Restore wallet
cargo run --bin trinity-wallet -- restore <backup_file>
```

#### Mining

```bash
# Start persistent miner
cargo run --bin trinity-miner

# Mine a single block
cargo run --bin trinity-mine-block

# Check mining status
cargo run --bin trinity-node
```

#### Transactions

```bash
# Send transaction
cargo run --bin trinity-send -- <recipient_address> <amount> --from <wallet_name>

# View transaction history
cargo run --bin trinity-history -- <address>

# Check address balance
cargo run --bin trinity-balance -- <address>
```

#### Other Utilities

```bash
# View address book
cargo run --bin trinity-addressbook

# Connect to peer
cargo run --bin trinity-connect -- <peer_address:port>

# Run Telegram bot
cargo run --bin trinity-telegram-bot
```

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                      TrinityChain Node                    │
├──────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────┐    ┌──────────────┐    ┌────────────┐ │
│  │   Wallet    │───▶│  Mempool     │───▶│   Miner    │ │
│  │  (secp256k1)│    │ (Pending TX) │    │ (SHA-256)  │ │
│  └─────────────┘    └──────────────┘    └────────────┘ │
│         │                   │                    │       │
│         │                   ▼                    ▼       │
│         │            ┌──────────────┐    ┌────────────┐ │
│         └───────────▶│  Blockchain  │◀───│   Block    │ │
│                      │  (UTXO Set)  │    │  (Header)  │ │
│                      └──────────────┘    └────────────┘ │
│                             │                    │       │
│                             ▼                    ▼       │
│                      ┌──────────────────────────────┐   │
│                      │   Persistence (SQLite)       │   │
│                      └──────────────────────────────┘   │
│                                                           │
├──────────────────────────────────────────────────────────┤
│  Network Layer: TCP P2P ◀──▶ WebSocket Bridge           │
├──────────────────────────────────────────────────────────┤
│  API Layer: REST Endpoints + WebSocket Subscriptions    │
└──────────────────────────────────────────────────────────┘
```

### Core Modules

```
src/
├── geometry.rs       # Triangle primitives, Shoelace area calculation
├── transaction.rs    # Coinbase, Transfer, Subdivision logic
├── blockchain.rs     # Chain validation, UTXO management, mempool
├── network.rs        # P2P message handling, peer discovery
├── miner.rs          # PoW mining, difficulty adjustment (10 blocks)
├── persistence.rs    # SQLite schema and queries
├── api.rs            # REST + WebSocket endpoints
├── crypto.rs         # secp256k1 signing and verification
├── wallet.rs         # UTXO selection, transaction building
└── hdwallet.rs       # BIP-39 mnemonic, BIP-32 derivation
```

### Concurrency Model

| Component | Synchronization | Reason |
|-----------|----------------|---------|
| P2P Network | `Arc<RwLock<T>>` | Multi-reader broadcast |
| API State | `Arc<Mutex<T>>` | Single-writer safety |
| Mining | `AtomicBool + AtomicU64` | Lock-free cancellation |

---

## Command Line Interface

### Primary Interface: CLI Tools

TrinityChain is designed as a command-line-first blockchain. All functionality is accessible through dedicated CLI binaries:

| Binary | Purpose | Example |
|--------|---------|---------|
| `trinity-wallet` | Wallet creation and management | `cargo run --bin trinity-wallet -- new` |
| `trinity-send` | Send transactions | `cargo run --bin trinity-send -- <addr> <amount>` |
| `trinity-balance` | Check address balance | `cargo run --bin trinity-balance -- <address>` |
| `trinity-node` | Run blockchain node (TUI) | `cargo run --bin trinity-node` |
| `trinity-miner` | Persistent background miner | `cargo run --bin trinity-miner` |
| `trinity-mine-block` | Mine a single block | `cargo run --bin trinity-mine-block` |
| `trinity-history` | Transaction history | `cargo run --bin trinity-history -- <address>` |
| `trinity-connect` | Connect to peer nodes | `cargo run --bin trinity-connect -- <addr>` |
| `trinity-addressbook` | Manage address book | `cargo run --bin trinity-addressbook` |
| `trinity-telegram-bot` | Telegram bot interface | `cargo run --bin trinity-telegram-bot` |

### Terminal User Interface

The `trinity-node` command provides an interactive TUI with:
- Real-time blockchain stats
- Peer connection status
- Mining controls
- Transaction visualization

```bash
cargo run --bin trinity-node
```

### REST API (Optional)

For programmatic access, an optional REST API server is available:

```bash
# Start API server (separate from CLI tools)
cargo run --bin trinity-api
```

**Note:** The REST API is provided for integration and development purposes. The CLI is the primary and recommended interface.

---

## Tokenomics

| Parameter | Value |
|-----------|-------|
| Initial Reward | 50 TRC + fees |
| Halving Interval | 210,000 blocks |
| Max Supply | 420,000,000 TRC |
| Block Time (target) | ~30 seconds |
| Current Difficulty | Dynamic (10-block avg) |

Supply curve follows geometric decay with periodic halvings.

---

## Command Line Tools

| Binary | Purpose |
|--------|---------|
| `trinity-wallet` | Wallet creation and management |
| `trinity-send` | Send transactions between addresses |
| `trinity-balance` | Check address balance |
| `trinity-node` | Run a blockchain node |
| `trinity-miner` | Persistent background miner |
| `trinity-mine-block` | Mine a single block |
| `trinity-api` | Start REST API server |
| `trinity-server` | Start API server with P2P networking |
| `trinity-history` | View transaction history |
| `trinity-connect` | Connect to peer nodes |
| `trinity-addressbook` | Manage address book |
| `trinity-user` | Manage user profiles |

---

## Documentation

| Document | Description |
|----------|-------------|
| [`ARCHITECTURE_MOC.md`](documentation/ARCHITECTURE_MOC.md) | Component map with ASCII diagrams |
| [`ARCHITECTURE_AUDIT.md`](documentation/ARCHITECTURE_AUDIT.md) | Data flow and module interactions |
| [`SAFETY_AUDIT.md`](documentation/SAFETY_AUDIT.md) | Concurrency and error handling review |
| [`TRIANGLE_UTXO_AUDIT.md`](documentation/TRIANGLE_UTXO_AUDIT.md) | UTXO model deep dive |
| [`API_ENDPOINTS.md`](documentation/API_ENDPOINTS.md) | Complete API reference |
| [`NODE_SETUP.md`](documentation/NODE_SETUP.md) | Production deployment guide |

---

## Deployment

Configurations for cloud deployment:

```
deployment/
├── render.yaml       # Render.com service config
└── vercel.json       # Vercel frontend config
```

See [`NODE_SETUP.md`](documentation/NODE_SETUP.md) for production deployment instructions.

---

## Contributing

This project needs developers, reviewers, and testers. Contributions are welcome.

### How to Contribute

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/TrinityChain.git

# 2. Create feature branch
git checkout -b feature/your-feature

# 3. Make changes and test
cargo test --lib

# 4. Submit PR
git push origin feature/your-feature
```

### Areas Needing Work

- Security audits (cryptography, consensus)
- Performance optimization (mining, validation)
- Network protocol improvements
- Documentation and tutorials
- Test coverage expansion

Look for issues tagged `good first issue` or propose your own improvements.

### Code Standards

- Run `cargo fmt` and `cargo clippy` before committing
- Add tests for new functionality
- Document public APIs with rustdoc comments
- Follow existing module structure

### Testing

We use integration tests to verify wallet and transaction functionality:

```bash
# Run the wallet and transaction test suite
cargo test --test wallet_and_transactions --verbose

# Expected output: 10 tests covering:
# ✓ Wallet creation
# ✓ Two-wallet scenarios (Alice & Bob)
# ✓ Wallet persistence (save/load)
# ✓ Keypair derivation
# ✓ Blockchain initialization
# ✓ Transfer transactions
# ✓ Multi-wallet isolation
# ✓ Fee calculations
```

---

## Project Structure

```
.
├── .github/          # CI/CD workflows
├── dashboard/        # React frontend
├── deployment/       # Cloud configs (Render, Vercel)
├── documentation/    # Architecture docs
├── scripts/          # Build and deployment scripts
├── src/              # Rust blockchain implementation
├── tests/            # Integration tests
│   └── wallet_and_transactions.rs  # Wallet & transaction tests
├── Cargo.toml
├── LICENSE
└── README.md
```

---

## Links

- **Repository:** https://github.com/TrinityChain/TrinityChain
- **Issues:** https://github.com/TrinityChain/TrinityChain/issues
- **License:** [MIT](LICENSE)

---

<div align="center">

**Built with Rust**

</div>
