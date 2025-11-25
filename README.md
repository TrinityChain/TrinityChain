<p align="center">
  <img src="assets/logo.png" alt="TrinityChain Logo" width="100"/>
</p>
<p align="center">
  <img src="assets/TrinityChain.svg" alt="TrinityChain Title" width="400"/>
</p>

<p align="center">
  <strong>Geometric Proof-of-Work Blockchain</strong>
</p>

<p align="center">
  <img alt="version" src="https://img.shields.io/badge/version-0.5.0-blue" />
  <img alt="build" src="https://img.shields.io/badge/build-passing-brightgreen" />
  <img alt="tests" src="https://img.shields.io/badge/tests-82%20passed-brightgreen" />
  <img alt="license" src="https://img.shields.io/badge/license-MIT-blue" />
  <img alt="rust" src="https://img.shields.io/badge/rust-1.70+-orange" />
</p>

---

TrinityChain is a proof-of-work blockchain where value exists as geometric triangles. Each UTXO is a triangle with three coordinates in 2D space—the area of that triangle is its value. This is a working blockchain with nodes, miners, wallets, and a complete transaction system.

The protocol supports three transaction types: coinbase transactions that mint new triangles as mining rewards, transfer transactions that move triangle ownership between addresses, and subdivision transactions that split triangles into three smaller pieces. Consensus is achieved through SHA-256 proof-of-work with dynamic difficulty adjustment.

Last test run: `2025-11-25 01:00:54Z` (UTC) — 82 tests passing.

---

## Installation

### Requirements

- Rust 1.70+
- SQLite3

### Build

```bash
git clone https://github.com/TrinityChain/TrinityChain.git
cd TrinityChain
cargo build --release
cargo test --release
```

Binaries are in `target/release/`.

---

## Quick Start

The fastest way to get a node and miner running:

```bash
chmod +x scripts/quickstart.sh
source "$HOME/.cargo/env"
./scripts/quickstart.sh
```

This will build everything, run tests, create a wallet, start a headless node (P2P server), and launch the headless miner. The quickstart miner runs the headless miner in the foreground and prints mined-block messages; to run the interactive miner dashboard use `trinity-miner` instead.

To run the miner in the background:

```bash
./scripts/quickstart.sh --detach-miner
tail -f logs/node.log logs/miner.log
```

### Manual Start

```bash
# Create a wallet
target/release/trinity-wallet-new mywalletname

# Start the node (headless). The node runs a P2P server on port 9090 by default.
nohup target/release/trinity-headless-node 9090 > logs/node.log 2>&1 &

# Start mining (interactive dashboard)
target/release/trinity-miner <your_address>

# Or run the headless miner (non-TUI) which mines N blocks and exits, e.g.:
target/release/trinity-headless-miner <your_address> --blocks 1
```

Your address is in `~/.trinitychain/wallet_mywalletname.json`.

---

## How Value Works

### Triangle UTXOs

Every spendable unit on TrinityChain is a triangle:

```rust
struct Triangle {
    vertices: [(f64, f64); 3],  // three points
    owner: Address,              // public key hash
    value: f64                   // geometric area
}
```

The area is computed with the Shoelace formula. That area is what you spend.

### Transactions

**Coinbase** — Miners create a new triangle when they mine a block. Current reward is 1,000 TRC, halving every 210,000 blocks.

**Transfer** — Send a triangle to another address. The coordinates stay the same, but ownership changes. Fees are paid by reducing the output triangle's area.

**Subdivision** — Split one triangle into three smaller ones using Sierpiński geometry:

```
    Parent                 After Split
      /\                      /\   /\
     /  \        →           /__\_/__\
    /____\                  \  / \  / /
                             \/___\/\/
```

The three children sum to the parent's area, minus any fee.

### Validation

Nodes validate transactions by checking:
- Digital signatures match the triangle owner
- Triangles exist and haven't been spent
- Area is conserved in subdivisions
- No double-spends

### Proof-of-Work

Miners solve SHA-256 puzzles to add blocks. Difficulty adjusts every 2,016 blocks (the difficulty window is 2,016 blocks). The target block time is 60 seconds (1 minute). The chain with the most accumulated work is the valid chain.

---

## Running a Node

### Interactive Mode

```bash
target/release/trinity-node [port]
```

Terminal interface with live stats. Press `q` to quit.

### Headless Mode

```bash
target/release/trinity-headless-node 9090
```

Runs in the background, logs to stdout.

### Adding Peers

```bash
target/release/trinity-headless-node 9090 --peer 192.168.1.50:9090 --peer node.example.com:9090
```

The node discovers peers through DNS seeds automatically, but you can add specific nodes if needed.

---

## Mining

### Interactive Miner

```bash
target/release/trinity-miner <your_address>
```

Shows hashrate, blocks found, and network stats. Press `q` to quit.

### Headless Miner

```bash
target/release/trinity-headless-miner <your_address>
```

Runs in the background for server deployments.

---

## Wallet Management

### Create Wallet

