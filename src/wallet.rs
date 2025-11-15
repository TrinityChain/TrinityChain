//! Wallet management for TrinityChain
//!
//! Provides functionality for creating, loading, and managing wallets
//! that store keypairs and track triangle ownership.

// Suppress deprecation warnings from aes-gcm's generic-array dependency
#![allow(deprecated)]

use crate::crypto::KeyPair;
use crate::error::ChainError;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Wallet data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Optional wallet name
    pub name: Option<String>,
    /// Wallet address (derived from public key)
    pub address: String,
    /// Secret key (hex-encoded)
    #[serde(rename = "secret_key")]
    pub secret_key_hex: String,
    /// Creation timestamp
    pub created: String,
}

impl Wallet {
    /// Create a new wallet with a generated keypair
    pub fn new(name: Option<String>) -> Result<Self, ChainError> {
        let keypair = KeyPair::generate()?;
        let address = keypair.address();
        let secret_key_hex = hex::encode(keypair.secret_key.secret_bytes());

        Ok(Wallet {
            name,
            address,
            secret_key_hex,
            created: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Load a wallet from a file
    pub fn load(path: &PathBuf) -> Result<Self, ChainError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ChainError::WalletError(format!("Failed to read wallet: {}", e)))?;

        let wallet: Wallet = serde_json::from_str(&contents)
            .map_err(|e| ChainError::WalletError(format!("Failed to parse wallet: {}", e)))?;

        Ok(wallet)
    }

    /// Save the wallet to a file
    pub fn save(&self, path: &PathBuf) -> Result<(), ChainError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ChainError::WalletError(format!("Failed to serialize wallet: {}", e)))?;

        fs::write(path, json)
            .map_err(|e| ChainError::WalletError(format!("Failed to write wallet: {}", e)))?;

        Ok(())
    }

    /// Get the keypair from the wallet
    pub fn get_keypair(&self) -> Result<KeyPair, ChainError> {
        let secret_bytes = hex::decode(&self.secret_key_hex)
            .map_err(|e| ChainError::WalletError(format!("Failed to decode secret key: {}", e)))?;

        KeyPair::from_secret_bytes(&secret_bytes)
    }
}

/// Get the default wallet directory
pub fn get_wallet_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".trinitychain")
}

/// Get the default wallet file path
pub fn get_default_wallet_path() -> PathBuf {
    get_wallet_dir().join("wallet.json")
}

/// Get a named wallet file path
pub fn get_named_wallet_path(name: &str) -> PathBuf {
    get_wallet_dir().join(format!("wallet_{}.json", name))
}

/// Create the wallet directory if it doesn't exist
pub fn ensure_wallet_dir() -> Result<(), ChainError> {
    let wallet_dir = get_wallet_dir();
    fs::create_dir_all(&wallet_dir)
        .map_err(|e| ChainError::WalletError(format!("Failed to create wallet directory: {}", e)))?;
    Ok(())
}

/// Create a new wallet and save it to the default location
pub fn create_default_wallet() -> Result<Wallet, ChainError> {
    ensure_wallet_dir()?;

    let path = get_default_wallet_path();

    if path.exists() {
        return Err(ChainError::WalletError(
            "Wallet already exists at default location".to_string()
        ));
    }

    let wallet = Wallet::new(None)?;
    wallet.save(&path)?;

    Ok(wallet)
}

/// Create a named wallet
pub fn create_named_wallet(name: &str) -> Result<Wallet, ChainError> {
    ensure_wallet_dir()?;

    let path = get_named_wallet_path(name);

    if path.exists() {
        return Err(ChainError::WalletError(
            format!("Wallet '{}' already exists", name)
        ));
    }

    let wallet = Wallet::new(Some(name.to_string()))?;
    wallet.save(&path)?;

    Ok(wallet)
}

/// Load the default wallet
pub fn load_default_wallet() -> Result<Wallet, ChainError> {
    let path = get_default_wallet_path();

    if !path.exists() {
        return Err(ChainError::WalletError(
            "No wallet found. Run 'trinity-wallet new' first.".to_string()
        ));
    }

    Wallet::load(&path)
}

/// Load a named wallet
pub fn load_named_wallet(name: &str) -> Result<Wallet, ChainError> {
    let path = get_named_wallet_path(name);

    if !path.exists() {
        return Err(ChainError::WalletError(
            format!("Wallet '{}' not found", name)
        ));
    }

    Wallet::load(&path)
}

/// List all available wallets in the wallet directory
pub fn list_wallets() -> Result<Vec<String>, ChainError> {
    let wallet_dir = get_wallet_dir();

    if !wallet_dir.exists() {
        return Ok(Vec::new());
    }

    let mut wallets = Vec::new();

    let entries = fs::read_dir(&wallet_dir)
        .map_err(|e| ChainError::WalletError(format!("Failed to read wallet directory: {}", e)))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| ChainError::WalletError(format!("Failed to read directory entry: {}", e)))?;

        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                wallets.push(filename.to_string());
            }
        }
    }

    wallets.sort();
    Ok(wallets)
}

// ============================================================================
// Wallet Encryption/Decryption
// ============================================================================

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString};

/// Encrypted wallet structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedWallet {
    pub name: Option<String>,
    pub address: String,
    pub encrypted_secret_key: String,  // Base64 encoded encrypted data
    pub salt: String,  // Base64 encoded salt
    pub nonce: String, // Base64 encoded nonce
    pub created: String,
}

