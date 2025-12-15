# TrinityChain Module Guide

## Quick Reference

| Module | Purpose | Public API | Tests |
|--------|---------|-----------|-------|
| `blockchain` | Chain validation & storage | `Blockchain`, `validate_block()` | `tests/` |
| `transaction` | Tx types & operations | `Transaction`, `TransactionInput/Output` | `tests/` |
| `geometry` | Triangle UTXO model | `Triangle`, `Point` | Inline |
| `mempool` | Pending transactions | `Mempool`, `add_transaction()` | Inline |
| `miner` | PoW mining | `mine_block()`, `Miner` | Inline |
| `wallet` | UTXO management | `Wallet`, `select_coins()` | Inline |
| `hdwallet` | BIP-39/32 wallets | `HDWallet`, `generate_mnemonic()` | Inline |
| `persistence` | SQLite storage | `Database`, `load_blockchain()` | Inline |
| `cache` | Performance caching | `Cache`, `CacheKey` | Inline |
| `network` | P2P networking | `Network`, `connect_peer()` | Inline |
| `discovery` | Peer discovery | `Discovery`, `find_peers()` | Inline |
| `sync` | Chain sync | `sync_blocks()`, `SyncManager` | Inline |
| `crypto` | ECDSA/hashing | `sign()`, `verify()`, `hash_sha256()` | Inline |
| `security` | Security utilities | `validate_input()`, `rate_limit()` | Inline |
| `config` | Configuration | `Config`, `load_config()` | None |
| `error` | Error types | `BlockchainError`, `TransactionError` | None |
| `cli` | CLI utilities | `prompt_user()`, `format_output()` | None |
| `addressbook` | Address contacts | `AddressBook`, `add_contact()` | Inline |
| `api` | REST API (optional) | `Node`, `run_api_server()` | Inline |

## Dependencies Between Modules

```
                    CLI Binaries (src/bin/)
                            ↓
    ┌───────────────────────┼───────────────────────┐
    ↓                       ↓                       ↓
Wallet Tools          Mining Tools           Networking Tools
(trinity-wallet)      (trinity-miner)        (trinity-node)
    ↓                       ↓                       ↓
  wallet ────→ blockchain ←─── miner
    ↓                ↑              ↑
 persistence         │             │
    ↑                │          crypto
    │        transaction          +
    └─ cache         │         security
                     │
                  geometry
                     +
                   mempool
```

## Module Details

### `blockchain.rs`

**Purpose**: Core blockchain logic, state machine, and validation.

**Key Types**:
```rust
pub struct Blockchain {
    blocks: Vec<Block>,
    utxo_set: HashMap<String, Vec<UTXO>>,
    difficulty: u32,
}

impl Blockchain {
    pub fn validate_block(&self, block: &Block) -> Result<(), BlockchainError>
    pub fn apply_block(&mut self, block: Block) -> Result<(), BlockchainError>
}
```

**Common Operations**:
- `new()` - Create genesis block
- `add_block()` - Add block to chain
- `get_balance()` - Query account balance
- `get_utxos()` - Get spendable UTXOs

**Dependencies**: `crypto`, `transaction`, `error`

### `transaction.rs`

**Purpose**: Transaction types, serialization, and validation.

**Transaction Types**:
1. **Transfer** - Move triangles between addresses
2. **Subdivision** - Split a triangle into smaller triangles
3. **Coinbase** - Miner reward (50 TCH per block)

**Key Types**:
```rust
pub enum Transaction {
    Transfer { ... },
    Subdivision { ... },
    Coinbase { ... },
}

pub struct TransactionInput {
    pub txid: [u8; 32],
    pub index: u32,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
}
```

**Common Operations**:
- `serialize()` - Convert to bytes
- `hash()` - Get transaction ID
- `validate()` - Check signatures and balances

**Dependencies**: `crypto`, `geometry`, `error`

### `geometry.rs`

**Purpose**: Triangle representation using fixed-point arithmetic (I32F32).

**Key Types**:
```rust
pub struct Triangle {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

pub struct Point {
    pub x: I32F32,
    pub y: I32F32,
}

impl Triangle {
    pub fn area(&self) -> I32F32  // Shoelace formula
    pub fn is_valid(&self) -> bool
}
```

**Invariants**:
- Non-zero area (positive area required)
- Coordinates must be within valid bounds
- No degenerate triangles (collinear points)

**Common Operations**:
- `area()` - Calculate triangle area
- `is_valid()` - Validate geometry
- `subdivide()` - Split into smaller triangles

