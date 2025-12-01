//! Address book for managing labeled addresses with production-grade features
//!
//! This module provides a thread-safe, validated address book with atomic operations,
//! audit trails, and comprehensive error handling.

use crate::error::ChainError;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
<<<<<<< HEAD
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Constants for validation
const MAX_LABEL_LENGTH: usize = 64;
const MAX_ADDRESS_LENGTH: usize = 128;
const MAX_NOTES_LENGTH: usize = 512;
const MAX_ENTRIES: usize = 10_000;
const BACKUP_SUFFIX: &str = ".backup";

/// Address book entry with audit trail
=======
use std::collections::{hash_map::Entry, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

/// Address book entry
>>>>>>> 310f51a (feat: Resolve compilation errors)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressEntry {
    /// Display label for the address (case-preserved)
    pub label: String,

    /// TrinityChain address
    pub address: String,

    /// Optional notes about this contact
    pub notes: Option<String>,

    /// RFC3339 timestamp when entry was created
    pub created_at: String,

    /// RFC3339 timestamp of last modification
    pub updated_at: String,

    /// Number of times this entry has been modified
    pub version: u32,
}

impl AddressEntry {
    /// Create a new address entry with validation
    fn new(label: String, address: String, notes: Option<String>) -> Result<Self, ChainError> {
        // Validate inputs
        validate_label(&label)?;
        validate_address(&address)?;
        if let Some(ref n) = notes {
            validate_notes(n)?;
        }

        let now = chrono::Utc::now().to_rfc3339();

        Ok(AddressEntry {
            label,
            address,
            notes,
            created_at: now.clone(),
            updated_at: now,
            version: 1,
        })
    }

    /// Update the entry with new information
    fn update(&mut self, address: Option<String>, notes: Option<String>) -> Result<(), ChainError> {
        if let Some(addr) = address {
            validate_address(&addr)?;
            self.address = addr;
        }

        if let Some(n) = notes {
            validate_notes(&n)?;
            self.notes = Some(n);
        }

        self.updated_at = chrono::Utc::now().to_rfc3339();
        self.version = self.version.saturating_add(1);
        Ok(())
    }
}

/// Thread-safe address book with atomic operations
#[derive(Debug, Clone)]
pub struct AddressBook {
<<<<<<< HEAD
    inner: Arc<RwLock<AddressBookInner>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddressBookInner {
    /// Entries keyed by lowercase label for case-insensitive lookup
    entries: HashMap<String, AddressEntry>,

    /// Reverse index: address -> label (for quick address lookups)
    #[serde(skip)]
    address_index: HashMap<String, String>,

    /// Metadata about the address book
    metadata: AddressBookMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddressBookMetadata {
    version: String,
    created_at: String,
    last_modified: String,
}

impl Default for AddressBookInner {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        AddressBookInner {
            entries: HashMap::new(),
            address_index: HashMap::new(),
            metadata: AddressBookMetadata {
                version: "1.0.0".to_string(),
                created_at: now.clone(),
                last_modified: now,
            },
        }
    }
}

impl AddressBookInner {
    /// Rebuild the address index from entries
    fn rebuild_index(&mut self) {
        self.address_index.clear();
        for (key, entry) in &self.entries {
            self.address_index
                .insert(entry.address.clone(), key.clone());
        }
    }

    /// Update modification timestamp
    fn touch(&mut self) {
        self.metadata.last_modified = chrono::Utc::now().to_rfc3339();
    }
=======
    /// Stores entries mapped by lowercase label for case-insensitive lookup
    pub entries: HashMap<String, AddressEntry>,
>>>>>>> 310f51a (feat: Resolve compilation errors)
}

impl AddressBook {
    /// Create a new empty address book
    pub fn new() -> Self {
<<<<<<< HEAD
        AddressBook {
            inner: Arc::new(RwLock::new(AddressBookInner::default())),
        }
    }

    /// Add an address to the book
    ///
    /// Returns an error if the label already exists or validation fails.
=======
        Self::default()
    }

    /// Add an address to the book. Trims whitespace from inputs.
>>>>>>> 310f51a (feat: Resolve compilation errors)
    pub fn add(
        &self,
        label: String,
        address: String,
        notes: Option<String>,
    ) -> Result<(), ChainError> {
<<<<<<< HEAD
        let mut inner = self.inner.write();

        // Check size limit
        if inner.entries.len() >= MAX_ENTRIES {
            return Err(ChainError::WalletError(format!(
                "Address book is full (max {} entries)",
                MAX_ENTRIES
            )));
        }

        let key = label.to_lowercase();

        // Check for duplicate label
        if inner.entries.contains_key(&key) {
            return Err(ChainError::WalletError(format!(
                "Label '{}' already exists",
                label
            )));
        }

        // Check for duplicate address
        if inner.address_index.contains_key(&address) {
            let existing_label = inner.address_index.get(&address).unwrap();
            let existing_entry = inner.entries.get(existing_label).unwrap();
            return Err(ChainError::WalletError(format!(
                "Address already exists with label '{}'",
                existing_entry.label
            )));
        }

        let entry = AddressEntry::new(label, address.clone(), notes)?;

        // Update indices
        inner.address_index.insert(address, key.clone());
        inner.entries.insert(key, entry);
        inner.touch();

        Ok(())
    }

    /// Remove an address from the book
    pub fn remove(&self, label: &str) -> Result<AddressEntry, ChainError> {
        let mut inner = self.inner.write();
        let key = label.to_lowercase();

        let entry = inner
            .entries
=======
        let label_clean = label.trim();
        let address_clean = address.trim();

        if label_clean.is_empty() || address_clean.is_empty() {
            return Err(ChainError::WalletError(
                "Label and address cannot be empty".to_string(),
            ));
        }

        let key = label_clean.to_lowercase();
        
        // Use Entry API to avoid double hashing (checking existence then inserting)
        match self.entries.entry(key) {
            Entry::Occupied(_) => Err(ChainError::WalletError(format!(
                "Label '{}' already exists",
                label_clean
            ))),
            Entry::Vacant(v) => {
                v.insert(AddressEntry {
                    label: label_clean.to_string(),
                    address: address_clean.to_string(),
                    notes: notes.filter(|n| !n.trim().is_empty()).map(|n| n.trim().to_string()),
                    added: chrono::Utc::now().to_rfc3339(),
                });
                Ok(())
            }
        }
    }

    /// Remove an address from the book
    pub fn remove(&mut self, label: &str) -> Result<AddressEntry, ChainError> {
        let key = label.trim().to_lowercase();
        self.entries
>>>>>>> 310f51a (feat: Resolve compilation errors)
            .remove(&key)
            .ok_or_else(|| ChainError::WalletError(format!("Label '{}' not found", label)))?;

        // Update address index
        inner.address_index.remove(&entry.address);
        inner.touch();

        Ok(entry)
    }

    /// Update an existing entry
    pub fn update(
        &self,
        label: &str,
        new_address: Option<String>,
        new_notes: Option<String>,
    ) -> Result<(), ChainError> {
        let mut inner = self.inner.write();
        let key = label.to_lowercase();

        // First, check if the entry exists and get the old address
        let old_address = inner
            .entries
            .get(&key)
            .ok_or_else(|| ChainError::WalletError(format!("Label '{}' not found", label)))?
            .address
            .clone();

        // Check for duplicate address if updating address (before getting mutable reference)
        if let Some(ref new_addr) = new_address {
            if new_addr != &old_address && inner.address_index.contains_key(new_addr) {
                let existing_label = inner.address_index.get(new_addr).unwrap();
                let existing_entry = inner.entries.get(existing_label).unwrap();
                return Err(ChainError::WalletError(format!(
                    "Address already exists with label '{}'",
                    existing_entry.label
                )));
            }
        }

        // Now we can safely get the mutable reference
        let entry = inner.entries.get_mut(&key).unwrap(); // Safe: we checked existence above

        entry.update(new_address.clone(), new_notes)?;

        // Update address index if address changed
        if let Some(new_addr) = new_address {
            inner.address_index.remove(&old_address);
            inner.address_index.insert(new_addr, key);
        }

        inner.touch();
        Ok(())
    }

    /// Get an address by label
<<<<<<< HEAD
    pub fn get(&self, label: &str) -> Option<AddressEntry> {
        let inner = self.inner.read();
        let key = label.to_lowercase();
        inner.entries.get(&key).cloned()
    }

    /// Get an entry by address (reverse lookup)
    pub fn get_by_address(&self, address: &str) -> Option<AddressEntry> {
        let inner = self.inner.read();
        inner
            .address_index
            .get(address)
            .and_then(|key| inner.entries.get(key).cloned())
    }

    /// Search for addresses (by label, address, or notes)
    pub fn search(&self, query: &str) -> Vec<AddressEntry> {
        let inner = self.inner.read();
        let query_lower = query.to_lowercase();
=======
    pub fn get(&self, label: &str) -> Option<&AddressEntry> {
        self.entries.get(&label.trim().to_lowercase())
    }

    /// Search for addresses (by label, address, or notes)
    pub fn search(&self, query: &str) -> Vec<&AddressEntry> {
        let query_trimmed = query.trim();
        if query_trimmed.is_empty() {
            return self.list();
        }

        let query_lower = query_trimmed.to_lowercase();
>>>>>>> 310f51a (feat: Resolve compilation errors)

        let mut results: Vec<_> = inner
            .entries
            .values()
            .filter(|entry| {
                entry.label.to_lowercase().contains(&query_lower)
                    || entry.address.to_lowercase().contains(&query_lower)
                    || entry
                        .notes
                        .as_deref()
                        .map_or(false, |n| n.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        results.sort_by(|a, b| a.label.cmp(&b.label));
        results
    }

    /// List all entries sorted by label
<<<<<<< HEAD
    pub fn list(&self) -> Vec<AddressEntry> {
        let inner = self.inner.read();
        let mut entries: Vec<_> = inner.entries.values().cloned().collect();
        entries.sort_by(|a, b| a.label.cmp(&b.label));
        entries
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        let inner = self.inner.read();
        inner.entries.len()
    }

    /// Check if the address book is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if a label exists
    pub fn contains_label(&self, label: &str) -> bool {
        let inner = self.inner.read();
        inner.entries.contains_key(&label.to_lowercase())
    }

    /// Check if an address exists
    pub fn contains_address(&self, address: &str) -> bool {
        let inner = self.inner.read();
        inner.address_index.contains_key(address)
    }

    /// Save address book to file with atomic write and backup
    pub fn save(&self, path: &Path) -> Result<(), ChainError> {
        let inner = self.inner.read();

        // Create backup if file exists
        if path.exists() {
            let backup_path = path.with_extension(format!("json{}", BACKUP_SUFFIX));
            fs::copy(path, &backup_path)
                .map_err(|e| ChainError::WalletError(format!("Failed to create backup: {}", e)))?;
        }

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&*inner).map_err(|e| {
            ChainError::WalletError(format!("Failed to serialize address book: {}", e))
        })?;

        // Atomic write using temporary file
        let temp_path = path.with_extension("tmp");
        let mut file = File::create(&temp_path)
            .map_err(|e| ChainError::WalletError(format!("Failed to create temp file: {}", e)))?;

        file.write_all(json.as_bytes())
            .map_err(|e| ChainError::WalletError(format!("Failed to write address book: {}", e)))?;
=======
    pub fn list(&self) -> Vec<&AddressEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        // Use localized collation if strict alphabetic order is needed, 
        // but standard strict comparison is faster for general use.
        entries.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
        entries
    }

    /// Save address book to file using atomic write pattern
    pub fn save(&self, path: &Path) -> Result<(), ChainError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ChainError::WalletError(format!("Failed to create directory '{}': {}", parent.display(), e))
            })?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| {
            ChainError::WalletError(format!("Failed to serialize address book: {}", e))
        })?;

        // Write to a temp file in the same directory to ensure atomic rename
        let tmp_path = path.with_extension("tmp");
        
        fs::write(&tmp_path, json).map_err(|e| {
            ChainError::WalletError(format!("Failed to write temporary file: {}", e))
        })?;

        fs::rename(&tmp_path, path).map_err(|e| {
            // Attempt cleanup on failure, ignore cleanup errors
            let _ = fs::remove_file(&tmp_path);
            ChainError::WalletError(format!("Failed to commit address book file: {}", e))
        })?;
>>>>>>> 310f51a (feat: Resolve compilation errors)

        file.sync_all()
            .map_err(|e| ChainError::WalletError(format!("Failed to sync file: {}", e)))?;

        drop(file);

        // Atomic rename
        fs::rename(&temp_path, path)
            .map_err(|e| ChainError::WalletError(format!("Failed to finalize write: {}", e)))?;

        Ok(())
    }

<<<<<<< HEAD
    /// Load address book from file
    pub fn load(path: &Path) -> Result<Self, ChainError> {
        if !path.exists() {
            return Ok(AddressBook::new());
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| ChainError::WalletError(format!("Failed to read address book: {}", e)))?;

        let mut inner: AddressBookInner = serde_json::from_str(&contents)
            .map_err(|e| ChainError::WalletError(format!("Failed to parse address book: {}", e)))?;

        // Rebuild address index
        inner.rebuild_index();

        // Validate all entries
        for entry in inner.entries.values() {
            validate_label(&entry.label)?;
            validate_address(&entry.address)?;
            if let Some(ref notes) = entry.notes {
                validate_notes(notes)?;
            }
        }

        Ok(AddressBook {
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    /// Export address book to CSV format
    pub fn export_csv(&self, path: &Path) -> Result<(), ChainError> {
        let inner = self.inner.read();
        let mut entries: Vec<_> = inner.entries.values().collect();
        entries.sort_by(|a, b| a.label.cmp(&b.label));

        let mut csv = String::from("Label,Address,Notes,Created,Updated,Version\n");

        for entry in entries {
            let notes = entry.notes.as_deref().unwrap_or("");
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{}\n",
                entry.label.replace('"', "\"\""),
                entry.address,
                notes.replace('"', "\"\""),
                entry.created_at,
                entry.updated_at,
                entry.version
            ));
        }

        fs::write(path, csv)
            .map_err(|e| ChainError::WalletError(format!("Failed to export CSV: {}", e)))?;

        Ok(())
    }

    /// Clear all entries (use with caution!)
    pub fn clear(&self) -> Result<(), ChainError> {
        let mut inner = self.inner.write();
        inner.entries.clear();
        inner.address_index.clear();
        inner.touch();
        Ok(())
    }
}

impl Default for AddressBook {
    fn default() -> Self {
        Self::new()
    }
}

// Validation functions

fn validate_label(label: &str) -> Result<(), ChainError> {
    if label.is_empty() {
        return Err(ChainError::WalletError("Label cannot be empty".to_string()));
    }

    if label.len() > MAX_LABEL_LENGTH {
        return Err(ChainError::WalletError(format!(
            "Label too long (max {} characters)",
            MAX_LABEL_LENGTH
        )));
    }

    // Check for valid characters (alphanumeric, spaces, basic punctuation)
    if !label
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.,()[]{}".contains(c))
    {
        return Err(ChainError::WalletError(
            "Label contains invalid characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_address(address: &str) -> Result<(), ChainError> {
    if address.is_empty() {
        return Err(ChainError::WalletError(
            "Address cannot be empty".to_string(),
        ));
    }

    if address.len() > MAX_ADDRESS_LENGTH {
        return Err(ChainError::WalletError(format!(
            "Address too long (max {} characters)",
            MAX_ADDRESS_LENGTH
        )));
    }

    // Basic format validation (adjust for TrinityChain address format)
    if !address.chars().all(|c| c.is_alphanumeric()) {
        return Err(ChainError::WalletError(
            "Invalid address format".to_string(),
        ));
    }

    Ok(())
}

fn validate_notes(notes: &str) -> Result<(), ChainError> {
    if notes.len() > MAX_NOTES_LENGTH {
        return Err(ChainError::WalletError(format!(
            "Notes too long (max {} characters)",
            MAX_NOTES_LENGTH
        )));
    }

    Ok(())
}

// Helper functions for default paths

/// Get the default address book path
=======
    /// Load address book from a specific path
    pub fn load(path: &Path) -> Result<Self, ChainError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ChainError::WalletError(format!("Failed to read address book: {}", e)))?;

        serde_json::from_str(&contents)
            .map_err(|e| ChainError::WalletError(format!("Failed to parse address book: {}", e)))
    }

    /// Load from path, or return new empty book if file doesn't exist
    pub fn load_or_new(path: &Path) -> Result<Self, ChainError> {
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::new())
        }
    }
}

