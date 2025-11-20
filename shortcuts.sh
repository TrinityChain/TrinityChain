#!/bin/bash

# TrinityChain Quick Actions
# Source this file to use shortcuts: source shortcuts.sh

TRINITY_DIR="$(pwd)"

# Wallet commands
alias wallet="cargo run --bin trinity-wallet --manifest-path=$TRINITY_DIR/Cargo.toml"
alias wallet-new="cargo run --bin trinity-wallet-new --manifest-path=$TRINITY_DIR/Cargo.toml"
alias wallet-backup="cargo run --bin trinity-wallet-backup --manifest-path=$TRINITY_DIR/Cargo.toml"
alias wallet-restore="cargo run --bin trinity-wallet-restore --manifest-path=$TRINITY_DIR/Cargo.toml"

# Transaction commands
alias send="cargo run --bin trinity-send --manifest-path=$TRINITY_DIR/Cargo.toml"
alias balance="cargo run --bin trinity-balance --manifest-path=$TRINITY_DIR/Cargo.toml"
alias history="cargo run --bin trinity-history --manifest-path=$TRINITY_DIR/Cargo.toml"

# Mining commands
alias miner="cargo run --bin trinity-miner --manifest-path=$TRINITY_DIR/Cargo.toml"
alias mine-block="cargo run --bin trinity-mine-block --manifest-path=$TRINITY_DIR/Cargo.toml"

# Network commands
alias node="cargo run --bin trinity-node --manifest-path=$TRINITY_DIR/Cargo.toml"
alias api="cargo run --bin trinity-api --manifest-path=$TRINITY_DIR/Cargo.toml"

# Utility commands
alias addressbook="cargo run --bin trinity-addressbook --manifest-path=$TRINITY_DIR/Cargo.toml"

# Release mode aliases (faster execution)
alias wallet-release="cargo run --release --bin trinity-wallet --manifest-path=$TRINITY_DIR/Cargo.toml"
alias miner-release="cargo run --release --bin trinity-miner --manifest-path=$TRINITY_DIR/Cargo.toml"
alias node-release="cargo run --release --bin trinity-node --manifest-path=$TRINITY_DIR/Cargo.toml"
alias api-release="cargo run --release --bin trinity-api --manifest-path=$TRINITY_DIR/Cargo.toml"

echo "TrinityChain shortcuts loaded!"
echo "Available commands: wallet, wallet-new, wallet-backup, wallet-restore, send, balance, history, miner, mine-block, node, api, addressbook"
echo "Release mode: wallet-release, miner-release, node-release, api-release"
