//! Core blockchain implementation for TrinityChain, including block structure,
//! chain validation, UTXO management, and mining difficulty adjustment.

use std::collections::HashMap;
use std::sync::Mutex;
use sha2::{Digest, Sha256};
use crate::error::ChainError;
use crate::geometry::{Coord, Point, Triangle, GEOMETRIC_TOLERANCE};
use crate::mempool::Mempool;
use crate::miner::mine_block;
use crate::transaction::{
    Address, CoinbaseTx, TransferTx, Transaction
};

// ============================================================================
// Types
// ============================================================================

/// A 32-byte hash (SHA-256)
pub type Sha256Hash = [u8; 32];

/// A single block header
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub timestamp: u64,
    pub previous_hash: Sha256Hash,
    pub merkle_root: Sha256Hash,
    pub difficulty: u32,
    pub nonce: u64,
}

impl BlockHeader {
    /// Calculate the hash of the block header
    pub fn hash(&self) -> Sha256Hash {
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.previous_hash);
        hasher.update(self.merkle_root);
        hasher.update(self.difficulty.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.finalize().into()
    }
}

/// A complete block
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create a new block candidate before mining
    pub fn new(
        height: u64,
        previous_hash: Sha256Hash,
        difficulty: u32,
        transactions: Vec<Transaction>,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        let merkle_root = Block::calculate_merkle_root(&transactions);

        Block {
            header: BlockHeader {
                height,
                timestamp,
                previous_hash,
                merkle_root,
                difficulty,
                nonce: 0,
            },
            transactions,
        }
    }

    /// Calculate the SHA-256 hash of the block (header hash)
    pub fn hash(&self) -> Sha256Hash {
        self.header.hash()
    }

    /// Standard Merkle Root calculation (hashing all transaction hashes together)
    pub fn calculate_merkle_root(transactions: &[Transaction]) -> Sha256Hash {
        let mut hasher = Sha256::new();
        for tx in transactions {
            hasher.update(tx.hash());
        }
        hasher.finalize().into()
    }

    pub fn hash_to_u256(hash: &Sha256Hash) -> [u8; 32] {
        // Remove "Simple placeholder for conversion"
        *hash
    }

    pub fn hash_to_target(difficulty: &u32) -> [u8; 32] {
        // Target: 2^(256 - difficulty)
        let mut target = [0xFF; 32]; 
        let leading_zeros = *difficulty / 8; // Number of leading zero bytes
        let partial_bits = *difficulty % 8; // Number of partial bits
        
        for i in 0..leading_zeros as usize {
            target[i] = 0;
        }

        if leading_zeros < 32 && partial_bits > 0 {
            // E.g., if partial_bits is 4, we need the first 4 bits to be 0, 
            // so we set the byte to 0b00001111 = 15
            target[leading_zeros as usize] = (0xFF >> partial_bits) as u8;
        }

        target
    }
}

// ============================================================================
// State Management (UTXO Cache)
// ============================================================================

/// The global state of all unspent triangles (UTXOs) and derived address balances.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TriangleState {
    /// The actual UTXO set: Maps Triangle Hash -> Triangle
    pub utxo_set: HashMap<Sha256Hash, Triangle>,
    /// Derived balances: Maps Address -> Total Area (Coord)
    pub address_balances: HashMap<Address, Coord>,
}

impl TriangleState {
    /// Creates a new, empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current total area owned by an address.
    pub fn get_balance(&self, address: &Address) -> Coord {
        *self.address_balances.get(address).unwrap_or(&Coord::from_num(0))
    }

