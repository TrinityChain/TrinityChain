use clap::{Parser, Subcommand};
use colored::*;
use trinitychain::wallet;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Logs in to a wallet
    Login {
        /// The name of the wallet to log in to
        name: Option<String>,
    },
    /// Logs out of the current wallet
    Logout,
    /// Shows the current login status
    Status,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Login { name } => {
            login(name.as_deref())?;
        }
        Commands::Logout => {
            logout()?;
        }
        Commands::Status => {
            status()?;
        }
    }

    Ok(())
}

fn login(name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let wallet_name = match name {
        Some(n) => n.to_string(),
        None => {
            let wallets = wallet::list_wallets()?;
            if wallets.is_empty() {
                println!("{}", "No wallets found. Create one with 'trinity-wallet new'".red());
                return Ok(());
            }

            println!("{}", "Available wallets:".bright_green());
            for (i, wallet) in wallets.iter().enumerate() {
                println!("{}. {}", i + 1, wallet);
            }
            println!();

            let choice = loop {
                println!("Enter the number of the wallet to log in to:");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                match input.trim().parse::<usize>() {
                    Ok(n) if n > 0 && n <= wallets.len() => break wallets[n - 1].clone(),
                    _ => println!("{}", "Invalid choice. Please try again.".red()),
                }
            };
            choice
        }
    };

    println!("Attempting to log in to wallet: {}", wallet_name.bright_cyan());

    let wallet = if wallet_name == "wallet.json" {
        wallet::load_default_wallet()
    } else {
        let name = wallet_name.strip_prefix("wallet_").unwrap_or(&wallet_name);
        let name = name.strip_suffix(".json").unwrap_or(name);
        wallet::load_named_wallet(name)
    };

    match wallet {
        Ok(w) => {
            let session_path = wallet::get_wallet_dir()?.join("session.json");
            let session_data = serde_json::to_string(&w)?;
            std::fs::write(&session_path, session_data)?;
            println!("{}", "Login successful!".bright_green());
            println!("Wallet Address: {}", w.address.bright_yellow());
        }
        Err(e) => {
            println!("{} {}", "Login failed:".red(), e);
        }
    }

    Ok(())
}

fn logout() -> Result<(), Box<dyn std::error::Error>> {
    let session_path = wallet::get_wallet_dir()?.join("session.json");
    if session_path.exists() {
        std::fs::remove_file(session_path)?;
        println!("{}", "Logged out successfully.".bright_green());
    } else {
        println!("{}", "Not currently logged in.".yellow());
    }
    Ok(())
}

fn status() -> Result<(), Box<dyn std::error::Error>> {
    let session_path = wallet::get_wallet_dir()?.join("session.json");
    if session_path.exists() {
        let session_data = std::fs::read_to_string(&session_path)?;
        let wallet: wallet::Wallet = serde_json::from_str(&session_data)?;
        println!("{} {}", "Logged in as:".bright_green(), wallet.name.unwrap_or_else(|| "default".to_string()).bright_cyan());
        println!("Address: {}", wallet.address.bright_yellow());
    } else {
        println!("{}", "Not currently logged in.".yellow());
    }
    Ok(())
}
