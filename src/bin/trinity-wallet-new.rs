//! Create a new named wallet

use trinitychain::wallet::create_named_wallet;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: trinity-wallet-new <name>");
        return;
    }

    let wallet_name = &args[1];

    match create_named_wallet(wallet_name) {
        Ok(wallet) => {
            println!("🔑 New wallet '{}' created!", wallet_name);
            println!("   Address: {}", wallet.address);
            println!("   Location: ~/.trinitychain/wallet_{}.json", wallet_name);
        }
        Err(e) => {
            eprintln!("❌ Error creating wallet: {}", e);
        }
    }
}
