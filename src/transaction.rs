//! Transaction types for TrinityChain

use sha2::{Digest, Sha256};
use crate::blockchain::{Sha256Hash, TriangleState};
use crate::geometry::Triangle;
use crate::error::ChainError;

pub type Address = String;

/// A transaction that can occur in a block
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Transaction {
    Transfer(TransferTx),
    Subdivision(SubdivisionTx),
    Coinbase(CoinbaseTx),
}

impl Transaction {
    pub fn hash_str(&self) -> String {
        hex::encode(self.hash())
    }

    /// Get the fee for this transaction
    pub fn fee(&self) -> u64 {
        match self {
            Transaction::Subdivision(tx) => tx.fee,
            Transaction::Transfer(tx) => tx.fee,
            Transaction::Coinbase(_) => 0, // Coinbase has no fee
        }
    }

    /// Calculate the hash of this transaction
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        match self {
            Transaction::Subdivision(tx) => {
                hasher.update(tx.parent_hash);
                for child in &tx.children {
                    hasher.update(child.hash());
                }
                hasher.update(tx.owner_address.as_bytes());
                hasher.update(tx.fee.to_le_bytes());
                hasher.update(tx.nonce.to_le_bytes());
            }
            Transaction::Coinbase(tx) => {
                hasher.update("coinbase".as_bytes());
                hasher.update(tx.reward_area.to_le_bytes());
                hasher.update(tx.beneficiary_address.as_bytes());
            }
            Transaction::Transfer(tx) => {
                hasher.update("transfer".as_bytes());
                hasher.update(tx.input_hash);
                hasher.update(tx.new_owner.as_bytes());
                hasher.update(tx.sender.as_bytes());
                hasher.update(tx.fee.to_le_bytes());
                hasher.update(tx.nonce.to_le_bytes());
            }
        };
        hasher.finalize().into()
    }

    /// Validate this transaction against the current UTXO state
    pub fn validate(&self, state: &TriangleState) -> Result<(), ChainError> {
        match self {
            Transaction::Subdivision(tx) => tx.validate(state),
            Transaction::Coinbase(tx) => tx.validate(),
            Transaction::Transfer(tx) => tx.validate(),
        }
    }
}

/// Subdivision transaction: splits one parent triangle into three children
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubdivisionTx {
    pub parent_hash: Sha256Hash,
    pub children: Vec<Triangle>,
    pub owner_address: Address,
    pub fee: u64,
    pub nonce: u64,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
}

impl SubdivisionTx {
    pub fn new(
        parent_hash: Sha256Hash,
        children: Vec<Triangle>,
        owner_address: Address,
        fee: u64,
        nonce: u64,
    ) -> Self {
        SubdivisionTx {
            parent_hash,
            children,
            owner_address,
            fee,
            nonce,
            signature: None,
            public_key: None,
        }
    }

    pub fn signable_message(&self) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(&self.parent_hash);
        for child in &self.children {
            message.extend_from_slice(&child.hash());
        }
        message.extend_from_slice(self.owner_address.as_bytes());
        message.extend_from_slice(&self.fee.to_le_bytes());
        message.extend_from_slice(&self.nonce.to_le_bytes());
        message
    }

    pub fn sign(&mut self, signature: Vec<u8>, public_key: Vec<u8>) {
        self.signature = Some(signature);
        self.public_key = Some(public_key);
    }

    /// Validates just the signature of the transaction, without access to blockchain state.
    /// This is useful for early validation in the mempool.
    pub fn validate_signature(&self) -> Result<(), ChainError> {
        if self.signature.is_none() || self.public_key.is_none() {
            return Err(ChainError::InvalidTransaction(
                "Transaction not signed".to_string(),
            ));
        }

        let message = self.signable_message();
        let is_valid = crate::crypto::verify_signature(
            self.public_key.as_ref().unwrap(),
            &message,
            self.signature.as_ref().unwrap(),
        )?;

        if !is_valid {
            return Err(ChainError::InvalidTransaction(
                "Invalid signature".to_string(),
            ));
        }

        Ok(())
    }

    /// Performs a full validation of the transaction against the current blockchain state.
    pub fn validate(&self, state: &TriangleState) -> Result<(), ChainError> {
        // First, perform a stateless signature check.
        self.validate_signature()?;

        // Then, validate against the current state (UTXO set).
        if !state.utxo_set.contains_key(&self.parent_hash) {
            return Err(ChainError::TriangleNotFound(format!(
                "Parent triangle {} not found in UTXO set",
                hex::encode(self.parent_hash)
            )));
        }

        let parent = state.utxo_set.get(&self.parent_hash).unwrap();
        let expected_children = parent.subdivide();

        if self.children.len() != 3 {
            return Err(ChainError::InvalidTransaction(
                "Subdivision must produce exactly 3 children".to_string(),
            ));
        }

        for (i, child) in self.children.iter().enumerate() {
            let expected = &expected_children[i];
            if !child.a.equals(&expected.a) ||
               !child.b.equals(&expected.b) ||
               !child.c.equals(&expected.c) {
                return Err(ChainError::InvalidTransaction(format!(
                    "Child {} geometry does not match expected subdivision",
                    i
                )));
            }
        }

        Ok(())
    }
}

