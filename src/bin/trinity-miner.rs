//! Miner CLI for TrinityChain - Beautiful edition!

use trinitychain::blockchain::{Blockchain, Block};
use trinitychain::persistence::Database;
use trinitychain::network::NetworkNode;
use trinitychain::transaction::{Transaction, CoinbaseTx};
use std::env;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

const LOGO: &str = r#"
╔═══════════════════════════════════════════════════════════════╗
║     ████████╗██████╗ ██╗███╗   ██╗██╗████████╗██╗   ██╗      ║
║     ╚══██╔══╝██╔══██╗██║████╗  ██║██║╚══██╔══╝╚██╗ ██╔╝      ║
║        ██║   ██████╔╝██║██╔██╗ ██║██║   ██║    ╚████╔╝       ║
║        ██║   ██╔══██╗██║██║╚██╗██║██║   ██║     ╚██╔╝        ║
║        ██║   ██║  ██║██║██║ ╚████║██║   ██║      ██║         ║
║        ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝      ╚═╝         ║
║                    ⛏️  CHAIN MINER ⛏️                         ║
╚═══════════════════════════════════════════════════════════════╝
"#;

/// Format a large number with thousands separators
fn format_number(num: u64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    result
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", LOGO.bright_yellow());
        println!("{}", "╔══════════════════════════════════════════════════════════╗".bright_yellow());
        println!("{}", "║                      📖 Usage Guide                      ║".bright_yellow().bold());
        println!("{}", "╠══════════════════════════════════════════════════════════╣".bright_yellow());
        println!("{}", "║  Usage:                                                  ║".bright_yellow());
        println!("{}", "║    miner <beneficiary_address> [--peer <host:port>]      ║".white());
        println!("{}", "║                                                          ║".bright_yellow());
        println!("{}", "║  Example:                                                ║".bright_yellow());
        println!("{}", "║    miner abc123...                                       ║".white());
        println!("{}", "║    miner abc123... --peer 192.168.1.10:8333             ║".white());
        println!("{}", "╚══════════════════════════════════════════════════════════╝".bright_yellow());
        println!();
        return;
    }
    let beneficiary_address = args[1].clone();

    println!("{}", LOGO.bright_yellow());
    println!("{}", "┌─────────────────────────────────────────────────────────────┐".bright_green());
    println!("{}", "│                   ⛏️  STARTING MINER                        │".bright_green().bold());
    println!("{}", "└─────────────────────────────────────────────────────────────┘".bright_green());
    println!();
    
    let db = Database::open("trinitychain.db").expect("Failed to open database");
    let mut chain = db.load_blockchain().unwrap_or_else(|_| {
        println!("{}", "⚠️  No blockchain found, creating genesis...".yellow());
        Blockchain::new()
    });

    let beneficiary_display = if beneficiary_address.len() > 20 {
        format!("{}...{}", &beneficiary_address[..10], &beneficiary_address[beneficiary_address.len()-10..])
    } else {
        beneficiary_address.clone()
    };

    println!("{}", "╔══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║                  ⚙️  MINER CONFIGURATION                 ║".cyan().bold());
    println!("{}", "╠══════════════════════════════════════════════════════════╣".cyan());
    println!("{}", format!("║  👤 Beneficiary: {:<40} ║", beneficiary_display).cyan());
    println!("{}", format!("║  💰 Reward: {:<45} ║", "1000 area").cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".cyan());
    println!();

    let network_node = NetworkNode::new(chain.clone(), "trinitychain.db".to_string());

    if args.len() >= 4 && args[2] == "--peer" {
        let peer_addr = &args[3];
        let parts: Vec<&str> = peer_addr.split(':').collect();
        if parts.len() == 2 {
            let peer_host = parts[0].to_string();
            let peer_port: u16 = parts[1].parse().expect("Invalid peer port");

            println!("{}", format!("🔗 Connecting to peer {}:{}...", peer_host, peer_port).bright_blue());
            if let Err(e) = network_node.connect_peer(peer_host, peer_port).await {
                eprintln!("{}", format!("❌ Failed to connect to peer: {}", e).red());
            } else {
                println!("{}", "✅ Connected to peer successfully!".green());
            }
            println!();
        }
    }

    let mut blocks_mined = 0;
    let start_time = Instant::now();

    println!("{}", "╔══════════════════════════════════════════════════════════╗".bright_green());
    println!("{}", "║              ⛏️  MINING IN PROGRESS...                   ║".bright_green().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".bright_green());
    println!();

    loop {
        // Reload blockchain from database before each mining round
        // This ensures we're mining on the latest chain, including blocks from peers
        chain = db.load_blockchain().unwrap_or_else(|_| {
            eprintln!("⚠️  Failed to reload blockchain, using current chain");
            chain
        });

        let last_block = chain.blocks.last().unwrap();
        let new_height = last_block.header.height + 1;
        let difficulty = chain.difficulty;

        let coinbase_tx = Transaction::Coinbase(CoinbaseTx {
            reward_area: 1000,
            beneficiary_address: beneficiary_address.clone(),
        });

        let mut new_block = Block::new(
            new_height,
            last_block.hash,
            difficulty,
            vec![coinbase_tx],
        );

        // Ensure timestamp is greater than parent to avoid validation errors
        if new_block.header.timestamp <= last_block.header.timestamp {
            new_block.header.timestamp = last_block.header.timestamp + 1;
        }

        println!("{}", format!("⛏️  Mining block #{} (difficulty: {})...", new_height, difficulty).bright_yellow());

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.yellow} {msg}")
                .unwrap()
        );
        pb.enable_steady_tick(Duration::from_millis(100));

        let mine_start = Instant::now();
        let mut hash_count = 0u64;

        loop {
            new_block.hash = new_block.calculate_hash();
            hash_count += 1;

            if hash_count % 10000 == 0 {
                let elapsed = mine_start.elapsed().as_secs_f64();
                let hashrate = if elapsed > 0.0 { hash_count as f64 / elapsed } else { 0.0 };
                pb.set_message(format!("Hashing... {} attempts ({:.0} H/s)", hash_count, hashrate));
            }

            if new_block.verify_proof_of_work() {
                pb.finish_and_clear();
                let mine_duration = mine_start.elapsed();
                let hash_hex = hex::encode(new_block.hash);
                let hash_display = format!("{}...{}", &hash_hex[..10], &hash_hex[hash_hex.len()-10..]);

                println!("{}", "┌─────────────────────────────────────────────────────────────┐".green());
                println!("{}", format!("│ ✨ BLOCK FOUND! #{:<45} │", new_height).green().bold());
                println!("{}", "├─────────────────────────────────────────────────────────────┤".green());
                println!("{}", format!("│ Hash: {:<52} │", hash_display).green());
                println!("{}", format!("│ Attempts: {:<48} │", hash_count).green());
                println!("{}", format!("│ Time: {:.2}s{:<47} │", mine_duration.as_secs_f64(), "").green());
                println!("{}", format!("│ Avg Hashrate: {:.0} H/s{:<36} │", hash_count as f64 / mine_duration.as_secs_f64(), "").green());
                println!("{}", "└─────────────────────────────────────────────────────────────┘".green());
                break;
            }
            new_block.header.nonce += 1;
        }

        if let Err(e) = chain.apply_block(new_block.clone()) {
            eprintln!("{}", format!("❌ Failed to apply new block: {}", e).red());
            sleep(Duration::from_secs(10)).await;
            continue;
        }

        // Use atomic save to ensure database consistency
        db.save_blockchain_state(&new_block, &chain.state, chain.difficulty)
            .expect("Failed to save blockchain state");

        if let Err(e) = network_node.broadcast_block(&new_block).await {
            eprintln!("{}", format!("⚠️  Failed to broadcast block: {}", e).yellow());
        } else {
            println!("{}", "📡 Block broadcasted to network".bright_blue());
        }

        blocks_mined += 1;
        let elapsed = start_time.elapsed();
        let avg_block_time = elapsed.as_secs_f64() / blocks_mined as f64;

        // Calculate supply statistics
        let current_height = chain.blocks.last().unwrap().header.height;
        let current_supply = Blockchain::calculate_current_supply(current_height);
        let supply_pct = (current_supply as f64 / trinitychain::blockchain::MAX_SUPPLY as f64) * 100.0;
        let current_reward = Blockchain::calculate_block_reward(current_height);
        let halving_era = current_height / 210_000;
        let blocks_to_halving = ((halving_era + 1) * 210_000).saturating_sub(current_height);

        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════╗".bright_cyan());
        println!("{}", "║                    📊 MINING STATS                       ║".bright_cyan().bold());
        println!("{}", "╠══════════════════════════════════════════════════════════╣".bright_cyan());
        println!("{}", format!("║ 🔺 Blocks Mined: {:<39} ║", blocks_mined).cyan());
        println!("{}", format!("║ 🏔️  Chain Height: {:<39} ║", current_height).cyan());
        println!("{}", format!("║ ⏱️  Uptime: {:.0}m {:.0}s{:<38} ║", elapsed.as_secs() / 60, elapsed.as_secs() % 60, "").cyan());
        println!("{}", format!("║ ⚡ Avg Block Time: {:.1}s{:<34} ║", avg_block_time, "").cyan());
        println!("{}", format!("║ 🎯 Difficulty: {:<41} ║", chain.difficulty).cyan());
        println!("{}", format!("║ 💎 Current Reward: {:<35} ║", current_reward).cyan());
        println!("{}", format!("║ 🪙  Total Earned: {:<37.1} ║", blocks_mined as f64 * 1000.0).cyan());
        println!("{}", format!("║ 📈 Total Supply: {:>10} / {} ({:.3}%){:<6} ║",
                 format_number(current_supply),
                 format_number(trinitychain::blockchain::MAX_SUPPLY),
                 supply_pct, "").cyan());
        println!("{}", format!("║ ⏰ Blocks to Halving: {:<32} ║", format_number(blocks_to_halving)).cyan());
        println!("{}", format!("║ 🎚️  Halving Era: {:<38} ║", halving_era).cyan());
        println!("{}", "╚══════════════════════════════════════════════════════════╝".bright_cyan());
        println!();
    }
}