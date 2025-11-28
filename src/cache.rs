//! Caching layer for frequently accessed blockchain data
//!
//! Provides LRU caching for:
//! - Recent blocks (100 block limit)
//! - UTXO set entries (hot triangles)
//! - Address balances
//! - Block hashes

use crate::blockchain::{Block, Sha256Hash};
use crate::geometry::Triangle;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache for recent blocks
pub struct BlockCache {
    cache: Arc<RwLock<LruCache<Sha256Hash, Block>>>,
}

impl BlockCache {
    /// Create a new block cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        let cache =
            LruCache::new(NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap()));
        Self {
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    /// Get a block from cache
    pub async fn get(&self, hash: &Sha256Hash) -> Option<Block> {
        let mut cache = self.cache.write().await;
        cache.get(hash).cloned()
    }

    /// Put a block in cache
    pub async fn put(&self, hash: Sha256Hash, block: Block) {
        let mut cache = self.cache.write().await;
        cache.put(hash, block);
    }

    /// Clear all cached blocks
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        (cache.len(), cache.cap().get())
    }
}

impl Clone for BlockCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// Cache for UTXO set entries
pub struct UtxoCache {
    cache: Arc<RwLock<LruCache<Sha256Hash, Triangle>>>,
}

impl UtxoCache {
    /// Create a new UTXO cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        let cache =
            LruCache::new(NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(10000).unwrap()));
        Self {
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    /// Get a triangle from cache
    pub async fn get(&self, hash: &Sha256Hash) -> Option<Triangle> {
        let mut cache = self.cache.write().await;
        cache.get(hash).cloned()
    }

    /// Put a triangle in cache
    pub async fn put(&self, hash: Sha256Hash, triangle: Triangle) {
        let mut cache = self.cache.write().await;
        cache.put(hash, triangle);
    }

    /// Remove a triangle from cache
    pub async fn remove(&self, hash: &Sha256Hash) {
        let mut cache = self.cache.write().await;
        cache.pop(hash);
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        (cache.len(), cache.cap().get())
    }
}

impl Clone for UtxoCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// Cache for address balances
pub struct BalanceCache {
    cache: Arc<RwLock<HashMap<String, f64>>>,
}

impl BalanceCache {
    /// Create a new balance cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached balance for address
    pub async fn get(&self, address: &str) -> Option<f64> {
        let cache = self.cache.read().await;
        cache.get(address).copied()
    }

    /// Set balance for address
    pub async fn set(&self, address: String, balance: f64) {
        let mut cache = self.cache.write().await;
        cache.insert(address, balance);
    }

    /// Invalidate all cached balances
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Invalidate specific address balance
    pub async fn invalidate(&self, address: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(address);
    }

    /// Get cache size
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

impl Clone for BalanceCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

impl Default for BalanceCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined cache for all blockchain data
pub struct BlockchainCache {
    pub blocks: BlockCache,
    pub utxo: UtxoCache,
    pub balances: BalanceCache,
}

impl BlockchainCache {
    /// Create a new blockchain cache
    pub fn new(block_capacity: usize, utxo_capacity: usize) -> Self {
        Self {
            blocks: BlockCache::new(block_capacity),
            utxo: UtxoCache::new(utxo_capacity),
            balances: BalanceCache::new(),
        }
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        self.blocks.clear().await;
        self.utxo.clear().await;
        self.balances.invalidate_all().await;
    }
}

impl Clone for BlockchainCache {
    fn clone(&self) -> Self {
        Self {
            blocks: self.blocks.clone(),
            utxo: self.utxo.clone(),
            balances: self.balances.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::BlockHeader;

    #[tokio::test]
    async fn test_block_cache() {
        let cache = BlockCache::new(10);
        let hash = [0u8; 32];
        let block = Block {
            header: BlockHeader {
                height: 1,
                previous_hash: [0; 32],
                timestamp: 0,
                difficulty: 1,
                nonce: 0,
                merkle_root: [0; 32],
                headline: None,
            },
            hash: [0; 32],
            transactions: vec![],
        };

        cache.put(hash, block.clone()).await;
        let retrieved = cache.get(&hash).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_balance_cache() {
        let cache = BalanceCache::new();
        let addr = "test_address".to_string();

        cache.set(addr.clone(), 100.5).await;
        let balance = cache.get(&addr).await;
        assert_eq!(balance, Some(100.5));

        cache.invalidate(&addr).await;
        let balance = cache.get(&addr).await;
        assert!(balance.is_none());
    }

    #[tokio::test]
    async fn test_utxo_cache_lru_eviction() {
        let cache = UtxoCache::new(5);
        let triangle = Triangle::genesis();

        // Fill cache to capacity
        for i in 0..5 {
            let mut hash = [0u8; 32];
            hash[0] = i as u8;
            cache.put(hash, triangle.clone()).await;
        }

        let (size, cap) = cache.stats().await;
        assert_eq!(size, 5);
        assert_eq!(cap, 5);

        // Add one more, should evict oldest
        let mut hash = [0u8; 32];
        hash[0] = 6;
        cache.put(hash, triangle).await;

        let (size, _) = cache.stats().await;
        assert_eq!(size, 5); // Still 5 due to LRU eviction
    }
}
