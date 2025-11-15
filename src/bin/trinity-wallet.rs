//! Wallet CLI for TrinityChain - Beautiful edition!

use trinitychain::wallet::{self};
use colored::*;

const LOGO: &str = r#"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║     ████████╗██████╗ ██╗███╗   ██╗██╗████████╗██╗   ██╗      ║
║     ╚══██╔══╝██╔══██╗██║████╗  ██║██║╚══██╔══╝╚██╗ ██╔╝      ║
║        ██║   ██████╔╝██║██╔██╗ ██║██║   ██║    ╚████╔╝       ║
║        ██║   ██╔══██╗██║██║╚██╗██║██║   ██║     ╚██╔╝        ║
║        ██║   ██║  ██║██║██║ ╚████║██║   ██║      ██║         ║
║        ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝      ╚═╝         ║
║                                                               ║
║                   🔺 Wallet Manager 🔺                        ║
║                    Version 0.1.0 - Alpha                     ║
╚═══════════════════════════════════════════════════════════════╝
"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "new" => create_wallet(),
        "address" => show_address(),
        "list" => list_wallets(),
        "help" => print_usage(),
        _ => {
            println!("{}", format!("❌ Unknown command: {}", args[1]).red().bold());
            print_usage();
        }
    }
}

fn print_banner() {
    println!("{}", LOGO.bright_cyan());
}

fn create_wallet() {
    print_banner();

    println!("{}", "┌─────────────────────────────────────────┐".bright_green());
    println!("{}", "│       🔑 Creating New Wallet...        │".bright_green());
    println!("{}", "└─────────────────────────────────────────┘".bright_green());
    println!();

    match wallet::create_default_wallet() {
        Ok(wallet) => {
            println!("{}", "╔══════════════════════════════════════════════════════════╗".green());
            println!("{}", "║            ✨ Wallet Created Successfully! ✨            ║".green().bold());
            println!("{}", "╠══════════════════════════════════════════════════════════╣".green());
            let addr_len = wallet.address.len();
            let addr_part1 = if addr_len >= 42 { &wallet.address[..42] } else { &wallet.address };
            let addr_part2 = if addr_len > 42 { &wallet.address[42..] } else { "" };
            println!("{}", format!("║  📍 Address: {:<42} ║", addr_part1).green());
            println!("{}", format!("║             {:<42} ║", addr_part2).green());
            println!("{}", format!("║  📁 Location: {:<39} ║", wallet::get_default_wallet_path().display()).green());
            println!("{}", format!("║  📅 Created: {:<40} ║", wallet.created).green());
            println!("{}", "╚══════════════════════════════════════════════════════════╝".green());
            println!();
            println!("{}", "⚠️  IMPORTANT SECURITY NOTICE:".yellow().bold());
            println!("{}", "   • Backup your wallet file immediately!".yellow());
            println!("{}", "   • Never share your secret key".yellow());
            println!("{}", "   • Store backups in a secure location".yellow());
            println!();
        },
        Err(e) => {
            println!("{}", "╔══════════════════════════════════════════╗".red());
            println!("{}", "║       ❌ Wallet Creation Failed!        ║".red().bold());
            println!("{}", "╠══════════════════════════════════════════╣".red());
            println!("{}", format!("║  Error: {:<32} ║", e.to_string()).red());
            println!("{}", "╚══════════════════════════════════════════╝".red());
            println!();
        }
    }
}

