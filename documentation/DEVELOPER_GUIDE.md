# Developer Onboarding Guide

Welcome to TrinityChain! This guide will get you productive in 15 minutes.

## 1. Environment Setup (5 minutes)

### Install Rust
```bash
# macOS / Linux / WSL
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Clone and Build
```bash
git clone https://github.com/your-fork/TrinityChain.git
cd TrinityChain

# Build with default features (CLI only)
cargo build --release

# Build with all features (includes REST API + Telegram)
cargo build --release --all-features

# First build takes ~2-3 minutes, subsequent builds are faster
```

## 2. Project Structure (5 minutes)

### High-Level Layout
```
TrinityChain/
├── src/
│   ├── lib.rs              # Main library exports (8 module groups)
│   ├── blockchain.rs       # Core chain logic
│   ├── wallet.rs           # UTXO management
│   ├── crypto.rs           # Signing/hashing
│   ├── geometry.rs         # Triangle math
│   └── bin/                # 20+ CLI tools
│       ├── trinity-wallet.rs
│       ├── trinity-miner.rs
│       ├── trinity-send.rs
│       └── ...
├── Cargo.toml              # Dependencies + feature flags
├── config.toml             # Node configuration
├── tests/                  # Integration tests
├── documentation/          # Detailed guides
│   ├── ARCHITECTURE.md     # System design (500+ lines)
│   ├── MODULE_GUIDE.md     # Module reference (600+ lines)
│   ├── CLI_REFERENCE.md    # CLI tools (all 20+)
│   └── ...
└── dashboard/              # (Deprecated) Web UI - use CLI instead
```

### Quick Navigation
```bash
# Find what you need
cargo tree --depth 2                    # Dependency graph
cargo doc --open                        # API documentation
grep -r "pub fn" src/ | grep wallet     # Find wallet functions
```

## 3. Key Concepts (5 minutes)

### The Triangle Model
```rust
// Every UTXO is a triangle
Triangle {
    p1: Point { x: 0.0,  y: 0.0  },   // Vertex 1
    p2: Point { x: 10.0, y: 0.0  },   // Vertex 2
    p3: Point { x: 5.0,  y: 10.0 },   // Vertex 3
}

// Spendable value = Geometric area (100.0 TRC)
area = |10 × 10| / 2 = 50.0... 
       // (Shoelace formula)
```

### Coordinate System
- Coordinates are **I32F32 fixed-point** (no floating-point errors)
- Range: -2 billion to +2 billion
- Precision: 2^-32 (~0.0000000002)
- Why? Consensus-critical → deterministic calculations

### Key Operations
1. **Mining**: Miner creates new block with rewards (coinbase transaction)
2. **Transfer**: Send triangle to another address (geometric fee deducted)
3. **Subdivision**: Split triangle into 3 smaller triangles (Sierpiński fractal)

## 4. First Task: Run the CLI

### Create a Wallet
```bash
cargo run --release --bin trinity-wallet -- --create my-wallet
# Creates wallet with 12-word mnemonic (BIP-39)
# Stores at: ~/.trinity/wallets/my-wallet.dat
```

### Check Balance
```bash
cargo run --release --bin trinity-balance -- my-wallet
# Shows UTXO count and total area
```

### Mine a Block
```bash
cargo run --release --bin trinity-miner -- my-wallet
# Finds valid PoW nonce
# Awards 50 TRC (triangle) to your wallet
# Takes ~30 seconds on modern hardware
```

### Send Funds
```bash
cargo run --release --bin trinity-send -- \
  --from my-wallet \
  --to alice-address \
  --amount 10.0
# Creates transfer transaction
# Broadcasts to peers
```

## 5. Understanding Module Organization

### 8 Module Groups

| Group | Modules | Purpose | When You Need It |
|-------|---------|---------|-------------------|
| **Core** | blockchain, transaction, mempool | Chain logic | Adding tx types |
| **Geometric** | geometry, fees | Triangle math | Modifying fees |
| **Consensus** | miner | PoW mining | Mining changes |
| **Crypto** | crypto, security | Signing/hashing | Key generation |
| **State** | wallet, hdwallet, persistence, cache | Storage | Persistence logic |
| **Network** | network, discovery, sync | P2P | Node coordination |
| **Integration** | api (optional) | REST API | `--features api` |
| **Config** | config, error, cli, addressbook | Utilities | CLI commands |

### Import Cheat Sheet

**In a CLI binary** (`src/bin/trinity-wallet.rs`):
```rust
use trinitychain::{
    blockchain::Blockchain,          // Access chain
    wallet::Wallet,                  // Create/manage wallet
    transaction::Transaction,        // Build transactions
    persistence::Database,           // Load/save blocks
    config::load_config,            // Get settings
};
```

**In a module** (`src/wallet.rs`):
```rust
use crate::{
    blockchain::Blockchain,         // Note: `crate::` not `trinitychain::`
    crypto::sign,
    error::WalletError,
};
```

**With optional feature**:
```rust
#[cfg(feature = "api")]
use crate::api::run_api_server;
```

## 6. Common Tasks

### Add a New CLI Tool
1. Create `src/bin/trinity-mynew.rs`
2. Write `#[tokio::main] async fn main()`
3. Add to `Cargo.toml`:
   ```toml
   [[bin]]
   name = "trinity-mynew"
   path = "src/bin/trinity-mynew.rs"
   ```
4. Build: `cargo build --release --bin trinity-mynew`

### Add a Module Export
1. Create `src/mymodule.rs`
2. In `src/lib.rs`, add:
   ```rust
   // ============================================================================
   // My New Category
   // ============================================================================
   pub mod mymodule;
   ```
3. In `src/bin/trinity-wallet.rs`, import:
   ```rust
   use trinitychain::mymodule::MyType;
   ```

