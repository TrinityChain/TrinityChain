# Seirtrichain Shortcuts Guide

This guide provides quick command shortcuts for the Seirtrichain blockchain project, eliminating the need to type `cargo run --bin` before every command.

## Installation

### One-Time Setup

Load the shortcuts in your current terminal session:

```bash
source ~/trinitychain/shortcuts.sh
```

### Permanent Setup

To automatically load shortcuts every time you open a terminal:

```bash
echo "source ~/trinitychain/shortcuts.sh" >> ~/.bashrc
source ~/.bashrc
```

For other shells:
- **Zsh**: Add to `~/.zshrc`
- **Fish**: Add to `~/.config/fish/config.fish`

## Available Commands

### Wallet Management

| Shortcut | Full Command | Description |
|----------|--------------|-------------|
| `wallet` | `cargo run --bin trinity-wallet` | Main wallet interface |
| `wallet-new` | `cargo run --bin trinity-wallet-new` | Create a new wallet |
| `wallet-backup` | `cargo run --bin trinity-wallet-backup` | Backup your wallet |
| `wallet-restore` | `cargo run --bin trinity-wallet-restore` | Restore wallet from backup |

### Transactions

| Shortcut | Full Command | Description |
|----------|--------------|-------------|
| `send` | `cargo run --bin trinity-send` | Send coins to an address |
| `balance` | `cargo run --bin trinity-balance` | Check wallet balance |
| `history` | `cargo run --bin trinity-history` | View transaction history |

### Mining

| Shortcut | Full Command | Description |
|----------|--------------|-------------|
| `miner` | `cargo run --bin trinity-miner` | Start the mining process |
| `mine-block` | `cargo run --bin trinity-mine-block` | Mine a single block |

### Network

| Shortcut | Full Command | Description |
|----------|--------------|-------------|
| `node` | `cargo run --bin trinity-node` | Start a blockchain node |
| `api` | `cargo run --bin trinity-api` | Start the API server |

### Utilities

| Shortcut | Full Command | Description |
|----------|--------------|-------------|
| `addressbook` | `cargo run --bin trinity-addressbook` | Manage address book |

### Release Mode (Optimized)

For faster execution with optimizations enabled:

| Shortcut | Description |
|----------|-------------|
| `wallet-release` | Wallet in release mode |
| `miner-release` | Miner in release mode (recommended) |
| `node-release` | Node in release mode (recommended) |
| `api-release` | API server in release mode (recommended) |

## Usage Examples

### Basic Workflow

```bash
# Create a new wallet
wallet-new

# Check your balance
balance

# View transaction history
history

# Send coins to someone
send

# Start mining
miner
```

### Running Services

```bash
# Start a node in release mode (recommended for production)
node-release

# Start the API server
api-release

# Start mining in release mode (better performance)
miner-release
```

### Wallet Management

```bash
# Backup your wallet
wallet-backup

# Manage your address book
addressbook

# Restore from a backup
wallet-restore
```

## Tips

1. **Use Release Mode for Services**: When running the miner, node, or API server for extended periods, use the `-release` variants for better performance.

2. **Quick Access**: You can run these commands from any directory as they include the full path to the project.

3. **Arguments**: You can still pass arguments to the commands:
   ```bash
   balance --address ABC123...
   send --to XYZ789... --amount 10
   ```

4. **Updating Shortcuts**: If you modify `shortcuts.sh`, reload it with:
   ```bash
   source ~/trinitychain/shortcuts.sh
   ```

## Troubleshooting

### Shortcuts Not Working

If shortcuts aren't recognized:

1. Make sure you've sourced the file:
   ```bash
   source ~/trinitychain/shortcuts.sh
   ```

2. Check that the file is executable:
   ```bash
   chmod +x ~/trinitychain/shortcuts.sh
   ```

3. Verify the path in your shell config matches your installation location.

### Command Not Found

If a specific shortcut doesn't work:

1. Check that the binary exists in `Cargo.toml`
2. Try running the full `cargo run --bin` command to see the actual error
3. Ensure the project builds successfully: `cargo build`

## Contributing

If you add new binaries to the project, update `shortcuts.sh` with corresponding aliases to maintain consistency across the development team.