**Why I32F32?**
- Fixed-point eliminates floating-point rounding errors
- Deterministic across all platforms
- Used for fee calculations and geometric proofs

**Dependencies**: None (uses standard library + fixed_point crate)

### `mempool.rs`

**Purpose**: Transaction memory pool - staging ground for unmined transactions.

**Key Types**:
```rust
pub struct Mempool {
    transactions: BTreeMap<String, Transaction>,
}

impl Mempool {
    pub fn add(&mut self, tx: Transaction) -> Result<(), Error>
    pub fn remove(&mut self, txid: &str) -> Option<Transaction>
    pub fn get_all(&self) -> Vec<Transaction>
}
```

**Common Operations**:
- `add()` - Add pending transaction
- `remove()` - Remove (when mined)
- `get_all()` - Get all pending for mining

**Dependencies**: `transaction`, `error`

### `miner.rs`

**Purpose**: Proof-of-Work mining implementation.

**Key Types**:
```rust
pub struct Miner {
    difficulty: u32,
}

impl Miner {
    pub async fn mine_block(
        &self,
        previous_hash: [u8; 32],
        transactions: Vec<Transaction>,
    ) -> Result<Block, Error>
}
```

**Common Operations**:
- `mine_block()` - Find valid nonce (async)
- `adjust_difficulty()` - Retarget based on block times
- `calculate_reward()` - Get miner reward for block

**Algorithm**:
1. Try nonces sequentially starting from 0
2. Hash block header with each nonce
3. Check if hash < difficulty threshold
4. Return block when threshold met

**Dependencies**: `crypto`, `transaction`, `blockchain`, `error`

### `wallet.rs`

**Purpose**: Wallet operations and UTXO management.

**Key Types**:
```rust
pub struct Wallet {
    address: String,
    private_key: [u8; 32],
    public_key: Vec<u8>,
}

impl Wallet {
    pub fn select_coins(&self, amount: I32F32) -> Result<Vec<UTXO>, Error>
    pub fn create_transaction(&self, to: &str, amount: I32F32) -> Result<Transaction, Error>
}
```

**Common Operations**:
- `new()` - Generate new wallet
- `from_private_key()` - Restore from key
- `select_coins()` - UTXO selection algorithm
- `create_transaction()` - Build transfer tx

**UTXO Selection Algorithm**:
- First-fit: Take oldest UTXOs
- Area-weighted: Prefer similar-sized UTXOs
- Change output: Send remainder to change address

**Dependencies**: `crypto`, `blockchain`, `hdwallet`, `transaction`, `error`

### `hdwallet.rs`

**Purpose**: Hierarchical Deterministic wallets (BIP-39 mnemonics, BIP-32 derivation).

**Key Types**:
```rust
pub struct HDWallet {
    master_key: [u8; 32],
}

impl HDWallet {
    pub fn generate() -> Result<(HDWallet, String), Error>  // Returns (wallet, mnemonic)
    pub fn from_mnemonic(mnemonic: &str) -> Result<HDWallet, Error>
    pub fn derive_address(&self, index: u32) -> String
}
```

**Common Operations**:
- `generate()` - Create new HD wallet with 12-word mnemonic
- `from_mnemonic()` - Restore wallet from words
- `derive_address()` - Get address at index
- `derive_private_key()` - Get private key at index

**Standards**:
- BIP-39: 12-word English mnemonics
- BIP-32: m/44'/0'/0'/0/i derivation path

**Dependencies**: `crypto`, `error`

### `persistence.rs`

**Purpose**: SQLite database layer for persistent storage.

**Key Types**:
```rust
pub struct Database {
    connection: rusqlite::Connection,
}

impl Database {
    pub fn open(path: &str) -> Result<Database, Error>
    pub fn load_blockchain(&self) -> Result<Blockchain, Error>
    pub fn save_block(&self, block: &Block) -> Result<(), Error>
}
```

**Tables**:
- `blocks` - Block headers and data
- `transactions` - Mined transactions with metadata
- `utxos` - Unspent outputs (materialized view)
- `addresses` - Address book

**Common Operations**:
- `open()` - Connect to database
- `load_blockchain()` - Load entire chain from disk
- `save_block()` - Persist new block
- `get_utxos()` - Query available UTXOs

**Dependencies**: `blockchain`, `transaction`, `error`

### `cache.rs`

**Purpose**: Performance optimization through caching.

**Key Types**:
```rust
pub struct Cache<K, V> {
    data: HashMap<K, V>,
}

impl<K, V> Cache<K, V> {
    pub fn get(&self, key: &K) -> Option<&V>
    pub fn insert(&mut self, key: K, value: V)
    pub fn clear(&mut self)
}
```

