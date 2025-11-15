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
    let wallet_dir = std::env::var("HOME").unwrap() + "/.trinitychain";
    fs::create_dir_all(&wallet_dir).expect("Failed to create wallet directory");
    
    let keypair = KeyPair::generate().expect("Failed to generate keypair");
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
    
    fs::write(&wallet_file, serde_json::to_string_pretty(&wallet_data).unwrap())
        .expect("Failed to write wallet file");
    
    println!("🔑 New wallet '{}' created!", wallet_name);
    println!("   Address: {}", address);
    println!("   Location: {}", wallet_file);
}
