//! Create a second wallet

use trinitychain::crypto::KeyPair;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: trinity-wallet-new <name>");
        return;
    }
    
    let wallet_name = &args[1];
    let wallet_dir = match std::env::var("HOME") {
        Ok(val) => val + "/.trinitychain",
        Err(_) => {
            eprintln!("Error: HOME environment variable not set.");
            return;
        }
    };
    if let Err(e) = fs::create_dir_all(&wallet_dir) {
        eprintln!("Error: Failed to create wallet directory '{}': {}", wallet_dir, e);
        return;
    }
    
    let keypair = match KeyPair::generate() {
        Ok(kp) => kp,
        Err(e) => {
            eprintln!("Error: Failed to generate keypair: {}", e);
            return;
        }
    };
    let address = keypair.address();
    
    let wallet_file = format!("{}/wallet_{}.json", wallet_dir, wallet_name);
    
    if std::path::Path::new(&wallet_file).exists() {
        println!("⚠️  Wallet '{}' already exists", wallet_name);
        return;
    }
    
    let secret_hex = hex::encode(keypair.secret_key.secret_bytes());
    let wallet_data = serde_json::json!({
        "name": wallet_name,
        "address": address,
        "secret_key": secret_hex,
        "created": chrono::Utc::now().to_rfc3339(),
    });
    
    let wallet_json = match serde_json::to_string_pretty(&wallet_data) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error: Failed to serialize wallet data: {}", e);
            return;
        }
    };
    
    if let Err(e) = fs::write(&wallet_file, wallet_json) {
        eprintln!("Error: Failed to write wallet file '{}': {}", wallet_file, e);
        return;
    }
    
    println!("🔑 New wallet '{}' created!", wallet_name);
    println!("   Address: {}", address);
    println!("   Location: {}", wallet_file);
}