**Cached Items**:
- UTXO set (updated per block)
- Block metadata (heights, hashes)
- Validation results (signature checks)

**Common Operations**:
- `get()` - Retrieve cached value
- `insert()` - Cache computation result
- `invalidate()` - Clear cache on new block

**Dependencies**: None (generic utility)

### `network.rs`

**Purpose**: P2P networking - peer connections and message routing.

**Key Types**:
```rust
pub struct Network {
    peers: HashMap<String, PeerConnection>,
}

impl Network {
    pub async fn connect_peer(&self, host: &str, port: u16) -> Result<(), Error>
    pub async fn broadcast(&self, message: &Message) -> Result<(), Error>
}
```

**Message Types**:
- `Ping` - Keep-alive
- `Block` - New block announcement
- `Transaction` - New transaction
- `Sync` - Block download request

**Common Operations**:
- `start_server()` - Listen for incoming connections
- `connect_peer()` - Outbound connection
- `broadcast()` - Send to all connected peers
- `list_peers()` - Get connected peer list

**Dependencies**: `error`

### `discovery.rs`

**Purpose**: Peer discovery mechanisms for bootstrapping connections.

**Key Types**:
```rust
pub struct Discovery {
    bootstrap_nodes: Vec<String>,
    dns_seeds: Vec<String>,
}

impl Discovery {
    pub async fn find_peers(&self) -> Result<Vec<String>, Error>
}
```

**Methods**:
- **DNS Seeds**: Query well-known DNS seed hosts
- **Bootstrap Nodes**: Connect to hardcoded bootstrap addresses
- **Peer Exchange**: Ask peers for their peer lists

**Common Operations**:
- `find_peers()` - Get list of peer addresses
- `connect_bootstrap()` - Connect to initial peers

**Dependencies**: `network`, `error`

### `sync.rs`

**Purpose**: Chain synchronization - bringing nodes up to date.

**Key Types**:
```rust
pub struct SyncManager {
    current_height: u64,
    target_height: u64,
}

impl SyncManager {
    pub async fn sync(&mut self, blockchain: &mut Blockchain) -> Result<(), Error>
}
```

**Synchronization Strategies**:
- **Initial Sync**: Download all blocks from peers
- **Incremental Sync**: Download only missing blocks
- **Fast Sync**: Download state snapshots (future)

**Common Operations**:
- `sync()` - Main synchronization loop
- `download_blocks()` - Get blocks from peer
- `validate_chain()` - Verify downloaded chain

**Dependencies**: `blockchain`, `network`, `error`

### `crypto.rs`

**Purpose**: Cryptographic operations - signing, verification, hashing.

**Key Functions**:
```rust
pub fn sign(message: &[u8], private_key: &[u8; 32]) -> Result<Vec<u8>, Error>
pub fn verify(message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, Error>
pub fn hash_sha256(data: &[u8]) -> [u8; 32]
pub fn hash_ripemd160(data: &[u8]) -> [u8; 20]
pub fn public_key_from_private(private_key: &[u8; 32]) -> Result<Vec<u8>, Error>
```

