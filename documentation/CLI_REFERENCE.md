# TrinityChain CLI Tools - Complete Reference

This document provides complete usage examples for all TrinityChain CLI tools.

---

## Getting Started

### Build All Tools

```bash
cd TrinityChain
cargo build --release
```

All CLI tools will be in `target/release/`.

---

## Core Mining Workflow

### 1. Create a Wallet

```bash
# Create new wallet
cargo run --release --bin trinity-wallet -- new

# Output:
# ✅ Wallet 'trinity-default' created!
# 📁 Path: ~/.TrinityChain/wallet.json
# 🔑 Address: e54369c2ef44435ba34ef6ee881f33b2fa3126c0
```

### 2. List Your Wallets

```bash
cargo run --release --bin trinity-wallet -- list

# Output:
# Wallets:
# 1. trinity-default
# 2. alice
# 3. bob
```

### 3. Mine Blocks

```bash
# Start continuous mining
cargo run --release --bin trinity-miner

# Or mine a single block
cargo run --release --bin trinity-mine-block
```

### 4. Check Your Balance

```bash
cargo run --release --bin trinity-balance -- <your_address>

# Example:
cargo run --release --bin trinity-balance -- e54369c2ef44435ba34ef6ee881f33b2fa3126c0
```

---

## Wallet Management

### Backup Your Wallet

```bash
cargo run --release --bin trinity-wallet -- backup alice

# Creates backup file you can store safely
```

### Restore Wallet

```bash
cargo run --release --bin trinity-wallet -- restore ~/.TrinityChain/wallet-alice-backup.json
```

---

## Transactions

### Send Transaction

```bash
cargo run --release --bin trinity-send -- <recipient> <amount> --from <wallet_name>

# Example:
cargo run --release --bin trinity-send -- \
  1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2 \
  50.0 \
  --from alice
```

### View Transaction History

```bash
cargo run --release --bin trinity-history -- <address>

# Example:
cargo run --release --bin trinity-history -- e54369c2ef44435ba34ef6ee881f33b2fa3126c0

# Output shows:
# - All incoming transactions
# - All outgoing transactions
# - Timestamps and amounts
```

---

## Networking

### Start a Network Node

```bash
cargo run --release --bin trinity-node

# Launches interactive TUI showing:
# - Blockchain height
# - Peer connections
# - Mempool status
# - Mining activity
# - Real-time stats
```

### Connect to a Peer

```bash
cargo run --release --bin trinity-connect -- <peer_address>:<port>

# Example:
cargo run --release --bin trinity-connect -- 192.168.1.100:8333
```

---

## Optional: REST API

For programmatic access or web integrations:

```bash
cargo run --release --bin trinity-api

# Starts API server on http://localhost:3000
# Available endpoints:
# GET  /api/blockchain/stats
# GET  /api/blockchain/blocks
# GET  /api/address/:addr/balance
# POST /api/transaction
# And many more (see API_ENDPOINTS.md)
```

---

## Utilities

### Address Book

```bash
cargo run --release --bin trinity-addressbook

# Interactive address management
```

### Sign Guestbook

```bash
cargo run --release --bin trinity-guestbook

# Sign and view blockchain guestbook
```

### Telegram Bot

```bash
# Set environment variables first
export TELEGRAM_TOKEN="your_token_here"
export TRINITY_API="http://localhost:3000"

cargo run --release --bin trinity-telegram-bot
```

---

## Configuration

Edit `config.toml` in the project root:

```toml
[miner]
threads = 4                              # Number of mining threads
beneficiary_address = "your_address"    # Where mining rewards go

[network]
api_port = 3000                         # REST API port
p2p_port = 8333                         # P2P networking port
max_peers = 50                          # Max peer connections

[database]
path = "/home/user/.TrinityChain"       # Data storage location
```

---

## Shell Scripts

### Quick Mining Setup

```bash
#!/bin/bash
# mine.sh - Start mining workflow

cd TrinityChain
cargo build --release

# Create wallet if needed
cargo run --release --bin trinity-wallet -- new

# Get your address
ADDR=$(cargo run --release --bin trinity-wallet -- address)
echo "Mining address: $ADDR"

# Start mining
cargo run --release --bin trinity-miner
```

### Check All Balances

```bash
#!/bin/bash
# check_all_balances.sh - View all wallet balances

for addr in $(cargo run --release --bin trinity-wallet -- list); do
  echo "=== $addr ==="
  cargo run --release --bin trinity-balance -- "$addr"
done
```

---

## Troubleshooting

### "Wallet not found"
```bash
# Create a wallet first
cargo run --release --bin trinity-wallet -- new
```

### "Connection refused"
```bash
# Make sure the node is running in another terminal
cargo run --release --bin trinity-node
```

### "Mining too slow"
```bash
# Increase threads in config.toml
# Or check system load:
top
# Increase CPU cores allocated if running in VM
```

### "Database locked"
```bash
# Only one process can access the database at a time
# Close other TrinityChain processes
pkill -f trinity-
```

---

## Performance Tips

1. **Mining**: Use `trinity-miner` for continuous mining (more efficient than `trinity-mine-block`)
2. **Multiple nodes**: Run on separate machines for network simulation
3. **Peer connections**: Connect to multiple peers for better network distribution
4. **Batch transactions**: Group send operations to reduce overhead

---

## Integration Examples

### Send 10 TRC Every Hour

```bash
#!/bin/bash
while true; do
  cargo run --release --bin trinity-send -- \
    1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2 \
    10.0 \
    --from alice
  sleep 3600
done
```

### Monitor Blockchain Growth

```bash
#!/bin/bash
while true; do
  height=$(curl -s http://localhost:3000/api/blockchain/height)
  echo "Block height: $height"
  sleep 10
done
```

### Automated Wallet Backup

```bash
#!/bin/bash
date=$(date +%Y%m%d_%H%M%S)
cargo run --release --bin trinity-wallet -- backup alice
mv ~/.TrinityChain/wallet-alice-backup.json \
   ~/.TrinityChain/backups/wallet-alice-$date.json
```

---

## Advanced Usage

See individual tool help:

```bash
cargo run --release --bin trinity-wallet -- --help
cargo run --release --bin trinity-miner -- --help
cargo run --release --bin trinity-send -- --help
# etc.
```

For API reference: See `documentation/API_ENDPOINTS.md`
