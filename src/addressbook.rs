//! Address book for managing labeled addresses

use crate::error::ChainError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Address book entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressEntry {
    pub label: String,
    pub address: String,
    pub notes: Option<String>,
    pub added: String,
}

/// Address book
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddressBook {
    pub entries: HashMap<String, AddressEntry>, // key is the label (lowercase)
}

impl AddressBook {
    /// Create a new empty address book
    pub fn new() -> Self {
        AddressBook {
            entries: HashMap::new(),
        }
    }

    /// Add an address to the book
    pub fn add(
        &mut self,
        label: String,
        address: String,
        notes: Option<String>,
    ) -> Result<(), ChainError> {
        let key = label.to_lowercase();

        if self.entries.contains_key(&key) {
            return Err(ChainError::WalletError(format!(
                "Label '{}' already exists",
                label
            )));
        }

        let entry = AddressEntry {
            label: label.clone(),
            address,
            notes,
            added: chrono::Utc::now().to_rfc3339(),
        };

        self.entries.insert(key, entry);
        Ok(())
    }

    /// Remove an address from the book
    pub fn remove(&mut self, label: &str) -> Result<AddressEntry, ChainError> {
        let key = label.to_lowercase();

        self.entries
            .remove(&key)
            .ok_or_else(|| ChainError::WalletError(format!("Label '{}' not found", label)))
    }

    /// Get an address by label
    pub fn get(&self, label: &str) -> Option<&AddressEntry> {
        let key = label.to_lowercase();
        self.entries.get(&key)
    }

    /// Search for addresses (by label or address)
    pub fn search(&self, query: &str) -> Vec<&AddressEntry> {
        let query_lower = query.to_lowercase();

        self.entries
            .values()
            .filter(|entry| {
                entry.label.to_lowercase().contains(&query_lower)
                    || entry.address.to_lowercase().contains(&query_lower)
                    || entry
                        .notes
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect()
    }

    /// List all entries sorted by label
    pub fn list(&self) -> Vec<&AddressEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| a.label.cmp(&b.label));
        entries
    }

    /// Save address book to file
    pub fn save(&self, path: &PathBuf) -> Result<(), ChainError> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            ChainError::WalletError(format!("Failed to serialize address book: {}", e))
        })?;

        fs::write(path, json)
            .map_err(|e| ChainError::WalletError(format!("Failed to write address book: {}", e)))?;

        Ok(())
    }

    /// Load address book from file
    pub fn load(path: &PathBuf) -> Result<Self, ChainError> {
        if !path.exists() {
            return Ok(AddressBook::new());
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| ChainError::WalletError(format!("Failed to read address book: {}", e)))?;

        let book: AddressBook = serde_json::from_str(&contents)
            .map_err(|e| ChainError::WalletError(format!("Failed to parse address book: {}", e)))?;

        Ok(book)
    }
}

/// Get the default address book path
pub fn get_addressbook_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".trinitychain")
        .join("addressbook.json")
}

/// Load the default address book
pub fn load_default() -> Result<AddressBook, ChainError> {
    AddressBook::load(&get_addressbook_path())
}

/// Save the default address book
pub fn save_default(book: &AddressBook) -> Result<(), ChainError> {
    let path = get_addressbook_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| ChainError::WalletError(format!("Failed to create directory: {}", e)))?;
    }

    book.save(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addressbook_add_and_get() {
        let mut book = AddressBook::new();

        book.add(
            "Alice".to_string(),
            "abc123".to_string(),
            Some("Friend".to_string()),
        )
        .unwrap();

        let entry = book.get("alice").unwrap();
        assert_eq!(entry.label, "Alice");
        assert_eq!(entry.address, "abc123");
    }

    #[test]
    fn test_addressbook_remove() {
        let mut book = AddressBook::new();
        book.add("Bob".to_string(), "def456".to_string(), None)
            .unwrap();

        let removed = book.remove("bob").unwrap();
        assert_eq!(removed.label, "Bob");
        assert!(book.get("bob").is_none());
    }

    #[test]
    fn test_addressbook_search() {
        let mut book = AddressBook::new();
        book.add(
            "Alice".to_string(),
            "abc123".to_string(),
            Some("Friend".to_string()),
        )
        .unwrap();
        book.add(
            "Bob".to_string(),
            "def456".to_string(),
            Some("Colleague".to_string()),
        )
        .unwrap();

        let results = book.search("friend");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "Alice");
    }
}