    /// Updates the UTXO set and derived balances based on a transaction.
    /// This is the core state transition logic.
    pub fn apply_transaction(&mut self, tx: &Transaction, _block_height: u64) -> Result<(), ChainError> {
        match tx {
            // 1. Coinbase: Creates new value and assigns it to the miner (beneficiary)
            Transaction::Coinbase(tx) => {
                // Create the new Genesis Triangle for the reward
                let new_triangle = Triangle::new(
                    Point::new(Coord::from_num(0.0), Coord::from_num(0.0)), // Point A
                    Point::new(Coord::from_num(0.0), Coord::from_num(0.0)), // Point B
                    Point::new(Coord::from_num(0.0), Coord::from_num(0.0)), // Point C
                    None,
                    tx.beneficiary_address.clone(),
                )
                .with_effective_value(tx.reward_area);

                let tx_hash = Transaction::Coinbase(tx.clone()).hash();
                self.utxo_set.insert(tx_hash, new_triangle.clone());
                
                // Update balance for the beneficiary
                *self.address_balances.entry(tx.beneficiary_address.clone()).or_insert(Coord::from_num(0)) += tx.reward_area;
            }

            // 2. Transfer: Consumes one UTXO and creates a new one (or two: change is not implemented yet)
            Transaction::Transfer(tx) => {
                // a) Validate and remove input (Consumed UTXO)
                let input_hash = tx.input_hash;
                let consumed_triangle = self.utxo_set.remove(&input_hash).ok_or_else(|| {
                    ChainError::TriangleNotFound(format!(
                        "Transfer input {} not found in UTXO set",
                        hex::encode(input_hash)
                    ))
                })?;

                // b) Check ownership and decrease sender balance
                if consumed_triangle.owner != tx.sender {
                    // This check should have happened in tx.validate_with_state, but we ensure state is consistent
                    self.utxo_set.insert(input_hash, consumed_triangle.clone()); // Put it back
                    return Err(ChainError::InvalidTransaction(format!(
                        "Sender {} does not own input triangle (owned by {})",
                        tx.sender, consumed_triangle.owner
                    )));
                }

                let input_value = consumed_triangle.effective_value();
                let total_spent = tx.amount + tx.fee_area;
                let remaining_value = input_value - total_spent;
                
                // Decrease sender balance by the full consumed value (value + fee + remainder)
                let sender_balance = self.address_balances.entry(tx.sender.clone()).or_insert(Coord::from_num(0));
                *sender_balance = *sender_balance - input_value;
                if *sender_balance < Coord::from_num(0) { *sender_balance = Coord::from_num(0); } // Prevent negative balance

                // c) Create new UTXO for the new owner (Output UTXO)
                let new_owner_triangle = consumed_triangle.clone()
                    .change_owner(tx.new_owner.clone())
                    .with_effective_value(tx.amount); // Assign the transferred amount as new value

                let tx_hash = Transaction::Transfer(tx.clone()).hash();
                self.utxo_set.insert(tx_hash, new_owner_triangle);

                // d) Update new owner balance
                *self.address_balances.entry(tx.new_owner.clone()).or_insert(Coord::from_num(0)) += tx.amount;

                // e) Handle Change UTXO (Remaining value is treated as 'change' and returned to sender)
                if remaining_value > GEOMETRIC_TOLERANCE {
                    // Create a separate UTXO for the change, sent back to the sender
                    let change_tx = Transaction::Transfer(TransferTx {
                        input_hash: tx_hash, // Use the hash of the current output as a pseudo-input for change
                        new_owner: tx.sender.clone(),
                        sender: tx.sender.clone(),
                        amount: remaining_value,
                        fee_area: Coord::from_num(0), // No fee on change
                        nonce: 0,
                        signature: None,
                        public_key: None,
                        memo: Some("Change".to_string()),
                    });
                    
                    let change_hash = change_tx.hash();
                    let change_triangle = consumed_triangle
                        .change_owner(tx.sender.clone())
                        .with_effective_value(remaining_value);

                    self.utxo_set.insert(change_hash, change_triangle);
                    
                    // Re-add change value to sender's balance (since we deducted the whole input_value earlier)
                    *self.address_balances.entry(tx.sender.clone()).or_insert(Coord::from_num(0)) += remaining_value;
                }
            }

            // 3. Subdivision: Consumes one UTXO and creates three new children UTXOs
            Transaction::Subdivision(tx) => {
                // a) Validate and remove input (Consumed UTXO)
                let input_hash = tx.parent_hash;
                let consumed_triangle = self.utxo_set.remove(&input_hash).ok_or_else(|| {
                    ChainError::TriangleNotFound(format!(
                        "Subdivision parent {} not found in UTXO set",
                        hex::encode(input_hash)
                    ))
                })?;

                // b) Check ownership
                if consumed_triangle.owner != tx.owner_address {
                    self.utxo_set.insert(input_hash, consumed_triangle); // Put it back
                    return Err(ChainError::InvalidTransaction(
                        "Subdivision tx owner does not match triangle owner".to_string()
                    ));
                }

                // c) Remove parent value from owner's balance
                let parent_value = consumed_triangle.effective_value();
                let owner_balance = self.address_balances.entry(tx.owner_address.clone()).or_insert(Coord::from_num(0));
                *owner_balance = *owner_balance - parent_value;
                if *owner_balance < Coord::from_num(0) { *owner_balance = Coord::from_num(0); } // Prevent negative balance

                // d) Create new UTXOs (Outputs)
                let total_area_of_children = consumed_triangle.area();
                let fee_ratio = tx.fee_area / total_area_of_children;

                for child in &tx.children {
                    // The value of each child is calculated based on its proportion of the parent's area
                    let child_area = child.area();
                    let child_value = child_area * (Coord::from_num(1.0) - fee_ratio);
                    
                    let new_utxo = child.clone()
                        .change_owner(tx.owner_address.clone())
                        .with_effective_value(child_value); // Assign new value post-fee

                    // Hash the new UTXO based on its properties (Triangle hash)
                    self.utxo_set.insert(new_utxo.hash(), new_utxo.clone());
                    
                    // e) Update owner balance with the new children's value
                    *self.address_balances.entry(tx.owner_address.clone()).or_insert(Coord::from_num(0)) += child_value;
                }
            }
        }
        Ok(())
    }
}


