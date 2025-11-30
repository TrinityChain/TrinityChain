# GEMINI.md

This file provides a comprehensive overview of the TrinityChain project for the Gemini CLI.

## Project Overview

TrinityChain is a proof-of-work blockchain implemented in Rust. Its unique feature is a UTXO (Unspent Transaction Output) model where each UTXO is a triangle with real coordinates. The spendable value of each UTXO is its geometric area.

The project consists of two main parts:

1.  **The Blockchain Core (Rust):** This is the main implementation of the blockchain, including the consensus engine, P2P networking, and all the logic for handling transactions and blocks.
2.  **The Dashboard (React):** A web-based interface for monitoring the blockchain's status in real-time.

### Key Technologies

*   **Backend (Blockchain):**
    *   Language: Rust
    *   Cryptography: `secp256k1` for digital signatures.
    *   Consensus: Proof-of-Work (SHA-256).
    *   Storage: SQLite.
    *   Networking: Custom TCP-based P2P protocol.
*   **Frontend (Dashboard):**
    *   Language: JavaScript (React).
    *   Communication: WebSockets to connect to the blockchain node.

### Architecture

The blockchain node is a monolithic application that includes:

*   **Wallet:** For managing keys and creating transactions.
*   **Mempool:** For storing pending transactions.
*   **Miner:** For creating new blocks.
*   **Blockchain:** For managing the chain of blocks and the UTXO set.
*   **Persistence:** For storing the blockchain data in a SQLite database.
*   **Network Layer:** For communicating with other nodes.
*   **API Layer:** A REST and WebSocket API for interacting with the node.

## Building and Running

### Blockchain (Rust)

**Build:**
```bash
cargo build --release
```

**Test:**
```bash
cargo test --lib
```

**Run a node:**
```bash
./target/release/trinity-node
```

**Create a new wallet:**
```bash
./target/release/trinity-wallet-new
```

**Mine a block:**
```bash
./target/release/trinity-mine-block
```

**Sign the guestbook:**
```bash
./target/release/trinity-guestbook sign "Your message here"
```

**View the guestbook:**
```bash
./target/release/trinity-guestbook view
```

### Dashboard (React)

**Install dependencies:**
```bash
cd dashboard
npm install
```

**Build:**
```bash
npm run build
```

The dashboard is then accessible at `http://localhost:3000/dashboard` (assuming the node is running on port 3000).

## Configuration

TrinityChain uses a `config.toml` file for configuration. A default configuration file is provided in the root of the project. You can modify this file to change the network ports, database path, and mining settings.

## Development Conventions

*   **Code Style:** Run `cargo fmt` to format the Rust code.
*   **Linting:** Run `cargo clippy` to check for common mistakes and style issues.
*   **Testing:** New functionality should be accompanied by tests.
*   **Documentation:** Public APIs should be documented with rustdoc comments.
*   **Contribution:** Contributions are welcome. See `CONTRIBUTING.md` for more details.
