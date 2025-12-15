# 🚀 Quickstart Guide - CLI Mining in 5 Minutes

Welcome to **TrinityChain** - a geometric blockchain you control entirely from the command line!

---

## ⚡ Super Quick Start (3 Commands)

```bash
# 1. Clone and build
git clone https://github.com/TrinityChain/TrinityChain.git
cd TrinityChain && cargo build --release

# 2. Create your wallet
cargo run --release --bin trinity-wallet -- new

# 3. Start mining
cargo run --release --bin trinity-miner
```

**That's it!** You're now mining triangles on TrinityChain. ⛓️🔺

---

## 📋 Detailed CLI Setup

### Prerequisites

**Required:**
- Rust 1.70+ ([install here](https://rustup.rs/))
- SQLite (usually pre-installed)
- 100MB disk space

**Platform Support:**
- ✅ Linux (Ubuntu, Debian, Arch, Fedora)
- ✅ macOS (Intel & Apple Silicon)
- ✅ Windows (WSL2 recommended)
- ✅ Termux (Android)

### Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

### Step 2: Clone & Build

```bash
git clone https://github.com/TrinityChain/TrinityChain.git
cd TrinityChain
cargo build --release
```

### Step 3: Create a Wallet

```bash
# Create new wallet
cargo run --release --bin trinity-wallet -- new

# List your wallets
cargo run --release --bin trinity-wallet -- list

# Get your address
cargo run --release --bin trinity-wallet -- address
```

**Save your wallet address!** You'll need it for mining.

### Step 4: Create Wallet

```bash
cargo run --release --bin trinity-wallet -- new
```

**Output:**
```
🎉 Wallet created successfully!
Save your wallet backup:
```bash
# Backup your wallet
cargo run --release --bin trinity-wallet -- backup <wallet_name>
```

### Step 4: Start Mining

```bash
# Start the miner
cargo run --release --bin trinity-miner
```

**You'll see:**
```
⛏️  Mining block #1...
[████████████████░░░░░] 75% - Attempts: 15,234 - Speed: 1.2 MH/s

✨ BLOCK FOUND!
├─ Height: 1
├─ Difficulty: 2
├─ Hash: 00a3f7b9...8c2d1e4f
├─ Reward: 50 TRC
└─ Time: 2.3s
```

**Congratulations!** 🎉 You've mined your first block!

---

## 🔍 Common CLI Commands

### Check Your Balance

```bash
cargo run --release --bin trinity-balance -- <address>
```

### View Transaction History

```bash
cargo run --release --bin trinity-history -- <address>
```

### Send Triangles

```bash
cargo run --release --bin trinity-send -- <recipient_address> <amount> --from <wallet_name>
```

### Connect to Peers

```bash
cargo run --release --bin trinity-connect -- <peer_address:port>
```

### Run a Node

```bash
cargo run --release --bin trinity-node
```

This starts an interactive TUI showing:
- Real-time blockchain stats
- Peer connections
- Mining activity
- Mempool transactions

### Connect to Another Node (Multi-Player!)

```bash
# Person A (host):
cargo run --release --bin trinity-node --port 8333

# Person B (connect to A):
cargo run --release --bin trinity-node --port 8334 --peer <PERSON_A_IP>:8333

# Now Person B can mine:
cargo run --release --bin trinity-miner --peer <PERSON_A_IP>:8333
```

---

## 🎓 Learn More

### What Makes This Different?

**Traditional Blockchain:**
- Currency = coins
- Transactions = send coins
- Mining = earn coins

**Sierpinski Blockchain:**
- Currency = **triangular areas** (fractal geometry!)
- Transactions = **subdivide** and **transfer** triangles
- Mining = earn **area units**

### The Economics

```
Max Supply: 420,000,000 area units
Initial Reward: 50 per block (plus transaction fees)
Halving: Every 210,000 blocks (~4 years)

Block 0 - 209,999:    50 area/block + fees
Block 210,000 - 419,999:  25 area/block + fees
Block 420,000 - 629,999:  12.5 area/block + fees
...and so on (64 halvings total)
```

This is modeled after Bitcoin! See [BITCOIN_FEATURES.md](BITCOIN_FEATURES.md) for details.

### The Geometry

Each triangle:
- Has 3 vertices (x, y coordinates)
- Has a calculated area
- Can be **subdivided** into 3 smaller triangles (Sierpinski fractal pattern)
- Is tracked in the UTXO set (like Bitcoin's unspent outputs)

---

## 🐛 Troubleshooting

### Build Fails

```bash
# Make sure Rust is up to date:
rustup update

# Try clean rebuild:
cargo clean
cargo build --release
```

### Miner Shows "Failed to apply new block"

This usually means:
- Timestamp issue (blocks too fast)
- Chain needs to sync

**Fix:**
```bash
# Stop miner (Ctrl+C)
# Wait 1-2 seconds
# Restart miner
cargo run --release --bin trinity-miner
```

### "Database is locked"

Only one process can write to the database at a time.

**Fix:**
```bash
# Stop all running trinity-* processes
pkill trinity

# Then restart what you need
```

### Wallet Not Found

```bash
# Create a new wallet:
cargo run --release --bin trinity-wallet -- new

# Or specify wallet location:
export HOME=/path/to/wallet/directory
```

### Low Hashrate

This is normal! CPU mining is intentionally slow.

**To improve:**
- Use `--release` flag (10-50x faster)
- Close other programs
- Use a faster CPU
- Wait for GPU mining support (coming soon!)

---

## 📚 Documentation

- **[README.md](README.md)** - Full documentation
- **[BITCOIN_FEATURES.md](BITCOIN_FEATURES.md)** - Economics & supply
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - How to contribute
- **[PROJECT_STATUS.md](PROJECT_STATUS.md)** - Roadmap

---

## 💬 Get Help

**Found a bug?** [Open an issue](https://github.com/littlekickoffkittie/trinitychain/issues)

**Have questions?** Check existing [issues](https://github.com/littlekickoffkittie/trinitychain/issues) or ask!

**Want to contribute?** Read [CONTRIBUTING.md](CONTRIBUTING.md)

---

## 🎯 Common Mining Scenarios

### Scenario 1: Solo Mining (Just You)

```bash
# Make sure your address is in config.toml, then:
cargo run --release --bin trinity-miner
```

**You'll get:**
- All blocks you find
- Full 50 area reward + transaction fees per block
- Stored in local `TrinityChain.db`

### Scenario 2: Mining with Friends (P2P Network)

```bash
# Friend 1 (host node):
cargo run --release --bin trinity-node --port 8333

# Friend 2 (connect to Friend 1):
cargo run --release --bin trinity-miner --peer <FRIEND1_IP>:8333

# Friend 3 (connect to Friend 1):
cargo run --release --bin trinity-miner --peer <FRIEND1_IP>:8333
```

**Now you're all on the same blockchain!**
- Blocks propagate between nodes
- Longest chain wins (Bitcoin-style)
- Everyone sees everyone's blocks

### Scenario 3: Starting Fresh

```bash
# Delete old blockchain:
rm TrinityChain.db

# Create new wallet:
cargo run --release --bin trinity-wallet -- new

# Configure config.toml with your new address, then:
cargo run --release --bin trinity-miner
```

---

## 🔥 Pro Tips

1. **Always use `--release`** - It's 10-50x faster than debug builds
2. **Backup your wallet** - Copy `~/.TrinityChain/wallet.json` somewhere safe
3. **Monitor difficulty** - As it increases, blocks take longer
4. **Wait for halvings** - First halving at block 210,000 (reward drops to 25)
5. **Check supply** - Track how much of the 420M has been mined

---

## 🎉 You're Ready!

You now know how to:
- ✅ Build the project
- ✅ Create a wallet
- ✅ Mine blocks
- ✅ Check your balance
- ✅ Transfer triangles
- ✅ Connect with other nodes

**Welcome to the Sierpinski Triangle Blockchain community!** 🔺⛓️

Happy mining! ⛏️✨

---

*Built with fractals, secured with cryptography, powered by Rust.*