// ============================================================================
// Blockchain
// ============================================================================

/// The main structure holding the chain, UTXO state, and pending transactions.
#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: u32,
    pub mempool: Mempool,
    pub state: TriangleState, // UTXO Cache (TriangleState)
    // Mutex for difficulty adjustment to prevent race conditions during mining/sync
    pub difficulty_mutex: Mutex<()>, 
}

impl Clone for Blockchain {
    fn clone(&self) -> Self {
        Self {
            blocks: self.blocks.clone(),
            difficulty: self.difficulty,
            mempool: self.mempool.clone(),
            state: self.state.clone(),
            difficulty_mutex: Mutex::new(()),
        }
    }
}

impl Blockchain {
    /// Initializer for a new blockchain with the genesis block.
    pub fn new(genesis_miner_address: Address, initial_difficulty: u32) -> Result<Self, ChainError> {
        let genesis_block = Self::create_genesis_block(genesis_miner_address, initial_difficulty)?;
        
        let mut blockchain = Blockchain {
            blocks: vec![],
            difficulty: initial_difficulty,
            mempool: Mempool::new(),
            state: TriangleState::new(),
            difficulty_mutex: Mutex::new(()),
        };

        // Apply the genesis block to initialize the state
        blockchain.apply_block(genesis_block)?;
        Ok(blockchain)
    }

    /// Creates the immutable genesis block.
    fn create_genesis_block(miner_address: Address, initial_difficulty: u32) -> Result<Block, ChainError> {
        // Create a special genesis transaction (Coinbase)
        let coinbase_tx = Transaction::Coinbase(CoinbaseTx {
            reward_area: Coord::from_num(1_000_000.0), // Initial fixed supply
            beneficiary_address: miner_address,
        });

        let transactions = vec![coinbase_tx];
        let merkle_root = Block::calculate_merkle_root(&transactions);

        let header = BlockHeader {
            height: 0,
            timestamp: 1672531200000, // Jan 1, 2023
            previous_hash: [0u8; 32],
            merkle_root,
            difficulty: initial_difficulty,
            nonce: 0,
        };

        let genesis_block = Block {
            header,
            transactions,
        };

        mine_block(genesis_block)
    }

    /// Calculates the block reward based on height (halving model)
    pub fn calculate_block_reward(height: u64) -> f64 {
        const INITIAL_REWARD: f64 = 50.0; // In area units
        const HALVING_INTERVAL: u64 = 210000;
        
        let halving_count = height / HALVING_INTERVAL;
        if halving_count >= 64 { // Prevent overflow and reward going to 0
            0.0
        } else {
            INITIAL_REWARD / (2u64.pow(halving_count as u32) as f64)
        }
    }


    // ============================================================================
    // Core Chain and State Logic
    // ============================================================================