### Fix a Bug
1. Find the bug: `grep -r "bug_name" src/`
2. Read [MODULE_GUIDE.md](documentation/MODULE_GUIDE.md) for that module
3. Make changes
4. Test: `cargo test`
5. Submit PR

### Run Tests
```bash
cargo test --lib                    # Unit tests
cargo test --test '*'               # Integration tests
cargo test test_mining_reward       # Specific test
RUST_LOG=debug cargo test           # With debugging
```

### Enable Verbose Logging
```bash
RUST_LOG=debug cargo run --release --bin trinity-wallet
RUST_LOG=trinitychain::wallet=trace cargo run --release --bin trinity-wallet
```

## 7. Feature Flags

### Build Without Optional Features
```bash
# CLI only (no REST API)
cargo build --release                  # ✅ Works

# To disable all optional features explicitly
cargo build --release --no-default-features --features cli
```

### Build With API
```bash
cargo build --release --features api   # ✅ Enables REST endpoints
cargo run --release --bin trinity-api  # ✅ Starts API server
```

### Build With Telegram
```bash
cargo build --release --features telegram
cargo run --release --bin trinity-telegram-bot
```

### Build Everything
```bash
cargo build --release --features all
# or
cargo build --release --all-features
```

## 8. Documentation Map

| Document | Purpose | Read When |
|----------|---------|-----------|
| [README.md](../README.md) | Project overview | First time here |
| [ARCHITECTURE.md](documentation/ARCHITECTURE.md) | System design (500+ lines) | Understanding architecture |
| [MODULE_GUIDE.md](documentation/MODULE_GUIDE.md) | Module reference (600+ lines) | Implementing features |
| [CLI_REFERENCE.md](documentation/CLI_REFERENCE.md) | All CLI tools | Running commands |
| [QUICKSTART.md](documentation/QUICKSTART.md) | 3-minute mining guide | First time mining |
| [SECURITY.md](documentation/SECURITY.md) | Security practices | Handling secrets |
| [NODE_SETUP.md](documentation/NODE_SETUP.md) | Running a node | Operating infrastructure |

## 9. Git Workflow

```bash
# 1. Create a branch
git checkout -b feature/my-feature

# 2. Make changes
cargo build --release
cargo test

# 3. Commit
git add .
git commit -m "feat: add my feature"

# 4. Push
git push origin feature/my-feature

# 5. Open PR on GitHub
```

## 10. Debugging Tips

### See What's Happening
```bash
# Enable debug logging
RUST_LOG=debug cargo run --release --bin trinity-wallet

# Trace specific module
RUST_LOG=trinitychain::blockchain=trace cargo run --release --bin trinity-wallet

# Show backtraces on panic
RUST_BACKTRACE=1 cargo run --release --bin trinity-wallet
```

### Check Code Issues
```bash
# Run clippy (linter)
cargo clippy --release

# Check formatting
cargo fmt --check

# Fix formatting
cargo fmt
```

### Find Unused Code
```bash
# Show dead code warnings
cargo build --release 2>&1 | grep "never used"

# Detailed analysis
cargo clippy --release -- -W warnings
```

## 11. Performance Profiling

### Flamegraph (Installation Required)
```bash
# Install profiler
cargo install flamegraph

# Profile a command
cargo flamegraph --bin trinity-miner -- my-wallet

# View: flamegraph.svg
```

### Benchmarking
```bash
# See how long an operation takes
time cargo run --release --bin trinity-miner -- my-wallet
```

## 12. Troubleshooting

| Problem | Solution |
|---------|----------|
| "Failed to open database" | Check `~/.trinity/` permissions |
| "No such file or directory" | Ensure wallet exists: `trinity-wallet --list` |
| Build fails | `cargo clean && cargo build --release` |
| "Feature not available" | Add `--features api` or `--features all` |
| Slow compilation | Only build what you need: `cargo build --release --bin trinity-wallet` |

## 13. Best Practices

✅ **DO:**
- Read the module documentation before editing
- Run `cargo fmt` before committing
- Write tests for new code
- Use feature flags for optional functionality
- Check existing code style and match it

❌ **DON'T:**
- Mix feature-gated code with core logic
- Add dependencies without documenting in Cargo.toml
- Change database schema without migration
- Use `unwrap()` in library code (use `?` or `Result`)
- Ignore compiler warnings

## 14. Quick Commands Reference

```bash
# Development
cargo build               # Debug build (fast compile, slow run)
cargo build --release    # Release build (slow compile, fast run)
cargo run --release --bin trinity-wallet

# Testing
cargo test               # All tests
cargo test --lib        # Library tests only
cargo test -- --nocapture  # Show println! output

# Code Quality
cargo fmt               # Format code
cargo clippy            # Lint suggestions
cargo clippy --fix      # Auto-fix issues
cargo doc --open        # View documentation

# Dependencies
cargo tree              # Dependency graph
cargo outdated          # Check for new versions
cargo update            # Update dependencies
cargo add serde         # Add new dependency

# Profiling
cargo build --release  # Optimized build
cargo bloat --release  # What's using binary size
```

## 15. Get Help

**Questions?**
- 📖 Read [MODULE_GUIDE.md](documentation/MODULE_GUIDE.md)
- 🏗️ Check [ARCHITECTURE.md](documentation/ARCHITECTURE.md)
- 💬 Open an issue on GitHub
- 🤝 Join Discord community

**Want to Contribute?**
- See [CONTRIBUTING.md](documentation/CONTRIBUTING.md)
- Pick an issue labeled `good-first-issue`
- Follow the PR workflow above

---

**Ready to code?** Start with the [QUICKSTART](documentation/QUICKSTART.md) guide! 🚀
