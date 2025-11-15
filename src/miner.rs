//! Proof-of-Work (PoW) implementation for TrinityChain.

use crate::blockchain::{Block, Sha256Hash};
use crate::error::ChainError;

/// Checks if a hash meets the required difficulty target.
/// The difficulty is the required number of leading zeros in the hash.
/// Optimized version that checks bytes directly without hex encoding.
pub fn is_hash_valid(hash: &Sha256Hash, difficulty: u64) -> bool {
    // Prevent DoS by limiting difficulty to a reasonable maximum (64 hex chars = 32 bytes)
    const MAX_DIFFICULTY: u64 = 64;
    let difficulty = difficulty.min(MAX_DIFFICULTY);

    if difficulty == 0 {
        return true;
    }

    // Each byte represents 2 hex characters
    // Calculate how many full bytes must be zero
    let full_zero_bytes = (difficulty / 2) as usize;

    // Check if we need to examine a partial byte (for odd difficulty)
    let check_nibble = difficulty % 2 == 1;

    // Fast check: verify full zero bytes
    for i in 0..full_zero_bytes {
        if hash[i] != 0 {
            return false;
        }
    }

    // If we have an odd difficulty, check the upper nibble of the next byte
    if check_nibble && full_zero_bytes < hash.len() {
        // Upper nibble must be 0 (byte value must be < 16)
        if hash[full_zero_bytes] >= 16 {
            return false;
        }
    }

    true
}

/// Mines a new block by searching for a nonce that satisfies the current difficulty.
pub fn mine_block(mut block: Block) -> Result<Block, ChainError> {
    let difficulty = block.header.difficulty;
    let mut nonce: u64 = 0;
    
    loop {
        block.header.nonce = nonce;
        let hash = block.calculate_hash();
        
        if is_hash_valid(&hash, difficulty) {
            block.hash = hash;
            return Ok(block);
        }

        nonce = nonce.checked_add(1).ok_or(ChainError::InvalidProofOfWork)?; 
    }
}
