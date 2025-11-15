# TrinityChain

> A revolutionary blockchain using Sierpinski triangles as the fundamental unit of value

[![Tests](https://img.shields.io/badge/tests-33%20passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

**TrinityChain** is a novel blockchain implementation that replaces traditional cryptocurrency coins with **fractal geometric shapes** (Sierpinski triangles). Instead of transferring coins, users subdivide and transfer triangular regions, creating a unique economy based on fractal geometry.

## Key Features

- **Fractal-based Value**: Sierpinski triangles as currency (not coins!)
- **Bitcoin-Like Economics**: 420M max supply, halving every 210k blocks, deflationary
- **Real Cryptography**: ECDSA signatures with secp256k1
- **Proof-of-Work**: SHA-256 based mining with Bitcoin-style difficulty adjustment (2,016 block window)
- **UTXO Model**: Bitcoin-style unspent triangle outputs
- **P2P Network**: Fully decentralized node-to-node communication
- **SQLite Persistence**: Lightweight blockchain storage with atomic operations

## How It Works

### Triangle Economy

Traditional blockchains use coins. **TrinityChain** uses geometric triangles:

1. **Genesis Triangle**: The blockchain starts with one large triangle
2. **Subdivision**: Triangles can be split into 3 smaller triangles (75% area conservation)
3. **Transfer**: Triangles can be sent between wallet addresses
4. **Mining Rewards**: Miners receive area-based rewards for solving blocks

Each triangle has:
- 3 coordinate points (vertices)
- A unique hash-based address
- Verifiable geometric properties
- A parent-child relationship (fractal hierarchy)

### Transaction Types

1. **Subdivision**: Split one triangle → 3 children (Sierpinski fractal pattern)
2. **Transfer**: Send triangles from one wallet to another
3. **Coinbase**: Mining rewards (creates new triangle area)

## Quick Start

### Prerequisites

- Rust 1.90+ (stable toolchain)
- SQLite support (bundled via rusqlite)

### Installation

```bash
# Clone the repository
git clone https://github.com/littlekickoffkittie/trinitychain.git
cd trinitychain

# Build the project
cargo build --release

# Run tests (33 tests should pass)
cargo test
```

## Usage

### 1. Create a Wallet

```bash
# Generate a new wallet with keypair
cargo run --bin trinity-wallet-new

# This creates: ~/.TrinityChain/wallet.json
```

Your wallet contains:
- Private key (keep this secret!)
- Public key
- Address (share this to receive triangles)

### 2. Start a Node

```bash
# Start a standalone node on port 8333
cargo run --bin trinity-node 8333

# Connect to another node
cargo run --bin trinity-node 8334 --peer 192.168.1.100:8333
```

### 3. Check Balance

```bash
# View your triangle holdings
cargo run --bin trinity-balance

# Output shows:
# - Triangle hashes you own
# - Area of each triangle
# - Total triangular area
```

### 4. Mine Blocks

```bash
# Get your wallet address first
cargo run --bin trinity-wallet

# Run continuous mining (RECOMMENDED - includes full statistics)
cargo run --release --bin trinity-miner <your_wallet_address>

# Mine a single block
cargo run --bin trinity-mine-block
```

**New Enhanced Miner Output:**
```
╔══════════════════════════════════════════════════════════╗
║                    📊 MINING STATS                       ║
╠══════════════════════════════════════════════════════════╣
║ 🔺 Blocks Mined: 42                                      ║
║ 🏔️  Chain Height: 763                                    ║
║ 🎯 Difficulty: 8                                         ║
║ 💎 Current Reward: 1000 area units                       ║
║ 📈 Total Supply: 763,000 / 420,000,000 (0.182%)         ║
║ ⏰ Blocks to Halving: 209,237                            ║
║ 🎚️  Halving Era: 0                                       ║
╚══════════════════════════════════════════════════════════╝
```

Mining process:
1. Reloads blockchain (stays in sync with network)
2. Collects pending transactions
3. Creates coinbase reward (1000 area, halves every 210k blocks)
4. Computes Merkle root
5. Searches for valid nonce (Proof-of-Work)
6. Broadcasts block to network

### 5. Transfer Triangles

```bash
# Send a triangle to another address
cargo run --bin trinity-send <recipient_address> <triangle_hash>
```

Example:
```bash
cargo run --bin trinity-send \
  1a2b3c4d5e6f7g8h9i0j \
  abc123def456triangle_hash_here
```

## REST API

The blockchain can be queried through a REST API.

**Start the API Server:**
```bash
cargo run --bin trinity-api
```

**Endpoints:**
- `GET /blockchain/height`: Get the current height of the blockchain.
- `GET /blockchain/block/:hash`: Get a block by its hash.
- `GET /address/:addr/balance`: Get the balance for a given address.
- `POST /transaction`: Submit a new transaction.
- `GET /transaction/:hash`: Get the status of a transaction.

## CLI Tools

| Tool | Purpose |
|------|---------|
| `trinity-api` | Runs the REST API server |
| `trinity-wallet-new` | Create a new wallet |
| `trinity-wallet` | Manage existing wallet |
| `trinity-balance` | Check triangle holdings |
| `trinity-send` | Transfer triangles |
| `trinity-mine-block` | Mine a single block |
| `trinity-miner` | Continuous mining daemon |
| `trinity-node` | P2P network node |

## Architecture

```
TrinityChain/
├── src/
│   ├── lib.rs              # Module exports
│   ├── geometry.rs         # Sierpinski triangle math
│   ├── blockchain.rs       # Blockchain & UTXO state
│   ├── transaction.rs      # Transaction types & validation
│   ├── crypto.rs           # ECDSA cryptography
│   ├── miner.rs            # Proof-of-Work mining
│   ├── persistence.rs      # SQLite database
│   ├── network.rs          # P2P networking (Tokio)
│   ├── error.rs            # Error handling
│   └── bin/                # CLI tools (7 binaries)
├── Cargo.toml
└── TrinityChain.db         # Blockchain database
```

## Database Schema

The blockchain uses SQLite with 3 tables:

**blocks**
- `height`: Block number
- `hash`: Block hash (SHA-256)
- `data`: Serialized block (bincode)

**utxo_set**
- `triangle_hash`: Unique triangle identifier
- `triangle_data`: Serialized triangle (bincode)

**metadata**
- `key`: Config key (e.g., "difficulty")
- `value`: Config value

## Network Protocol

Nodes communicate via TCP with bincode serialization:

```rust
// Message types
enum NetworkMessage {
    Ping,
    Pong,
    GetBlockchain,
    Blockchain(Vec<Block>),
    NewBlock(Block),
    NewTransaction(Transaction),
}
```

**Port**: Default 8333 (configurable)
**Protocol**: TCP with async I/O (Tokio)

## Consensus Rules

### Proof-of-Work
- Algorithm: SHA-256 double hash
- Difficulty: Leading zero bits (dynamically adjusted)
- Target block time: 60 seconds (1 minute)
- Adjustment window: 2,016 blocks (~1.4 days, Bitcoin-style)
- Adjustment range: 0.25x to 4x per period

### Supply & Economics (NEW! 🎯)
- **Max Supply**: 420,000,000 area units (20× Bitcoin's 21M)
- **Initial Reward**: 1,000 area units per block
- **Halving**: Every 210,000 blocks (~4 years at 1 min/block)
- **Halvings**: 64 total (after which reward = 0)
- **Supply Formula**: 1000 × 210,000 × 2 = 420M
- See [BITCOIN_FEATURES.md](BITCOIN_FEATURES.md) for full economics documentation

### Block Validation
1. Valid block hash (meets difficulty)
2. Correct previous block hash linkage
3. Valid Merkle root
4. All transactions valid
5. No double-spends

### Transaction Validation
1. Valid ECDSA signature
2. Parent triangle exists in UTXO set
3. Geometric properties correct (area conservation)
4. Children match subdivision rules

## Configuration

Environment variables:

```bash
# Wallet location
HOME=~/.TrinityChain

# Optional: AI validation (experimental) - leave blank if not using
# DEEPSEEK_API_KEY=sk-your-key-here
# GEMINI_API_KEY=your-gemini-key-here
```

## Development Status

**Current Version**: 0.1.0 (Alpha)

**Implemented**:
- ✅ Core blockchain (blocks, chain, validation)
- ✅ Sierpinski triangle geometry
- ✅ 3 transaction types (subdivision, transfer, coinbase)
- ✅ ECDSA cryptography (secp256k1)
- ✅ Proof-of-Work mining
- ✅ P2P networking
- ✅ SQLite persistence
- ✅ 7 CLI tools
- ✅ 33 passing tests

**Coming Soon**:
- 🔜 Transaction mempool
- 🔜 Wallet encryption
- 🔜 Block explorer UI
- 🔜 REST API server
- 🔜 Multi-signature support
- 🔜 Mining optimizations

See [PROJECT_STATUS.md](PROJECT_STATUS.md) for the full roadmap (200+ planned features).

## Examples

### Example: Mine and Transfer

```bash
# 1. Create two wallets
cargo run --bin trinity-wallet-new
# Save address as ALICE_ADDR

cargo run --bin trinity-wallet-new
# Save address as BOB_ADDR

# 2. Mine blocks (Alice's wallet is active)
cargo run --bin trinity-mine-block

# 3. Check Alice's balance
cargo run --bin trinity-balance
# Note a triangle hash: TRIANGLE_HASH

# 4. Send triangle to Bob
cargo run --bin trinity-send $BOB_ADDR $TRIANGLE_HASH

# 5. Mine another block to confirm
cargo run --bin trinity-mine-block

# 6. Bob checks balance (switch wallet first)
cargo run --bin trinity-wallet # Use Bob's wallet
cargo run --bin trinity-balance
# Bob now owns the triangle!
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test geometry
cargo test blockchain
cargo test transaction

# Run with output
cargo test -- --nocapture
```

**Test Coverage**:
- Geometry: 7 tests (subdivision, area, hashing)
- Blockchain: 10 tests (blocks, Merkle trees, validation)
- Transactions: 6 tests (signing, validation, double-spend)
- Cryptography: 5 tests (keys, signatures, addresses)
- Persistence: 2 tests (save/load)
- Network: 3 tests (P2P communication)

## Security Considerations

**Production NOT Ready**: This is an experimental alpha release.

**Known Limitations**:
- Wallet keys stored in plaintext JSON
- No HD wallet support
- No multi-signature support
- Single-threaded mining (slow)
- Limited P2P security
- No transaction fees yet

**Before Production**:
- Implement wallet encryption
- Add transaction fees
- Security audit required
- Peer-to-peer authentication
- Rate limiting on API endpoints

## Performance

**Benchmarks** (on average hardware):

| Operation | Time |
|-----------|------|
| Triangle subdivision | ~1 μs |
| Block validation | ~500 μs |
| Transaction signing | ~200 μs |
| Signature verification | ~300 μs |
| Mining (difficulty 2) | ~1-5 seconds |
| Block propagation | ~100 ms |

**Blockchain Metrics**:
- Block size: ~2-10 KB
- Max transactions per block: Unlimited (currently)
- Database size growth: ~100 KB per 100 blocks
- UTXO set size: ~1 KB per 100 triangles

## Contributing

Contributions welcome! Areas needing help:

1. **Core Features**: Implement mempool, wallet encryption
2. **Performance**: Optimize mining, parallel transaction validation
3. **UI/UX**: Build web-based block explorer
4. **Documentation**: API docs, tutorials, videos
5. **Testing**: Add integration tests, fuzzing
6. **Security**: Audit crypto implementation

## Roadmap

**Phase 1 (Current)**: Core blockchain operational
- [x] Blocks, transactions, mining
- [x] P2P networking
- [x] CLI tools

**Phase 2 (Q1 2026)**: Enhanced functionality
- [ ] Transaction mempool
- [ ] REST API server
- [ ] Web-based block explorer
- [ ] Wallet encryption

**Phase 3 (Q2 2026)**: Ecosystem growth
- [ ] Mobile wallet app
- [ ] Mining pools
- [ ] Smart contracts framework
- [ ] Token standards

**Phase 4 (Q3+ 2026)**: Advanced features
- [ ] Sharding for scalability
- [ ] Cross-chain bridges
- [ ] Privacy features (zk-SNARKs)
- [ ] DAO governance

See [PROJECT_STATUS.md](PROJECT_STATUS.md) for the complete feature list.

## Technical Details

### Sierpinski Triangle Subdivision

When a triangle is subdivided:

```
Original triangle vertices: A, B, C

Midpoints:
  AB = (A + B) / 2
  BC = (B + C) / 2
  AC = (A + C) / 2

Three children:
  Child 1: A, AB, AC
  Child 2: AB, B, BC
  Child 3: AC, BC, C

Area conservation: 3 * child_area = 0.75 * parent_area
```

### Hash-Based Addressing

Each triangle has a canonical hash:

```rust
hash = SHA256(point_a || point_b || point_c || parent_hash)
```

This creates a hierarchical address space where:
- Genesis triangle: No parent
- Child triangles: Include parent hash
- Hash uniqueness: Guaranteed by coordinate ordering

### Difficulty Adjustment

Every 10 blocks, difficulty adjusts:

```rust
time_taken = block[N].timestamp - block[N-10].timestamp
expected_time = 10 blocks * 60 seconds = 600 seconds

if time_taken < 600s → increase difficulty
if time_taken > 600s → decrease difficulty
else → maintain difficulty
```

## FAQ

**Q: Why triangles instead of coins?**
A: Triangles create a geometric economy with unique properties: fractal subdivision, area-based value, visual representation, and novel DeFi primitives.

**Q: Can triangles be merged back together?**
A: Not in v0.1.0, but it's on the roadmap. Merging would reverse subdivision (3 children → 1 parent).

**Q: What's the maximum supply?**
A: **420,000,000 area units** (capped). Mining rewards halve every 210,000 blocks (like Bitcoin), reaching zero after 64 halvings. Subdivision preserves 75% of area, creating additional deflationary pressure.

**Q: Is this faster than Bitcoin?**
A: Block time is 60s (vs Bitcoin's 10min), but it's not optimized for speed yet. Sharding is planned for Phase 4.

**Q: Can I mine with a GPU?**
A: Not yet. GPU mining kernels are on the roadmap (Phase 2).

**Q: How do I backup my wallet?**
A: Copy `~/.TrinityChain/wallet.json` to a secure location. Wallet encryption coming soon.

## Resources

- **Documentation**: [PROJECT_STATUS.md](PROJECT_STATUS.md)
- **Source Code**: [GitHub](https://github.com/littlekickoffkittie/trinitychain)
- **Issue Tracker**: [GitHub Issues](https://github.com/littlekickoffkittie/trinitychain/issues)
- **Sierpinski Triangle**: [Wikipedia](https://en.wikipedia.org/wiki/Sierpi%C5%84ski_triangle)

## License

MIT License - see [LICENSE](LICENSE) for details

## Credits

Built with Rust 🦀

**Key Dependencies**:
- `secp256k1` - Cryptographic signatures
- `sha2` - Hashing
- `rusqlite` - Database
- `tokio` - Async runtime
- `serde` - Serialization

## Support

Found a bug? Have a question?

- Open an [issue](https://github.com/littlekickoffkittie/trinitychain/issues)
- Read the [docs](PROJECT_STATUS.md)
- Check existing [tests](src/) for examples

---

**Built with fractals, secured with cryptography, powered by Rust.** 🔺⛓️

*TrinityChain - Where geometry meets blockchain*