    /// Attempts to apply a block to the chain, performing all necessary validations.
    pub fn apply_block(&mut self, block: Block) -> Result<(), ChainError> {
        let _lock = self.difficulty_mutex.lock().map_err(|_| ChainError::InternalError("Failed to lock difficulty mutex".to_string()))?;

        let is_genesis = block.header.height == 0;

        // 1. Validate block height and previous hash
        if !is_genesis {
            let last_block = self.blocks.last().ok_or(ChainError::InvalidBlock("Chain is empty, but block height is not 0".to_string()))?;

            if block.header.height != last_block.header.height + 1 {
                return Err(ChainError::InvalidBlock(format!(
                    "Expected height {}, got {}",
                    last_block.header.height + 1,
                    block.header.height
                )));
            }

            if block.header.previous_hash != last_block.hash() {
                return Err(ChainError::InvalidBlock("Previous hash mismatch".to_string()));
            }
        } else if self.blocks.len() > 0 {
             return Err(ChainError::InvalidBlock("Genesis block already exists".to_string()));
        }

        // 2. Validate Proof-of-Work (PoW)
        if !self.verify_pow(&block) {
            return Err(ChainError::InvalidBlock("Invalid Proof-of-Work".to_string()));
        }

        // 3. Temporary state to check transaction validity
        let mut temp_state = self.state.clone(); 
        let mut seen_utxos: HashMap<Sha256Hash, Sha256Hash> = HashMap::new(); // Tracks UTXOs used in this block

        // 4. Validate and apply transactions to temporary state
        for (i, tx) in block.transactions.iter().enumerate() {
            // All transactions must validate their size first
            tx.validate_size()?;

            // The first transaction must be a Coinbase transaction
            if i == 0 {
                match tx {
                    Transaction::Coinbase(_) => {},
                    _ => return Err(ChainError::InvalidBlock("First transaction must be Coinbase".to_string())),
                }
            } else {
                // Check general validation (including signature)
                tx.validate(&temp_state)?;
            }

            // Check for double spending within the block for Transfer and Subdivision
            match tx {
                Transaction::Transfer(transfer_tx) => {
                    if seen_utxos.contains_key(&transfer_tx.input_hash) {
                         return Err(ChainError::InvalidTransaction("Double spend detected within block".to_string()));
                    }
                    // Mark input as used
                    seen_utxos.insert(transfer_tx.input_hash, tx.hash());
                },
                Transaction::Subdivision(subdivision_tx) => {
                    if seen_utxos.contains_key(&subdivision_tx.parent_hash) {
                         return Err(ChainError::InvalidTransaction("Double spend detected within block".to_string()));
                    }
                    // Mark input as used
                    seen_utxos.insert(subdivision_tx.parent_hash, tx.hash());
                },
                _ => {} // Coinbase has no input to double spend
            }

            // Apply transaction to the temporary state
            temp_state.apply_transaction(tx, block.header.height)?;
        }
        
        // 5. Finalize block validity checks
        if Block::calculate_merkle_root(&block.transactions) != block.header.merkle_root {
            return Err(ChainError::InvalidBlock("Merkle root mismatch".to_string()));
        }
        
        // 6. Update chain state
        self.blocks.push(block.clone());
        self.state = temp_state; // Commit the temporary state
        
        // 7. Remove confirmed transactions from mempool
        for tx in &block.transactions {
            self.mempool.remove_transaction(&tx.hash());
        }

        // 8. Adjust difficulty (This logic is usually complex and based on time, skipping for now)
        // self.adjust_difficulty();

        Ok(())
    }

    /// Verifies the Proof-of-Work constraint of a block.
    fn verify_pow(&self, block: &Block) -> bool {
        let hash_target = Block::hash_to_target(&block.header.difficulty);
        let block_hash_int = Block::hash_to_u256(&block.hash());
        
        // Block hash must be less than or equal to the target
        block_hash_int <= hash_target 
    }
}

// ============================================================================
// Blockchain
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blockchain() {
        let blockchain = Blockchain::new("test_miner".to_string(), 1).unwrap();
        assert_eq!(blockchain.blocks.len(), 1);
        let genesis_block = &blockchain.blocks[0];
        assert_eq!(genesis_block.header.height, 0);
        assert_eq!(genesis_block.header.previous_hash, [0u8; 32]);
    }

    #[test]
    fn test_apply_invalid_block() {
        let mut blockchain = Blockchain::new("test_miner".to_string(), 1).unwrap();
        let last_block = blockchain.blocks.last().unwrap().clone();
        let new_block = Block::new(
            last_block.header.height + 2, // Invalid height
            last_block.hash(),
            1,
            vec![],
        );
        let res = blockchain.apply_block(new_block);
        assert!(res.is_err());
    }
}

