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

echo "Running test suite..."
# Run tests to ensure project health before starting services
cargo test --release

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

# Start headless node in background and persist logs
echo "Starting headless node (logs/node.log)..."
nohup "$NODE_BIN" 9090 --peer 127.0.0.1:9090 > logs/node.log 2>&1 &
NODE_PID=$!
echo "Node PID: $NODE_PID"

# Give node a moment to initialize and create DB files
sleep 1

echo "Headless node started and syncing (logs/node.log)."

echo "Launching interactive miner dashboard (will run in foreground). To run detached instead, re-run this script with --detach-miner flag."

if [ "${1-}" = "--detach-miner" ] || [ "${2-}" = "--detach-miner" ]; then
  # Run miner in a loop detached, useful for CI or background runs
  nohup bash -c 'while true; do "$MINER_BIN" "$ADDR" --blocks 1; sleep 1; done' > logs/miner.log 2>&1 &
  MINER_PID=$!
  echo "Detached miner PID: $MINER_PID"
  echo "Quickstart launched. Tail logs with: tail -f logs/node.log logs/miner.log"
  exit 0
else
  # Launch the interactive miner TUI in the foreground so you can watch dashboards.
  exec "$MINER_BIN" "$ADDR"
fi