```bash
target/release/trinity-wallet-new walletname
```

Generates a new wallet with BIP-39 mnemonic. Wallet file goes to `~/.trinitychain/`.

### Restore Wallet

```bash
target/release/trinity-wallet-restore
```

Restore from your mnemonic phrase.

### Backup Wallet

```bash
target/release/trinity-wallet-backup
```

Export wallet data for safekeeping.

---

## Sending Transactions

```bash
target/release/trinity-send <recipient_address> <triangle_hash> [memo]
```

This creates, signs, and broadcasts the transaction.

---

## HTTP API

The node runs an HTTP server on port 3000 (set `PORT` env var to change).

### Blockchain Data

```bash
# Current chain state
curl http://localhost:3000/api/blockchain/stats

# Recent blocks
curl http://localhost:3000/api/blockchain/blocks

# Specific block
curl http://localhost:3000/api/blockchain/block/<hash>
```

### Address Data

```bash
# Check balance
curl http://localhost:3000/api/address/<address>/balance

# List triangles owned
curl http://localhost:3000/api/address/<address>/triangles
```

### Submit Transaction

```bash
curl -X POST http://localhost:3000/api/transaction \
  -H "Content-Type: application/json" \
  -d '{"transaction": "<hex_tx>"}'
```

### Mining Control

```bash
# Start mining
curl -X POST http://localhost:3000/api/mining/start

# Stop mining
curl -X POST http://localhost:3000/api/mining/stop

# Check status
curl http://localhost:3000/api/mining/status
```

### Network

```bash
# List connected peers
curl http://localhost:3000/api/network/peers

# WebSocket for live updates
ws://localhost:3000/ws/p2p
```

---

## Data Storage

All blockchain data is stored in `trinitychain.db` (SQLite). This includes:
- All blocks
- All transactions
- Current UTXO set
- Chain metadata

Wallet files are in `~/.trinitychain/`.

### Backups

```bash
# Backup the blockchain
cp trinitychain.db trinitychain.db.backup

# Backup wallets
cp -r ~/.trinitychain ~/.trinitychain.backup
```

---

## Network

Nodes communicate over TCP on port 9090. They discover peers automatically through DNS seeds and share blocks and transactions with connected peers.

If you're behind a firewall:

```bash
target/release/trinity-headless-node 9090 --peer <known_node_ip>:9090
```

---

## Protocol Specifications

| Parameter | Value |
|-----------|-------|
| Block reward (initial) | 1,000 TRC |
| Halving interval | 210,000 blocks |
| Target block time | 60 seconds |
| Difficulty adjustment | Every 2,016 blocks |
| Max supply | ~420,000,000 TRC |
| PoW hash function | SHA-256 |
| Signature algorithm | secp256k1 ECDSA |
| Key derivation | BIP-39/BIP-32 |

---

## Codebase

```
src/
├── geometry.rs       # Triangle math and area calculation
├── transaction.rs    # Transaction types and validation
├── blockchain.rs     # UTXO set, mempool, consensus rules
├── network.rs        # P2P networking
├── miner.rs          # Proof-of-work mining
├── persistence.rs    # SQLite operations
├── api.rs            # HTTP and WebSocket server
├── crypto.rs         # secp256k1 wrappers
├── wallet.rs         # Wallet operations
├── hdwallet.rs       # HD wallet derivation
└── bin/              # CLI programs
```

---

## Testing

```bash
# Run all tests
cargo test --release

# Test specific module
cargo test --release --lib blockchain
cargo test --release --lib geometry
```

Tests cover geometry calculations, transaction validation, UTXO management, signature verification, block validation, and difficulty adjustment.

---

## Troubleshooting

### Build Issues

```bash
rustup update stable
cargo clean
cargo build --release
```

### Peer Discovery

If the node can't find peers:
- Check that port 9090 is open in your firewall
- Use `--peer` to manually add nodes
- Verify DNS is working

### TUI Problems

The terminal dashboards need:
- A real terminal (not log redirection)
- ANSI color support
- Minimum 80x24 size

### Database Issues

Check integrity:
```bash
sqlite3 trinitychain.db "PRAGMA integrity_check;"
```

---

## Documentation

- `ARCHITECTURE_MOC.md` — System architecture
- `ARCHITECTURE_AUDIT.md` — Data flow
- `SAFETY_AUDIT.md` — Concurrency and error handling
- `TRIANGLE_UTXO_AUDIT.md` — Geometric model details
- `API_ENDPOINTS.md` — Complete API docs
- `NODE_SETUP.md` — Deployment guide

---

## Contributing

Fork the repo, make your changes, add tests, and submit a pull request.

Use `cargo fmt` and `cargo clippy`. Write tests for new features. Follow Rust conventions.

---

## License

MIT — see `LICENSE`.

---

## Links

**Repository:** https://github.com/TrinityChain/TrinityChain  
**Issues:** https://github.com/TrinityChain/TrinityChain/issues

---

<p align="center">
Rust • SQLite • secp256k1 • SHA-256
</p>