//! Shared CLI utilities

use crate::blockchain::Blockchain;
use crate::config::{load_config, Config};
use crate::persistence::Database;

pub fn load_blockchain_from_config() -> Result<(Config, Blockchain), Box<dyn std::error::Error>> {
    let config = load_config()?;
    let db = Database::open(&config.database.path)?;
    let blockchain = db.load_blockchain()?;
    Ok((config, blockchain))
}
