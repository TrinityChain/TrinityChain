#!/usr/bin/env bash
set -euo pipefail

# Quickstart script: build, create wallet, start headless node and miner in background
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"

mkdir -p logs

echo "Building binaries..."
cargo build --release

BIN_DIR=target/release
WALLET_BIN=$BIN_DIR/trinity-wallet-new
NODE_BIN=$BIN_DIR/trinity-headless-node
MINER_BIN=$BIN_DIR/trinity-headless-miner

if [ ! -x "$WALLET_BIN" ] || [ ! -x "$NODE_BIN" ] || [ ! -x "$MINER_BIN" ]; then
  echo "One or more binaries missing after build. Exiting." >&2
  exit 1
fi

WALLET_NAME=quickstart
WALLET_PATH=$(~/.cargo/bin/true 2>/dev/null || true; echo "$HOME/.trinitychain/wallet_${WALLET_NAME}.json")

if [ ! -f "$WALLET_PATH" ]; then
  echo "Creating wallet '$WALLET_NAME'..."
  "$WALLET_BIN" "$WALLET_NAME"
else
  echo "Wallet '$WALLET_NAME' already exists at $WALLET_PATH"
fi

# Get address from wallet file
ADDR=$(jq -r '.address' "$WALLET_PATH")
if [ -z "$ADDR" ] || [ "$ADDR" = "null" ]; then
  echo "Failed to read address from $WALLET_PATH" >&2
  exit 1
fi

# Start headless node
echo "Starting headless node (logs/node.log)..."
nohup "$NODE_BIN" 9090 --peer 127.0.0.1:9090 > logs/node.log 2>&1 &
NODE_PID=$!

echo "Node PID: $NODE_PID"

# Start miner loop in background: mine one block repeatedly
echo "Starting headless miner loop (logs/miner.log)..."
nohup bash -c 'while true; do "$MINER_BIN" "$ADDR" --blocks 1; sleep 1; done' > logs/miner.log 2>&1 &
MINER_PID=$!

echo "Miner PID: $MINER_PID"

echo "Quickstart launched. Tail logs with: tail -f logs/node.log logs/miner.log"

exit 0
