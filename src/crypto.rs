//! Cryptographic primitives for TrinityChain

use sha2::{Digest, Sha256};
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, ecdsa::Signature};
use rand::rngs::OsRng;
use crate::error::ChainError;

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl KeyPair {
    pub fn generate() -> Result<Self, ChainError> {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        Ok(KeyPair {
            secret_key,
            public_key,
        })
    }
    
    pub fn from_secret_key(secret_key: SecretKey) -> Self {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        KeyPair { secret_key, public_key }
    }

    pub fn from_secret_bytes(bytes: &[u8]) -> Result<Self, ChainError> {
        let secret_key = SecretKey::from_slice(bytes)
            .map_err(|e| ChainError::CryptoError(format!("Invalid secret key bytes: {}", e)))?;

        Ok(Self::from_secret_key(secret_key))
    }
    
    pub fn address(&self) -> String {
        let pubkey_bytes = self.public_key.serialize();
        let mut hasher = Sha256::new();
        hasher.update(&pubkey_bytes);
        format!("{:x}", hasher.finalize())
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key.serialize().to_vec()
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ChainError> {
        let secp = Secp256k1::new();
        
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hash = hasher.finalize();
        
        let message = Message::from_digest_slice(&hash)
            .map_err(|e| ChainError::CryptoError(format!("Invalid message: {}", e)))?;
        
        let signature = secp.sign_ecdsa(&message, &self.secret_key);
        Ok(signature.serialize_compact().to_vec())
    }
}

pub fn verify_signature(
    public_key_bytes: &[u8],
    message: &[u8],
    signature_bytes: &[u8],
) -> Result<bool, ChainError> {
    // Input validation
    if public_key_bytes.is_empty() {
        return Err(ChainError::CryptoError("Public key cannot be empty".to_string()));
    }
    
    if message.is_empty() {
        return Err(ChainError::CryptoError("Message cannot be empty".to_string()));
    }
    
    if signature_bytes.is_empty() {
        return Err(ChainError::CryptoError("Signature cannot be empty".to_string()));
    }

    let secp = Secp256k1::new();
    
    let public_key = PublicKey::from_slice(public_key_bytes)
        .map_err(|e| ChainError::CryptoError(format!("Invalid public key: {}", e)))?;
    
    let mut hasher = Sha256::new();
    hasher.update(message);
    let hash = hasher.finalize();
    
    let message = Message::from_digest_slice(&hash)
        .map_err(|e| ChainError::CryptoError(format!("Invalid message: {}", e)))?;
    
    let signature = Signature::from_compact(signature_bytes)
        .map_err(|e| ChainError::CryptoError(format!("Invalid signature: {}", e)))?;
    
    Ok(secp.verify_ecdsa(&message, &signature, &public_key).is_ok())
}

pub type Address = String;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let keypair = KeyPair::generate().unwrap();
        assert_eq!(keypair.public_key.serialize().len(), 33);
    }
    
    #[test]
    fn test_address_generation() {
        let keypair = KeyPair::generate().unwrap();
        let address = keypair.address();
        assert_eq!(address.len(), 64);
    }
    
    #[test]
    fn test_signing_and_verification() {
        let keypair = KeyPair::generate().unwrap();
        let message = b"Hello, TrinityChain!";
        
        let signature = keypair.sign(message).unwrap();
        let pubkey_bytes = keypair.public_key.serialize();
        
        let is_valid = verify_signature(&pubkey_bytes, message, &signature).unwrap();
        assert!(is_valid);
    }
    
    #[test]
    fn test_invalid_signature() {
        let keypair1 = KeyPair::generate().unwrap();
        let keypair2 = KeyPair::generate().unwrap();
        
        let message = b"Test message";
        let signature = keypair1.sign(message).unwrap();
        let pubkey2_bytes = keypair2.public_key.serialize();
        
        let is_valid = verify_signature(&pubkey2_bytes, message, &signature).unwrap();
        assert!(!is_valid);
    }
    
    #[test]
    fn test_tampered_message() {
        let keypair = KeyPair::generate().unwrap();
        let message = b"Original message";
        let tampered = b"Tampered message";
        
        let signature = keypair.sign(message).unwrap();
        let pubkey_bytes = keypair.public_key.serialize();
        
        let is_valid = verify_signature(&pubkey_bytes, tampered, &signature).unwrap();
        assert!(!is_valid);
    }
}
