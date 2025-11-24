//! Backup wallet with password encryption

use trinitychain::wallet::{self, EncryptedWallet};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Wallet Backup Tool\n");

    // Load current wallet
    let wallet = wallet::load_default_wallet()?;

    println!("📍 Wallet Address: {}...\n", &wallet.address[..42.min(wallet.address.len())]);

    // Get password (twice for confirmation)
    print!("Enter backup password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;

    print!("Confirm backup password: ");
    io::stdout().flush()?;
    let password_confirm = rpassword::read_password()?;

    if password != password_confirm {
        eprintln!("\n❌ Passwords do not match!");
        std::process::exit(1);
    }

    if password.len() < 8 {
        eprintln!("\n❌ Password must be at least 8 characters!");
        std::process::exit(1);
    }

    // Encrypt wallet
    println!("\n🔒 Encrypting wallet...");
    let encrypted = EncryptedWallet::from_wallet(&wallet, &password)?;

    // Save encrypted backup
    let backup_path = wallet::get_wallet_dir()?.join("wallet_backup.json");
    encrypted.save(&backup_path)?;

    println!("✅ Wallet backed up successfully!");
    println!("📁 Backup location: {}", backup_path.display());
    println!();
    println!("⚠️  IMPORTANT:");
    println!("   • Keep your password safe - it cannot be recovered!");
    println!("   • Store this backup file in a secure location");
    println!("   • You can restore from this backup using: trinity-wallet-restore");
    println!();

    Ok(())
}
