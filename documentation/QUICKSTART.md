# 🚀 Quickstart Guide - Get Mining in 5 Minutes!

Welcome to the **Sierpinski Triangle Blockchain**! This guide will get you from zero to mining your first blocks.

---

## ⚡ Super Quick Start (3 Commands)

```bash
# 1. Clone and build
git clone https://github.com/littlekickoffkittie/trinitychain.git
cd trinitychain && cargo build --release

# 2. Create wallet
cargo run --release --bin trinity-wallet-new

# 3. Start mining (use the address from step 2)
cargo run --release --bin trinity-miner <YOUR_ADDRESS_HERE>
```

That's it! You're now mining triangular cryptocurrency. 🔺⛓️

---

## 📋 Detailed Setup

### Prerequisites

**Required:**
- Rust 1.90+ ([install here](https://rustup.rs/))
- SQLite (usually pre-installed on Linux/Mac)
- 100MB disk space

**Platform Support:**
- ✅ Linux (Ubuntu, Debian, Arch, etc.)
- ✅ macOS
- ✅ Windows (WSL recommended)
- ✅ Termux (Android)

### Step 1: Install Rust

```bash
# If you don't have Rust installed:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then:
source $HOME/.cargo/env

# Verify installation:
rustc --version
# Should show: rustc 1.90.0 or higher
```

### Step 2: Clone Repository

```bash
git clone https://github.com/littlekickoffkittie/trinitychain.git
cd trinitychain
```

### Step 3: Build Project

```bash
# Build all binaries (takes 1-3 minutes first time)
cargo build --release

# Binaries will be in: target/release/
```

**Available Commands:**
- `trinity-wallet-new` - Create wallet
- `trinity-wallet` - View wallet info
- `trinity-miner` - Mine blocks (recommended)
- `trinity-balance` - Check your balance
- `trinity-send` - Transfer triangles
- `trinity-node` - Run P2P node
- `trinity-api` - REST API server
- `trinity-guestbook` - Sign and view the guestbook
- `trinity-user` - User login/logout management

### Step 4: Create Wallet

```bash
cargo run --release --bin trinity-wallet-new
```

**Output:**
```
🎉 Wallet created successfully!
📁 Location: /home/you/.TrinityChain/wallet.json

📋 Your Address:
1KjT9P8mW...xyz123

⚠️  IMPORTANT: Backup your wallet file! Without it, you lose access to your triangles.
```

**Save your address** - you'll need it for mining!

### Step 5: Start Mining

```bash
# Replace with YOUR address from step 4
cargo run --release --bin trinity-miner 1KjT9P8mW...xyz123
```

**What you'll see:**
```
╔═══════════════════════════════════════════════════════════════╗
║         ███████╗██╗███████╗██████╗ ████████╗██████╗ ██╗      ║
║         ██╔════╝██║██╔════╝██╔══██╗╚══██╔══╝██╔══██╗██║      ║
║         ███████╗██║█████╗  ██████╔╝   ██║   ██████╔╝██║      ║
║         ╚════██║██║██╔══╝  ██╔══██╗   ██║   ██╔══██╗██║      ║
║         ███████║██║███████╗██║  ██║   ██║   ██║  ██║██║      ║
║         ╚══════╝╚═╝╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝      ║
║                    ⛏️  FRACTAL MINER ⛏️                        ║
╚═══════════════════════════════════════════════════════════════╝

⛏️  Mining block #1 (difficulty: 2)...

┌─────────────────────────────────────────────────────────────┐
│ ✨ BLOCK FOUND! #1                                           │
├─────────────────────────────────────────────────────────────┤
│ Hash: 00a3f7b9...8c2d1e4f                                   │
│ Attempts: 1,247                                             │
│ Time: 2.3s                                                  │
│ Avg Hashrate: 542 H/s                                       │
└─────────────────────────────────────────────────────────────┘

╔══════════════════════════════════════════════════════════╗
║                    📊 MINING STATS                       ║
╠══════════════════════════════════════════════════════════╣
║ 🔺 Blocks Mined: 1                                       ║
║ 🏔️  Chain Height: 1                                      ║
║ 🎯 Difficulty: 2                                         ║
║ 💎 Current Reward: 50 area units + fees                  ║
║ 📈 Total Supply: 1,000 / 420,000,000 (0.0002%)          ║
║ ⏰ Blocks to Halving: 209,999                            ║
║ 🎚️  Halving Era: 0                                       ║
╚══════════════════════════════════════════════════════════╝
```

**Congratulations! You just mined your first block!** 🎉

---

## 🔍 Understanding the Stats

| Stat | Meaning |
|------|---------|
| **Blocks Mined** | How many blocks YOU found this session |
| **Chain Height** | Total blocks in the blockchain |
| **Difficulty** | How hard it is to find blocks (adjusts every 2,016 blocks) |
| **Current Reward** | Area units you earn per block (halves every 210k blocks) |
| **Total Supply** | Total area units mined by everyone / max supply (420M) |
| **Blocks to Halving** | When mining reward drops by 50% |
| **Halving Era** | Which halving cycle we're in (0 = first era) |

---

## ✅ Next Steps

### Check Your Balance

```bash
cargo run --release --bin trinity-balance
```

**Output:**
```
💰 Your Balance:

Triangle 1:
  Hash: a3b5c7...xyz
  Area: 1000.0
  Owner: 1KjT9P8mW...xyz123

Triangle 2:
  Hash: d4e6f8...abc
  Area: 1000.0
  Owner: 1KjT9P8mW...xyz123

Total: 2 triangles, 2000.0 area units
```

### Send Triangles to Someone

```bash
# Get a friend's address, then:
cargo run --release --bin trinity-send <FRIEND_ADDRESS> <TRIANGLE_HASH>

# Example:
cargo run --release --bin trinity-send \
  1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2 \
  a3b5c7d9e1f3...xyz123
```

### Connect to Another Node (Multi-Player!)

```bash
# Person A (host):
cargo run --release --bin trinity-node 8333

# Person B (connect to A):
cargo run --release --bin trinity-node 8334 --peer <PERSON_A_IP>:8333

# Now Person B can mine:
cargo run --release --bin trinity-miner <YOUR_ADDRESS> --peer <PERSON_A_IP>:8333
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
cargo run --release --bin trinity-miner <YOUR_ADDRESS>
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
cargo run --release --bin trinity-wallet-new

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
# Start mining, that's it!
cargo run --release --bin trinity-miner <YOUR_ADDRESS>
```

**You'll get:**
- All blocks you find
- Full 50 area reward + transaction fees per block
- Stored in local `TrinityChain.db`

### Scenario 2: Mining with Friends (P2P Network)

```bash
# Friend 1 (host node):
cargo run --release --bin trinity-node 8333

# Friend 2 (connect to Friend 1):
cargo run --release --bin trinity-miner <ADDRESS> --peer <FRIEND1_IP>:8333

# Friend 3 (connect to Friend 1):
cargo run --release --bin trinity-miner <ADDRESS> --peer <FRIEND1_IP>:8333
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
cargo run --release --bin trinity-wallet-new

# Start mining from genesis:
cargo run --release --bin trinity-miner <YOUR_ADDRESS>
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