/// Coinbase transaction: miner reward
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoinbaseTx {
    pub reward_area: u64,
    pub beneficiary_address: Address,
}

impl CoinbaseTx {
    /// Maximum reward area that can be claimed in a coinbase transaction
    pub const MAX_REWARD_AREA: u64 = 1000;

    pub fn validate(&self) -> Result<(), ChainError> {
        // Validate reward area is within acceptable bounds
        if self.reward_area == 0 {
            return Err(ChainError::InvalidTransaction(
                "Coinbase reward area must be greater than zero".to_string()
            ));
        }

        if self.reward_area > Self::MAX_REWARD_AREA {
            return Err(ChainError::InvalidTransaction(
                format!("Coinbase reward area {} exceeds maximum {}",
                    self.reward_area, Self::MAX_REWARD_AREA)
            ));
        }

        // Validate beneficiary address is not empty
        if self.beneficiary_address.is_empty() {
            return Err(ChainError::InvalidTransaction(
                "Coinbase beneficiary address cannot be empty".to_string()
            ));
        }

        Ok(())
    }
}

/// Transfer transaction - moves ownership of a triangle
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransferTx {
    pub input_hash: Sha256Hash,
    pub new_owner: Address,
    pub sender: Address,
    pub fee: u64,
    pub nonce: u64,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
    #[serde(default)]
    pub memo: Option<String>,
}

impl TransferTx {
    /// Maximum memo length (256 characters)
    pub const MAX_MEMO_LENGTH: usize = 256;

    pub fn new(input_hash: Sha256Hash, new_owner: Address, sender: Address, fee: u64, nonce: u64) -> Self {
        TransferTx {
            input_hash,
            new_owner,
            sender,
            fee,
            nonce,
            signature: None,
            public_key: None,
            memo: None,
        }
    }

    pub fn with_memo(mut self, memo: String) -> Result<Self, ChainError> {
        if memo.len() > Self::MAX_MEMO_LENGTH {
            return Err(ChainError::InvalidTransaction(
                format!("Memo exceeds maximum length of {} characters", Self::MAX_MEMO_LENGTH)
            ));
        }
        self.memo = Some(memo);
        Ok(self)
    }
    
