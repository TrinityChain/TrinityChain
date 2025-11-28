//! Fee estimation and transaction fee market
//!
//! Provides dynamic fee estimation based on network conditions and transaction size

use crate::mempool::Mempool;
use crate::transaction::Transaction;

/// Fee statistics for the current network state
#[derive(Debug, Clone, Copy)]
pub struct FeeStats {
    /// Minimum fee (satoshis per byte equivalent)
    pub min_fee: u64,
    /// Median fee of recent transactions
    pub median_fee: u64,
    /// High priority fee (faster confirmation)
    pub high_priority_fee: u64,
    /// Network congestion level (0-100)
    pub congestion_level: u8,
}

impl Default for FeeStats {
    fn default() -> Self {
        Self {
            min_fee: 1,
            median_fee: 5,
            high_priority_fee: 10,
            congestion_level: 0,
        }
    }
}

/// Fee estimator for transactions
pub struct FeeEstimator {
    /// Base fee in satoshis
    base_fee: u64,
    /// Fee multiplier for congestion
    congestion_multiplier: f64,
}

impl FeeEstimator {
    /// Create a new fee estimator
    pub fn new(base_fee: u64) -> Self {
        Self {
            base_fee,
            congestion_multiplier: 1.0,
        }
    }

    /// Estimate fee for a transaction based on size
    pub fn estimate_fee(&self, tx_size_bytes: usize) -> u64 {
        let base = self.base_fee as f64 * tx_size_bytes as f64 / 1000.0; // per KB
        (base * self.congestion_multiplier).ceil() as u64
    }

    /// Estimate fee for low priority (slower, cheaper)
    pub fn estimate_low_priority(&self, tx_size_bytes: usize) -> u64 {
        let base = self.estimate_fee(tx_size_bytes);
        (base as f64 * 0.5).ceil() as u64
    }

    /// Estimate fee for standard priority
    pub fn estimate_standard(&self, tx_size_bytes: usize) -> u64 {
        self.estimate_fee(tx_size_bytes)
    }

    /// Estimate fee for high priority (faster, expensive)
    pub fn estimate_high_priority(&self, tx_size_bytes: usize) -> u64 {
        let base = self.estimate_fee(tx_size_bytes);
        (base as f64 * 2.0).ceil() as u64
    }

    /// Update congestion multiplier based on mempool state
    pub fn update_from_mempool(&mut self, mempool: &Mempool) {
        let pool_size = mempool.len();
        let max_pool_size = 10000; // From mempool const

        let congestion = (pool_size as f64 / max_pool_size as f64).min(1.0);
        self.congestion_multiplier = 1.0 + (congestion * 2.0); // Up to 3x multiplier
    }

    /// Get current fee statistics
    pub fn get_stats(&self, mempool: &Mempool) -> FeeStats {
        let pool_size = mempool.len();
        let congestion = ((pool_size as f64 / 10000.0) * 100.0).min(100.0) as u8;

        FeeStats {
            min_fee: self.base_fee,
            median_fee: self.estimate_standard(250), // Typical tx size
            high_priority_fee: self.estimate_high_priority(250),
            congestion_level: congestion,
        }
    }

    /// Check if fee is acceptable (above minimum)
    pub fn is_acceptable_fee(&self, fee: u64, tx_size_bytes: usize) -> bool {
        let min_fee = self.estimate_low_priority(tx_size_bytes);
        fee >= min_fee
    }

    /// Check if fee is sufficient for quick confirmation
    pub fn is_high_priority(&self, fee: u64, tx_size_bytes: usize) -> bool {
        let high_priority_fee = self.estimate_high_priority(tx_size_bytes);
        fee >= high_priority_fee
    }
}

impl Default for FeeEstimator {
    fn default() -> Self {
        Self::new(1) // 1 satoshi base fee
    }
}

/// Calculate approximate transaction size in bytes
pub fn estimate_transaction_size(tx: &Transaction) -> usize {
    match tx {
        Transaction::Transfer(tx) => {
            // ~160 bytes for signature + pubkey + fields
            let base = 160;
            let memo_size = tx.memo.as_ref().map(|m| m.len()).unwrap_or(0);
            base + memo_size
        }
        Transaction::Subdivision(tx) => {
            // ~100 bytes for parent hash + 3 children + signature
            100 + (tx.children.len() * 50)
        }
        Transaction::Coinbase(_) => {
            // ~50 bytes
            50
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_estimator_creation() {
        let estimator = FeeEstimator::new(1);
        assert_eq!(estimator.base_fee, 1);
    }

    #[test]
    fn test_estimate_fee() {
        let estimator = FeeEstimator::new(1);
        let fee = estimator.estimate_fee(250);
        assert!(fee > 0);
    }

    #[test]
    fn test_fee_priority_ordering() {
        let estimator = FeeEstimator::new(10);
        let size = 250;

        let low = estimator.estimate_low_priority(size);
        let standard = estimator.estimate_standard(size);
        let high = estimator.estimate_high_priority(size);

        assert!(low <= standard);
        assert!(standard <= high);
    }

    #[test]
    fn test_congestion_multiplier() {
        // Use a larger base fee to show congestion multiplier effect
        let mut estimator = FeeEstimator::new(100); // 100 satoshi base fee
        let base_fee = estimator.estimate_fee(100); // 100 byte tx

        // base_fee = (100 * 100 / 1000) * 1.0 = 10 satoshis

        // Simulate high congestion (2.5x multiplier)
        estimator.congestion_multiplier = 2.5;
        let congested_fee = estimator.estimate_fee(100);
        // congested_fee = (100 * 100 / 1000) * 2.5 = 25 satoshis

        assert!(congested_fee > base_fee);
        assert_eq!(base_fee, 10);
        assert_eq!(congested_fee, 25);
    }

    #[test]
    fn test_acceptable_fee() {
        let estimator = FeeEstimator::new(1);
        assert!(estimator.is_acceptable_fee(10, 250));
        assert!(!estimator.is_acceptable_fee(0, 250));
    }
}