**Algorithms**:
- **ECDSA**: secp256k1 (Bitcoin's curve)
- **Hashing**: SHA256 for blocks and transactions
- **Address Generation**: SHA256(pubkey) → RIPEMD160 → Base58Check

**Common Operations**:
- `sign()` - Sign transaction
- `verify()` - Verify transaction signature
- `hash_sha256()` - Hash block/transaction
- `public_key_from_private()` - Derive public key

**Dependencies**: `sha2`, `secp256k1` crates

### `security.rs`

**Purpose**: Security utilities beyond basic cryptography.

**Key Functions**:
```rust
pub fn validate_input(input: &str) -> Result<(), Error>
pub fn rate_limit(key: &str, limit: u32, window: Duration) -> bool
```

**Protections**:
- Input validation (length, format)
- Rate limiting on network requests
- Timing attack prevention
- Memory safety (zeroize secrets)

**Common Operations**:
- `validate_input()` - Check user input
- `rate_limit()` - Throttle operations
- `zeroize()` - Clear sensitive memory

**Dependencies**: `zeroize` crate

### `config.rs`

**Purpose**: Configuration management from `config.toml`.

**Key Types**:
```rust
pub struct Config {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub consensus: ConsensusConfig,
}

impl Config {
    pub fn load() -> Result<Config, Error>
}
```

**Configuration Sections**:
- `[network]` - Ports, peers, timeouts
- `[database]` - Path, cache size
- `[consensus]` - Block time, reward, difficulty

**Common Operations**:
- `load()` - Load from config.toml
- `get_network()` - Get network settings
- `get_database()` - Get database settings

**Dependencies**: None (uses serde + toml)

### `error.rs`

**Purpose**: Error types and error handling.

**Error Types**:
```rust
pub enum BlockchainError {
    InvalidBlockHash,
    InvalidTransaction,
    InsufficientFunds,
    // ...
}

pub enum TransactionError {
    InvalidSignature,
    DoubleSpend,
    // ...
}
```

**Common Operations**:
- Implement `Display` for user-friendly messages
- Implement `From<T>` for error conversion
- Provide error codes for API responses

**Dependencies**: None (standard library)

### `cli.rs`

**Purpose**: CLI utilities shared across binary tools.

**Key Functions**:
```rust
pub fn prompt_user(prompt: &str) -> String
pub fn confirm(prompt: &str) -> bool
pub fn format_output(data: &impl Serialize) -> String
pub fn format_balance(amount: I32F32) -> String
```

**Common Operations**:
- `prompt_user()` - Interactive input
- `confirm()` - Yes/No confirmation
- `format_output()` - Pretty-print results
- `format_balance()` - Format TCH amounts

**Dependencies**: `clap`, `colored`

### `addressbook.rs`

**Purpose**: Named address book for user convenience.

**Key Types**:
```rust
pub struct AddressBook {
    contacts: HashMap<String, Contact>,
}

impl AddressBook {
    pub fn add_contact(&mut self, name: &str, address: &str) -> Result<(), Error>
    pub fn get_address(&self, name: &str) -> Option<&str>
}
```

**Common Operations**:
- `add_contact()` - Save address with label
- `get_address()` - Look up by name
- `list()` - Show all contacts
- `export()` / `import()` - Backup/restore

**Dependencies**: `persistence`, `error`

### `api.rs` (Optional: `api` feature)

**Purpose**: REST API for programmatic blockchain access.

**Key Types**:
```rust
pub struct Node {
    blockchain: Arc<RwLock<Blockchain>>,
    wallet: Arc<Wallet>,
}

pub async fn run_api_server(node: Arc<Node>) -> Result<(), Error>
```

**Endpoints**:
- `GET /blocks/:height` - Get block
- `GET /balance/:address` - Check balance
- `POST /transaction` - Send transaction
- `GET /mempool` - View pending transactions

**Common Operations**:
- `run_api_server()` - Start Axum server
- RESTful blockchain operations

**Dependencies**: `axum`, `tower-http` (feature-gated)

**Note**: Only available with `--features api`

## Import Patterns

### In a Binary

```rust
use trinitychain::{
    blockchain::Blockchain,
    wallet::Wallet,
    config::load_config,
    error::TransactionError,
};
```

### In a Module

```rust
use crate::{
    blockchain::Blockchain,
    error::BlockchainError,
    crypto::sign,
};
```

### Conditional (Feature-Gated)

```rust
#[cfg(feature = "api")]
use crate::api::run_api_server;
```

## Adding Tests

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blockchain() {
        let bc = Blockchain::new([0; 32], 1).unwrap();
        assert_eq!(bc.height(), 1);
    }
}
```

### Integration Test Example

Create `tests/integration_test.rs`:

```rust
use trinitychain::{blockchain::Blockchain, wallet::Wallet};

#[test]
fn test_full_workflow() {
    let bc = Blockchain::new([0; 32], 1).unwrap();
    let wallet = Wallet::new().unwrap();
    // Test complete flow...
}
```

## Performance Tips

1. **Use persistence caching** for frequently accessed blocks
2. **Batch validation** of multiple transactions
3. **Async I/O** for network operations (tokio)
4. **Fixed-point arithmetic** (I32F32) avoids float overhead
5. **Parallel validation** with Rayon for transaction batches

## Debugging

### Enable Logging
```bash
RUST_LOG=debug cargo run --bin trinity-wallet
RUST_LOG=trinitychain=trace cargo build
```

### Common Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| "Invalid block hash" | Wrong difficulty or nonce | Check miner difficulty |
| "Double spend" | Same UTXO in multiple txs | Mempool already preventing this |
| "Insufficient funds" | Not selecting enough UTXOs | Check UTXO selection algorithm |
| Database locked | Multiple writers | Ensure single Database instance |

## Versioning & Stability

- **v0.2.0**: Current stable
- API stability: ✅ Stable (semver)
- ABI: Not guaranteed (internal Rust)
- File format: Stable (SQLite schema versioned)