    pub fn signable_message(&self) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice("TRANSFER:".as_bytes());
        message.extend_from_slice(&self.input_hash);
        message.extend_from_slice(self.new_owner.as_bytes());
        message.extend_from_slice(self.sender.as_bytes());
        message.extend_from_slice(&self.fee.to_le_bytes());
        message.extend_from_slice(&self.nonce.to_le_bytes());
        message
    }
    
    pub fn sign(&mut self, signature: Vec<u8>, public_key: Vec<u8>) {
        self.signature = Some(signature);
        self.public_key = Some(public_key);
    }
    
    pub fn validate(&self) -> Result<(), ChainError> {
        if self.signature.is_none() || self.public_key.is_none() {
            return Err(ChainError::InvalidTransaction("Transfer not signed".to_string()));
        }

        // Validate memo length to prevent DoS attacks
        if let Some(ref memo) = self.memo {
            if memo.len() > Self::MAX_MEMO_LENGTH {
                return Err(ChainError::InvalidTransaction(
                    format!("Memo exceeds maximum length of {} characters", Self::MAX_MEMO_LENGTH)
                ));
            }
        }

        let message = self.signable_message();
        let is_valid = crate::crypto::verify_signature(
            self.public_key.as_ref().unwrap(),
            &message,
            self.signature.as_ref().unwrap(),
        )?;

        if !is_valid {
            return Err(ChainError::InvalidTransaction("Invalid signature".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::TriangleState;
    use crate::crypto::KeyPair;
    use crate::geometry::{Point, Triangle};

    #[test]
    fn test_tx_validation_success() {
        let mut state = TriangleState::new();
        let parent = Triangle::new(
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.5, y: 0.866 },
            None,
            "test_owner".to_string(),
        );
        let parent_hash = parent.hash();
        state.utxo_set.insert(parent_hash, parent.clone());

        let children = parent.subdivide();
        let keypair = KeyPair::generate().unwrap();
        let address = keypair.address();

        let mut tx = SubdivisionTx::new(parent_hash, children.to_vec(), address, 0, 1);
        let message = tx.signable_message();
        let signature = keypair.sign(&message).unwrap();
        let public_key = keypair.public_key.serialize().to_vec();
        tx.sign(signature, public_key);

        assert!(tx.validate(&state).is_ok());
    }

    #[test]
    fn test_unsigned_transaction_fails() {
        let mut state = TriangleState::new();
        let parent = Triangle::new(
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.5, y: 0.866 },
            None,
            "test_owner".to_string(),
        );
        let parent_hash = parent.hash();
        state.utxo_set.insert(parent_hash, parent.clone());

        let children = parent.subdivide();
        let address = "test_address".to_string();

        let tx = SubdivisionTx::new(parent_hash, children.to_vec(), address, 0, 1);
        assert!(tx.validate(&state).is_err());
    }

    #[test]
    fn test_invalid_signature_fails() {
        let mut state = TriangleState::new();
        let parent = Triangle::new(
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.5, y: 0.866 },
            None,
            "test_owner".to_string(),
        );
        let parent_hash = parent.hash();
        state.utxo_set.insert(parent_hash, parent.clone());

        let children = parent.subdivide();
        let keypair = KeyPair::generate().unwrap();
        let address = keypair.address();

        let mut tx = SubdivisionTx::new(parent_hash, children.to_vec(), address, 0, 1);
        let fake_signature = vec![0u8; 64];
        let public_key = keypair.public_key.serialize().to_vec();
        tx.sign(fake_signature, public_key);

        assert!(tx.validate(&state).is_err());
    }

    #[test]
    fn test_tx_validation_area_conservation_failure() {
        let mut state = TriangleState::new();
        let parent = Triangle::new(
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.5, y: 0.866 },
            None,
            "test_owner".to_string(),
        );
        let parent_hash = parent.hash();
        state.utxo_set.insert(parent_hash, parent);

        let bad_child = Triangle::new(
            Point { x: 0.0, y: 0.0 },
            Point { x: 2.0, y: 0.0 },
            Point { x: 1.0, y: 1.732 },
            None,
            "test_owner".to_string(),
        );
        let children = vec![bad_child.clone(), bad_child.clone(), bad_child];

        let keypair = KeyPair::generate().unwrap();
        let address = keypair.address();

        let tx = SubdivisionTx::new(parent_hash, children, address, 0, 1);
        assert!(tx.validate(&state).is_err());
    }

    #[test]
    fn test_tx_validation_double_spend_check() {
        let state = TriangleState::new();

        let parent = Triangle::new(
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.5, y: 0.866 },
            None,
            "test_owner".to_string(),
        );
        let parent_hash = parent.hash();
        let children = parent.subdivide();

        let address = "test_address".to_string();
        let tx = SubdivisionTx::new(parent_hash, children.to_vec(), address, 0, 1);

        assert!(tx.validate(&state).is_err());
    }
}