impl EncryptedWallet {
    /// Encrypt a wallet with a password
    pub fn from_wallet(wallet: &Wallet, password: &str) -> Result<Self, ChainError> {
        use argon2::PasswordHasher;
        use argon2::password_hash::SaltString;

        // Generate a random salt
        let salt = SaltString::generate(&mut OsRng);

        // Derive encryption key from password using Argon2
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| ChainError::CryptoError(format!("Password hashing failed: {}", e)))?;

        // Extract the hash bytes for encryption key
        let hash_bytes = password_hash.hash
            .ok_or_else(|| ChainError::CryptoError("No hash generated".to_string()))?;
        let key_bytes = hash_bytes.as_bytes();

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key_bytes[..32])
            .map_err(|e| ChainError::CryptoError(format!("Failed to create cipher: {}", e)))?;

        // Generate a random nonce
        use rand::RngCore;
        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the secret key
        let secret_bytes = wallet.secret_key_hex.as_bytes();
        let ciphertext = cipher
            .encrypt(&nonce, secret_bytes)
            .map_err(|e| ChainError::CryptoError(format!("Encryption failed: {}", e)))?;

        use base64::{Engine as _, engine::general_purpose};

        Ok(EncryptedWallet {
            name: wallet.name.clone(),
            address: wallet.address.clone(),
            encrypted_secret_key: general_purpose::STANDARD.encode(&ciphertext),
            salt: salt.to_string(),
            nonce: general_purpose::STANDARD.encode(&nonce_bytes),
            created: wallet.created.clone(),
        })
    }

    /// Decrypt the wallet using a password
    pub fn decrypt(&self, password: &str) -> Result<Wallet, ChainError> {


        // Parse the stored salt
        let salt = SaltString::from_b64(&self.salt)
            .map_err(|e| ChainError::CryptoError(format!("Invalid salt: {}", e)))?;

        // Derive the same key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| ChainError::CryptoError(format!("Password hashing failed: {}", e)))?;

        let hash_bytes = password_hash.hash
            .ok_or_else(|| ChainError::CryptoError("No hash generated".to_string()))?;
        let key_bytes = hash_bytes.as_bytes();

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key_bytes[..32])
            .map_err(|e| ChainError::CryptoError(format!("Failed to create cipher: {}", e)))?;

        // Decode nonce and ciphertext
        use base64::{Engine as _, engine::general_purpose};

        let nonce_bytes = general_purpose::STANDARD.decode(&self.nonce)
            .map_err(|e| ChainError::CryptoError(format!("Invalid nonce: {}", e)))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = general_purpose::STANDARD.decode(&self.encrypted_secret_key)
            .map_err(|e| ChainError::CryptoError(format!("Invalid ciphertext: {}", e)))?;

        // Decrypt
        let plaintext = cipher
            .decrypt(&nonce, ciphertext.as_ref())
            .map_err(|_| ChainError::CryptoError("Decryption failed - wrong password?".to_string()))?;

        let secret_key_hex = String::from_utf8(plaintext)
            .map_err(|e| ChainError::CryptoError(format!("Invalid UTF-8: {}", e)))?;

        Ok(Wallet {
            name: self.name.clone(),
            address: self.address.clone(),
            secret_key_hex,
            created: self.created.clone(),
        })
    }

    /// Save encrypted wallet to file
    pub fn save(&self, path: &PathBuf) -> Result<(), ChainError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ChainError::WalletError(format!("Failed to serialize wallet: {}", e)))?;

        fs::write(path, json)
            .map_err(|e| ChainError::WalletError(format!("Failed to write wallet: {}", e)))?;

        // Set file permissions to owner-only (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .map_err(|e| ChainError::WalletError(format!("Failed to get file metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o600); // rw-------
            fs::set_permissions(path, perms)
                .map_err(|e| ChainError::WalletError(format!("Failed to set file permissions: {}", e)))?;
        }

        Ok(())
    }

    /// Load encrypted wallet from file
    pub fn load(path: &PathBuf) -> Result<Self, ChainError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ChainError::WalletError(format!("Failed to read wallet: {}", e)))?;

        let wallet: EncryptedWallet = serde_json::from_str(&contents)
            .map_err(|e| ChainError::WalletError(format!("Failed to parse encrypted wallet: {}", e)))?;

        Ok(wallet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new(Some("test".to_string())).unwrap();
        assert_eq!(wallet.name, Some("test".to_string()));
        assert!(!wallet.address.is_empty());
        assert!(!wallet.secret_key_hex.is_empty());
    }

    #[test]
    fn test_wallet_keypair_recovery() {
        let wallet = Wallet::new(None).unwrap();
        let keypair = wallet.get_keypair().unwrap();
        assert_eq!(wallet.address, keypair.address());
    }

    #[test]
    fn test_wallet_save_and_load() {
        let temp_dir = std::env::temp_dir();
        let wallet_path = temp_dir.join("test_wallet.json");

        // Clean up if exists
        let _ = fs::remove_file(&wallet_path);

        // Create and save
        let wallet = Wallet::new(Some("test_save".to_string())).unwrap();
        wallet.save(&wallet_path).unwrap();

        // Load and verify
        let loaded = Wallet::load(&wallet_path).unwrap();
        assert_eq!(wallet.address, loaded.address);
        assert_eq!(wallet.secret_key_hex, loaded.secret_key_hex);
        assert_eq!(wallet.name, loaded.name);

        // Cleanup
        fs::remove_file(&wallet_path).unwrap();
    }
}