/// Get the default address book path with cross-platform support
>>>>>>> 310f51a (feat: Resolve compilation errors)
pub fn get_addressbook_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".trinitychain")
        .join("addressbook.json")
}

/// Load the default address book, creating a new one if missing
pub fn load_default() -> Result<AddressBook, ChainError> {
    AddressBook::load_or_new(&get_addressbook_path())
}

/// Save to the default address book location
pub fn save_default(book: &AddressBook) -> Result<(), ChainError> {
    book.save(&get_addressbook_path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

<<<<<<< HEAD
    #[test]
    fn test_addressbook_add_and_get() {
        let book = AddressBook::new();
=======
    // RAII structure to auto-cleanup test files
    struct TempFile(PathBuf);
    
    impl TempFile {
        fn new(name: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!("trinity_test_{}_{}.json", name, uuid::Uuid::new_v4()));
            Self(path)
        }
    }
>>>>>>> 310f51a (feat: Resolve compilation errors)

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn test_addressbook_sanitization() {
        let mut book = AddressBook::new();
        book.add(
            "  Alice  ".to_string(),
            "  abc123  ".to_string(),
            Some("  Friend  ".to_string()),
        )
        .unwrap();

        let entry = book.get("alice").unwrap();
        assert_eq!(entry.label, "Alice");
        assert_eq!(entry.address, "abc123");
        assert_eq!(entry.notes.as_deref(), Some("Friend"));
<<<<<<< HEAD
    }

    #[test]
    fn test_addressbook_case_insensitive() {
        let book = AddressBook::new();
        book.add("Alice".to_string(), "abc123".to_string(), None)
            .unwrap();

        assert!(book.get("alice").is_some());
        assert!(book.get("ALICE").is_some());
        assert!(book.get("AlIcE").is_some());
    }

    #[test]
    fn test_addressbook_duplicate_label() {
        let book = AddressBook::new();
        book.add("Alice".to_string(), "abc123".to_string(), None)
            .unwrap();

        let result = book.add("alice".to_string(), "def456".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_addressbook_duplicate_address() {
        let book = AddressBook::new();
        book.add("Alice".to_string(), "abc123".to_string(), None)
            .unwrap();

        let result = book.add("Bob".to_string(), "abc123".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_addressbook_remove() {
        let book = AddressBook::new();
        book.add("Bob".to_string(), "def456".to_string(), None)
            .unwrap();

        let removed = book.remove("bob").unwrap();
        assert_eq!(removed.label, "Bob");
        assert!(book.get("bob").is_none());
        assert!(!book.contains_address("def456"));
    }

    #[test]
    fn test_addressbook_update() {
        let book = AddressBook::new();
        book.add("Alice".to_string(), "abc123".to_string(), None)
            .unwrap();

        book.update(
            "Alice",
            Some("xyz789".to_string()),
            Some("Updated notes".to_string()),
        )
        .unwrap();

        let entry = book.get("alice").unwrap();
        assert_eq!(entry.address, "xyz789");
        assert_eq!(entry.notes.as_deref(), Some("Updated notes"));
        assert_eq!(entry.version, 2);
    }

    #[test]
    fn test_addressbook_search() {
        let book = AddressBook::new();
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

        let results = book.search("abc");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "Alice");
    }

    #[test]
    fn test_addressbook_get_by_address() {
        let book = AddressBook::new();
        book.add("Alice".to_string(), "abc123".to_string(), None)
            .unwrap();

        let entry = book.get_by_address("abc123").unwrap();
        assert_eq!(entry.label, "Alice");
    }

    #[test]
    fn test_addressbook_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_addressbook.json");

        let book1 = AddressBook::new();
        book1
            .add("Alice".to_string(), "abc123".to_string(), None)
            .unwrap();
        book1
            .add("Bob".to_string(), "def456".to_string(), None)
            .unwrap();

        book1.save(&path).unwrap();

        let book2 = AddressBook::load(&path).unwrap();
        assert_eq!(book2.len(), 2);
        assert!(book2.get("alice").is_some());
        assert!(book2.get("bob").is_some());
        assert!(book2.contains_address("abc123"));
        assert!(book2.contains_address("def456"));
    }

    #[test]
    fn test_validation_label_too_long() {
        let book = AddressBook::new();
        let long_label = "a".repeat(MAX_LABEL_LENGTH + 1);
        let result = book.add(long_label, "abc123".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_empty_label() {
        let book = AddressBook::new();
        let result = book.add("".to_string(), "abc123".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_csv() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("export.csv");

        let book = AddressBook::new();
        book.add(
            "Alice".to_string(),
            "abc123".to_string(),
            Some("Friend".to_string()),
        )
        .unwrap();
        book.add("Bob".to_string(), "def456".to_string(), None)
            .unwrap();

        book.export_csv(&csv_path).unwrap();

        let csv_content = fs::read_to_string(&csv_path).unwrap();
        assert!(csv_content.contains("Alice"));
        assert!(csv_content.contains("abc123"));
        assert!(csv_content.contains("Bob"));
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let book = AddressBook::new();
        let book_clone = book.clone();

        let handle = thread::spawn(move || {
            book_clone
                .add("Alice".to_string(), "abc123".to_string(), None)
                .unwrap();
        });

        book.add("Bob".to_string(), "def456".to_string(), None)
            .unwrap();

        handle.join().unwrap();
        assert_eq!(book.len(), 2);
=======
    }

    #[test]
    fn test_empty_notes_handling() {
        let mut book = AddressBook::new();
        // Should convert whitespace-only notes to None or store trimmed? 
        // Logic implemented: stores trimmed string.
        book.add("Bob".into(), "123".into(), Some("   ".into())).unwrap();
        
        let entry = book.get("bob").unwrap();
        // Assuming we want to filter out empty notes entirely:
        assert!(entry.notes.is_none()); 
    }

    #[test]
    fn test_addressbook_persistence() {
        let temp_file = TempFile::new("persistence");
        
        let mut book = AddressBook::new();
        book.add("Bob".to_string(), "def456".to_string(), None).unwrap();
        book.save(&temp_file.0).unwrap();

        let loaded = AddressBook::load(&temp_file.0).unwrap();
        assert_eq!(loaded.get("bob").unwrap().address, "def456");
        // TempFile drop handles cleanup
    }

    #[test]
    fn test_search_logic() {
        let mut book = AddressBook::new();
        book.add("Alice".into(), "123".into(), Some("Dev".into())).unwrap();
        book.add("Bob".into(), "456".into(), Some("QA".into())).unwrap();

        assert_eq!(book.search("alice").len(), 1);
        assert_eq!(book.search("qa").len(), 1);
        assert_eq!(book.search("23").len(), 1);
        assert_eq!(book.search("  ").len(), 2);
>>>>>>> 310f51a (feat: Resolve compilation errors)
    }
}
