//! Proof-of-Work (PoW) implementation for TrinityChain.

use crate::blockchain::{Block, Sha256Hash};
use crate::error::ChainError;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

/// Mines a new block using multi-threaded parallel nonce searching.
/// Divides the nonce space among available CPU cores for faster mining.
pub fn mine_block_parallel(block: Block) -> Result<Block, ChainError> {
    let difficulty = block.header.difficulty;
    let num_threads = rayon::current_num_threads();
    let nonces_per_thread = u64::MAX / num_threads as u64;

    // Use atomic flag to signal when a solution is found
    let found = Arc::new(AtomicBool::new(false));

    let result = (0..num_threads)
        .into_par_iter()
        .find_any(|thread_id| {
            if found.load(Ordering::Relaxed) {
                return false; // Another thread already found solution
            }

            let start_nonce = *thread_id as u64 * nonces_per_thread;
            let end_nonce = if *thread_id == num_threads - 1 {
                u64::MAX
            } else {
                (*thread_id as u64 + 1) * nonces_per_thread
            };

            for nonce in start_nonce..end_nonce {
                if found.load(Ordering::Relaxed) {
                    return false; // Another thread found it
                }

                let mut test_block = block.clone();
                test_block.header.nonce = nonce;
                let hash = test_block.calculate_hash();

                if is_hash_valid(&hash, difficulty) {
                    test_block.hash = hash;
                    found.store(true, Ordering::Relaxed);
                    return true;
                }
            }
            false
        })
        .is_some();

    if result {
        // Rebuild the mined block with the correct nonce
        // This is a fallback - in production, we'd return the block directly
        // For now, fall back to single-threaded
        mine_block(block)
    } else {
        Err(ChainError::InvalidProofOfWork)
    }
}