fn show_address() {
    print_banner();

    println!("{}", "┌─────────────────────────────────────────┐".bright_cyan());
    println!("{}", "│      📍 Your Wallet Address...         │".bright_cyan());
    println!("{}", "└─────────────────────────────────────────┘".bright_cyan());
    println!();

    match wallet::load_default_wallet() {
        Ok(wallet) => {
            println!("{}", "╔══════════════════════════════════════════════════════════╗".cyan());
            println!("{}", "║                   Your Wallet Details                    ║".cyan().bold());
            println!("{}", "╠══════════════════════════════════════════════════════════╣".cyan());
            let addr_len = wallet.address.len();
            let addr_part1 = if addr_len >= 42 { &wallet.address[..42] } else { &wallet.address };
            let addr_part2 = if addr_len > 42 { &wallet.address[42..] } else { "" };
            println!("{}", format!("║  📍 Address: {:<42} ║", addr_part1).cyan());
            println!("{}", format!("║             {:<42} ║", addr_part2).cyan());
            println!("{}", format!("║  📅 Created: {:<40} ║", wallet.created).cyan());
            println!("{}", "╚══════════════════════════════════════════════════════════╝".cyan());
            println!();
            println!("{}", "💡 Tip: Share this address to receive triangles!".bright_blue());
            println!();
        },
        Err(e) => {
            println!("{}", "╔══════════════════════════════════════════╗".red());
            println!("{}", "║         ❌ Wallet Not Found!            ║".red().bold());
            println!("{}", "╠══════════════════════════════════════════╣".red());
            println!("{}", format!("║  Error: {:<32} ║", e.to_string()).red());
            println!("{}", "╚══════════════════════════════════════════╝".red());
            println!();
            println!("{}", "💡 Run 'trinity-wallet new' to create a wallet".yellow());
            println!();
        }
    }
}

fn list_wallets() {
    print_banner();

    println!("{}", "┌─────────────────────────────────────────┐".bright_magenta());
    println!("{}", "│      📋 Available Wallets...           │".bright_magenta());
    println!("{}", "└─────────────────────────────────────────┘".bright_magenta());
    println!();

    match wallet::list_wallets() {
        Ok(wallets) => {
            if wallets.is_empty() {
                println!("{}", "╔══════════════════════════════════════════╗".yellow());
                println!("{}", "║         No Wallets Found                ║".yellow());
                println!("{}", "╚══════════════════════════════════════════╝".yellow());
                println!();
                println!("{}", "💡 Run 'trinity-wallet new' to create your first wallet".yellow());
            } else {
                println!("{}", "╔══════════════════════════════════════════╗".magenta());
                println!("{}", format!("║  Found {} wallet(s):                       ║", wallets.len()).magenta().bold());
                println!("{}", "╠══════════════════════════════════════════╣".magenta());
                for (i, wallet_file) in wallets.iter().enumerate() {
                    println!("{}", format!("║  {}. {:<35} ║", i + 1, wallet_file).magenta());
                }
                println!("{}", "╚══════════════════════════════════════════╝".magenta());
            }
            println!();
        },
        Err(e) => {
            println!("{}", format!("❌ Error: {}", e).red());
            println!();
        }
    }
}

fn print_usage() {
    print_banner();

    println!("{}", "╔══════════════════════════════════════════════════════════╗".bright_yellow());
    println!("{}", "║                      📖 Usage Guide                      ║".bright_yellow().bold());
    println!("{}", "╠══════════════════════════════════════════════════════════╣".bright_yellow());
    println!("{}", "║                                                          ║".bright_yellow());
    println!("{}", "║  Commands:                                               ║".bright_yellow());
    println!("{}", "║                                                          ║".bright_yellow());
    println!("{}", "║    🔑 new       Create a new wallet                     ║".bright_yellow());
    println!("{}", "║    📍 address   Show your wallet address                ║".bright_yellow());
    println!("{}", "║    📋 list      List all available wallets              ║".bright_yellow());
    println!("{}", "║    ❓ help      Show this help message                  ║".bright_yellow());
    println!("{}", "║                                                          ║".bright_yellow());
    println!("{}", "╠══════════════════════════════════════════════════════════╣".bright_yellow());
    println!("{}", "║  Examples:                                               ║".bright_yellow());
    println!("{}", "║                                                          ║".bright_yellow());
    println!("{}", "║    $ trinity-wallet new                                  ║".white());
    println!("{}", "║    $ trinity-wallet address                              ║".white());
    println!("{}", "║    $ trinity-wallet list                                 ║".white());
    println!("{}", "║                                                          ║".bright_yellow());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".bright_yellow());
    println!();
}
