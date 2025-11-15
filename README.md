# TrinityChain

A triangle-based blockchain built on geometric principles with Proof-of-Work consensus.

## 🔺 What is TrinityChain?

TrinityChain is an innovative blockchain where value is represented as geometric triangles rather than traditional coins. Each unit of value has an area, and the blockchain maintains a UTXO (Unspent Triangle Output) set.

### Key Features

- **🔺 Triangle Economy** - Geometric triangles as the fundamental unit of value
- **⛏️ Proof-of-Work** - SHA-256 mining with Bitcoin-like difficulty adjustment
- **🔐 Real Cryptography** - ECDSA signatures with secp256k1, UTXO model
- **📉 Deflationary** - 21M max supply, halving every 210k blocks
- **🔄 Triangle Subdivision** - Split triangles to transfer value
- **🌐 P2P Network** - Decentralized node communication
- **📱 Telegram Bot** - Interact with the blockchain via Telegram
- **📊 Web Dashboard** - Telegram Mini App for blockchain exploration

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- SQLite3
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/TrinityChain/TrinityChain.git
cd trinitychain

# Build the project
cargo build --release
```

## 📦 Binaries

TrinityChain includes several command-line tools:

### Wallet Management

```bash
# Create a new wallet
cargo run --bin trinity-wallet-new

# Check balance
cargo run --bin trinity-balance

# View transaction history
cargo run --bin trinity-history

# Send triangles
cargo run --bin trinity-send <recipient_address> <amount>

# Backup wallet
cargo run --bin trinity-wallet-backup

# Restore wallet
cargo run --bin trinity-wallet-restore
```

### Mining

```bash
# Start mining
cargo run --bin trinity-miner <your_wallet_address>

# Mine a single block
cargo run --bin trinity-mine-block <beneficiary_address>
```

### Network Node

```bash
# Start a node
cargo run --bin trinity-node <port>

# Start a node and connect to peer
cargo run --bin trinity-node <port> --peer <peer_host:peer_port>
```

### API Server

```bash
# Start the REST API server
cargo run --bin trinity-api
# API will be available at http://localhost:3000
```

### Telegram Bot

```bash
# Set your bot token
export TELOXIDE_TOKEN="your_bot_token_here"

# Start the Telegram bot
cargo run --bin trinity-telegram-bot
```

## 🌐 API Endpoints

The TrinityChain API provides the following endpoints:

### Blockchain
- `GET /blockchain/height` - Get current blockchain height
- `GET /blockchain/stats` - Get blockchain statistics
- `GET /blockchain/blocks` - Get recent blocks
- `GET /blockchain/block/:hash` - Get block by hash
- `GET /blockchain/block/by-height/:height` - Get block by height

### Addresses & Balances
- `GET /address/:addr/balance` - Get address balance
- `GET /address/:addr/triangles` - Get address triangles
- `GET /address/:addr/history` - Get transaction history

### Transactions
- `POST /transaction` - Submit a transaction
- `GET /transaction/:hash` - Get transaction status
- `GET /transactions/pending` - Get pending transactions
- `GET /transactions/mempool-stats` - Get mempool statistics

### Mining
- `GET /mining/status` - Get mining status
- `POST /mining/start` - Start mining
- `POST /mining/stop` - Stop mining

### Network
- `GET /network/peers` - Get connected peers
- `GET /network/info` - Get network information

## 📱 Telegram Bot Commands

- `/start` - Welcome message
- `/help` - Show all commands
- `/stats` - View blockchain statistics
- `/balance <address>` - Check wallet balance
- `/blocks` - View recent blocks
- `/genesis` - See genesis block
- `/triangles` - Count total triangles in UTXO
- `/difficulty` - Current mining difficulty
- `/height` - Current blockchain height
- `/about` - Learn about TrinityChain
- `/dashboard` - Open blockchain explorer

## 📊 Dashboard

The TrinityChain dashboard is a Telegram Mini App that provides a visual interface for exploring the blockchain.

**Live Dashboard:** [https://dashboard-hlbo27wax-starjamisom-2642s-projects.vercel.app](https://dashboard-hlbo27wax-starjamisom-2642s-projects.vercel.app)

See [dashboard/README.md](dashboard/README.md) for setup instructions.

## 🏗️ Architecture

### Core Components

- **Blockchain** - Main blockchain logic and UTXO state management
- **Geometry** - Triangle primitives and subdivision logic
- **Transactions** - Coinbase, Subdivision, and Transfer transactions
- **Crypto** - ECDSA key generation and signing with secp256k1
- **Mining** - Proof-of-Work mining implementation
- **Network** - P2P node communication
- **Persistence** - SQLite database for blockchain storage
- **API** - REST API server built with Axum
- **Security** - Firewall rules and VPN/SOCKS5 proxy support

### Supply & Halving

TrinityChain follows a Bitcoin-like supply model:

- **Max Supply:** 21,000,000 area units
- **Initial Block Reward:** 1000 area
- **Halving Interval:** Every 210,000 blocks
- **Halving Schedule:**
  - Blocks 0-209,999: 1000 area
  - Blocks 210,000-419,999: 500 area
  - Blocks 420,000-629,999: 250 area
  - And so on...

### Triangle Geometry

Each triangle in the UTXO set has:
- Three vertices (x, y coordinates)
- An owner address
- An optional parent hash (for subdivision tracking)

Triangles can be subdivided into smaller triangles, enabling fractional value transfer.

## 🔒 Security

TrinityChain includes several security features:

- **Firewall Rules** - IP-based access control
- **VPN Support** - Route traffic through VPN tunnels
- **SOCKS5 Proxy** - Proxy support for enhanced privacy
- **Rate Limiting** - Protection against spam and DoS
- **Authentication** - Optional authentication for API endpoints

Configure via environment variables:
```bash
export TRINITY_VPN_INTERFACE="wg0"
export TRINITY_SOCKS5_PROXY="127.0.0.1:9050"
export TRINITY_REQUIRE_AUTH="true"
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_blockchain_creation

# Run with output
cargo test -- --nocapture
```

## 📝 Development

### Project Structure

```
trinitychain/
├── src/
│   ├── bin/           # Binary executables
│   ├── lib.rs         # Library entry point
│   ├── blockchain.rs  # Core blockchain logic
│   ├── geometry.rs    # Triangle primitives
│   ├── transaction.rs # Transaction types
│   ├── crypto.rs      # Cryptography utilities
│   ├── mining.rs      # Mining implementation
│   ├── network.rs     # P2P networking
│   ├── persistence.rs # Database layer
│   ├── api.rs         # REST API
│   ├── wallet.rs      # Wallet utilities
│   ├── addressbook.rs # Address book management
│   └── security.rs    # Security features
├── dashboard/         # Web dashboard (Telegram Mini App)
├── Cargo.toml         # Rust dependencies
└── README.md          # This file
```

### Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is open source. See the repository for license details.

## 🔗 Links

- **GitHub:** [https://github.com/TrinityChain/TrinityChain](https://github.com/TrinityChain/TrinityChain)
- **Dashboard:** [https://dashboard-hlbo27wax-starjamisom-2642s-projects.vercel.app](https://dashboard-hlbo27wax-starjamisom-2642s-projects.vercel.app)
- **Telegram Bot:** Contact @TrinityChainBot (when configured)

## 🙏 Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/)
- [secp256k1](https://github.com/rust-bitcoin/rust-secp256k1) - Elliptic curve cryptography
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLite](https://www.sqlite.org/) - Embedded database
- [Teloxide](https://github.com/teloxide/teloxide) - Telegram bot framework

---

**🔺 TrinityChain - Where Geometry Meets Blockchain**
