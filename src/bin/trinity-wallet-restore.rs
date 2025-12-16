#![forbid(unsafe_code)]
//! Restore wallet from encrypted backup

use std::io::{self, Write};
use std::path::PathBuf;
use trinitychain::wallet::{self, EncryptedWallet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    println!("🔓 Wallet Restore Tool\n");

    // Get backup file path
    let backup_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        wallet::get_wallet_dir()?.join("wallet_backup.json")
    };

    if !backup_path.exists() {
        eprintln!("❌ Backup file not found: {}", backup_path.display());
        eprintln!("\nUsage: trinity-wallet-restore [backup_file_path]");
        std::process::exit(1);
    }

    println!("📁 Backup file: {}\n", backup_path.display());

    // Load encrypted backup
    let encrypted = EncryptedWallet::load(&backup_path)?;

    println!(
        "📍 Wallet Address: {}...\n",
        &encrypted.address[..42.min(encrypted.address.len())]
    );

    // Get password
    print!("Enter backup password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;

    // Decrypt wallet
    println!("\n🔓 Decrypting wallet...");
    let wallet = match encrypted.decrypt(&password) {
        Ok(w) => w,
        Err(_) => {
            eprintln!("❌ Failed to decrypt - incorrect password!");
            std::process::exit(1);
        }
    };

    // Check if wallet already exists
    let wallet_path = wallet::get_default_wallet_path()?;
    if wallet_path.exists() {
        print!("\n⚠️  WARNING: A wallet already exists. Overwrite? (yes/no): ");
        io::stdout().flush()?;
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if response.trim().to_lowercase() != "yes" {
            println!("Restore cancelled.");
            std::process::exit(0);
        }
    }

    // Save wallet
    wallet::ensure_wallet_dir()?;
    wallet.save(&wallet_path)?;

    println!("✅ Wallet restored successfully!");
    println!("📁 Wallet location: {}", wallet_path.display());
    println!(
        "📍 Address: {}...",
        &wallet.address[..42.min(wallet.address.len())]
    );
    println!();

    Ok(())
}
