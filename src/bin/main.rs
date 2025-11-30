use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::{SinkExt, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block as TuiBlock, Borders, Gauge, Paragraph, Sparkline},
    Terminal,
};
use secp256k1::SecretKey;
use std::collections::HashSet;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use trinitychain::api::Node;
use trinitychain::blockchain::{Block, Blockchain};
use trinitychain::cli::load_blockchain_from_config;
use trinitychain::config::{load_config, Config};
use trinitychain::crypto::KeyPair;
use trinitychain::miner::{mine_block, mine_block_parallel};
use trinitychain::network::NetworkNode;
use trinitychain::persistence::Database;
use trinitychain::transaction::{CoinbaseTx, SubdivisionTx, Transaction, TransferTx};
use trinitychain::wallet;
use log::{info, warn};
use hex;
use colored::*; 
use trinitychain::geometry::Coord;
use comfy_table::{Attribute, Cell, ContentArrangement, Table, presets::UTF8_FULL};
use std::env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs the TrinityChain node
    Node {
        #[arg(long)]
        peer: Option<String>,
    },
    /// Mines a new block
    MineBlock,
    /// Starts the persistent miner
    Miner,
    /// Sends a transaction
    Send {
        to_address: String,
        amount: f64,
        #[arg(long)]
        from: Option<String>,
        memo: Option<String>,
    },
    /// Shows the transaction history
    History,
    /// Shows the balance of the wallet
    Balance {
        address: Option<String>,
    },
    /// Creates a new wallet
    NewWallet {
        name: Option<String>,
    },
    /// Backs up the wallet
    BackupWallet,
    /// Restores the wallet
    RestoreWallet,
    /// Manages the address book
    AddressBook,
    /// Connects to a peer
    Connect,
    /// Runs the API server
    Server,
    /// Runs the Telegram bot
    TelegramBot,
}

#[derive(Clone)]
struct NodeStats {
    chain_height: u64,
    peer_count: usize,
    uptime_secs: u64,
    status: String,
    peers: Vec<String>,
    last_block_hash: String,
}

impl Default for NodeStats {
    fn default() -> Self {
        Self {
            chain_height: 0,
            peer_count: 0,
            uptime_secs: 0,
            status: "Initializing...".to_string(),
            peers: Vec::new(),
            last_block_hash: "N/A".to_string(),
        }
    }
}

#[derive(Clone)]
struct MiningStats {
    mining_status: String,
    difficulty: u32,
    blocks_mined: u64,
    chain_height: u64,
    uptime_secs: u64,
    avg_block_time: f64,
    current_reward: f64,
    total_earned: f64,
    current_supply: f64,
    blocks_to_halving: u64,
    halving_era: u64,
    last_block_hash: String,
    last_block_time: f64,
    recent_blocks: Vec<(u64, String, String)>,
}

impl Default for MiningStats {
    fn default() -> Self {
        Self {
            mining_status: "Initializing...".to_string(),
            difficulty: 0,
            blocks_mined: 0,
            chain_height: 0,
            uptime_secs: 0,
            avg_block_time: 0.0,
            current_reward: 0.0,
            total_earned: 0.0,
            current_supply: 0.0,
            blocks_to_halving: 0,
            halving_era: 0,
            last_block_hash: "N/A".to_string(),
            last_block_time: 0.0,
            recent_blocks: Vec::new(),
        }
    }
}

async fn run_node(peer: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    let port = config.network.p2p_port;
    let db_path = config.database.path;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let db = Database::open(&db_path).expect("Failed to open database");
    let blockchain = db.load_blockchain().unwrap_or_else(|_| Blockchain::new("".to_string(), 1).expect("Failed to create new blockchain"));

    // Create the unified Node
    let node = Arc::new(Node::new(blockchain));

    let stats = Arc::new(tokio::sync::Mutex::new(NodeStats {
        ..Default::default()
    }));

    let start_time = Instant::now();

    // Connect to peer if specified
    if let Some(peer_addr) = peer {
        let node_clone = node.clone();
        let peer_addr = peer_addr.to_string();
        tokio::spawn(async move {
            if let Some((host, port_str)) = peer_addr.split_once(':') {
                if let Ok(peer_port) = port_str.parse::<u16>() {
                    if let Err(e) = node_clone
                        .network
                        .clone()
                        .connect_peer(host.to_string(), peer_port)
                        .await
                    {
                        eprintln!("⚠️  Failed to connect to peer: {}", e);
                    }
                }
            }
        });
    }

    // Start P2P server
    let p2p_node = node.network.clone();
    tokio::spawn(async move {
        if let Err(e) = p2p_node.start_server(port).await {
            eprintln!("❌ Network error: {}", e);
        }
    });

    // UI and stats update loop
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        {
            let mut s = stats.lock().await;
            s.status = "Running".to_string();
            s.uptime_secs = start_time.elapsed().as_secs();

            let bc = node.blockchain.read().await;
            s.chain_height = bc.blocks.len() as u64;
            if let Some(last_block) = bc.blocks.last() {
                s.last_block_hash = hex::encode(last_block.hash());
            }

            let peers = node.network.list_peers().await;
            s.peer_count = peers.len();
            s.peers = peers.iter().map(|p| p.addr()).collect();
        }

        let _stats_clone = stats.lock().await.clone();
        terminal.draw(|_f| {
            // draw_ui(f, &stats_clone); // Assuming draw_ui is defined elsewhere
        })?;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_mine_block() -> Result<(), Box<dyn std::error::Error>> {
    println!("⛏️  Mining Block...\n");

    let (config, mut chain) = load_blockchain_from_config()?;
    let threads = config.miner.threads;

    // Load mempool from disk if it exists
    if let Ok(mempool_data) = std::fs::read_to_string("mempool.json") {
        let transactions: Result<Vec<Transaction>, _> = serde_json::from_str(&mempool_data);
        if let Ok(txs) = transactions {
            for tx in txs {
                let _ = chain.mempool.add_transaction(tx);
            }
            println!(
                "📬 Loaded {} pending transactions from mempool",
                chain.mempool.len()
            );
        }
    }

    let current_height = chain
        .blocks
        .last()
        .map(|b| b.header.height)
        .ok_or("Blockchain is empty")?;
    println!("📊 Current height: {}", current_height);

    let wallet_path = wallet::get_default_wallet_path()?;
    if !wallet_path.exists() {
        println!("👛 No default wallet found. Creating a new one...");
        wallet::create_default_wallet()?;
        println!("✅ New wallet created at: {}", wallet_path.display());
    }

    let wallet_data = wallet::load_default_wallet()?;

    let address = wallet_data.address;
    let secret_hex = wallet_data.secret_key_hex;
    let secret_bytes = hex::decode(secret_hex)?;
    let secret_key = SecretKey::from_slice(&secret_bytes)?;
    let keypair = KeyPair::from_secret_key(secret_key);
    let mut mempool_txs = chain.mempool.get_all_transactions();
    mempool_txs.sort_by(|a, b| b.fee().cmp(&a.fee()));
    mempool_txs.truncate(100);

    // Collect locked triangles from pending transfers
    let mut locked_triangles = HashSet::new();
    for tx in &mempool_txs {
        if let Transaction::Transfer(transfer_tx) = tx {
            locked_triangles.insert(transfer_tx.input_hash);
        }
    }

    // Select parent triangle, avoiding locked ones
    let parent_hash = *chain
        .state
        .utxo_set
        .keys()
        .find(|hash| !locked_triangles.contains(*hash))
        .ok_or("No UTXOs available for subdivision")?;
    let parent_triangle = chain
        .state
        .utxo_set
        .get(&parent_hash)
        .ok_or("Parent triangle not found")? .clone();

    let hash_hex = hex::encode(parent_hash);
    let hash_prefix = &hash_hex[..16];
    println!("🔺 Subdividing triangle {}...", hash_prefix);

    let children = parent_triangle.subdivide();

    let mut tx = SubdivisionTx::new(
        parent_hash,
        children.to_vec(),
        address.clone(),
        trinitychain::geometry::Coord::from_num(0),
        chain.blocks.len() as u64,
    );
    let message = tx.signable_message();
    let signature = keypair.sign(&message)?;
    let public_key = keypair.public_key.serialize().to_vec();
    tx.sign(signature, public_key);

    let coinbase = CoinbaseTx {
        reward_area: trinitychain::geometry::Coord::from_num(1000),
        beneficiary_address: address,
    };

    let mut transactions = vec![Transaction::Coinbase(coinbase)];
    transactions.extend(mempool_txs);
    transactions.push(Transaction::Subdivision(tx));

    println!("⛏️  Mining block (difficulty {})...", chain.difficulty);

    let last_block = chain.blocks.last().ok_or("Blockchain is empty")?;
    let mut new_block = trinitychain::blockchain::Block::new(
        last_block.header.height + 1,
        last_block.hash(),
        chain.difficulty,
        transactions,
    );

    if threads > 1 {
        new_block = mine_block_parallel(new_block)?;
    } else {
        new_block = mine_block(new_block)?;
    }

    let new_hash_hex = hex::encode(new_block.hash());
    let new_hash_prefix = &new_hash_hex[..16];
    println!("✅ Block mined! Hash: {}", new_hash_prefix);

    chain.apply_block(new_block.clone())?;
    let db = Database::open(&config.database.path)?;
    db.save_block(&new_block)?;
    db.save_utxo_set(&chain.state)?;

    // Clear mempool file after mining
    let _ = std::fs::remove_file("mempool.json");

    println!("\n🎉 Block {} mined successfully!", chain.blocks.len() - 1);
    println!("   UTXOs: {}", chain.state.utxo_set.len());

    Ok(())
}

async fn run_send(to_address: &str, amount: f64, from: Option<&str>, memo: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let amount_coord = Coord::from_num(amount);

    let wallet_name = from.map(|s| s.to_string());

    println!(
        "{}",
        "┌─────────────────────────────────────────────────────────────┐".bright_magenta()
    );
    println!(
        "{}",
        "│                  💸 INITIATING TRANSFER                     │"
            .bright_magenta()
            .bold()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────────────────┘".bright_magenta()
    );
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    pb.set_message("Loading wallet...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let from_wallet = if let Some(name) = wallet_name {
        wallet::load_named_wallet(&name)?
    } else {
        wallet::load_default_wallet()? 
    };

    let from_address = from_wallet.address.clone();
    let keypair = from_wallet.get_keypair()?;

    pb.set_message("Loading blockchain...");

    let (_config, mut chain) = load_blockchain_from_config()?;

    // Track locked triangles from pending transactions
    let mut locked_triangles = HashSet::new();

    // Load existing mempool from disk
    if let Ok(mempool_data) = std::fs::read_to_string("mempool.json") {
        let transactions: Result<Vec<Transaction>, _> = serde_json::from_str(&mempool_data);
        if let Ok(txs) = transactions {
            let txs_clone = txs.clone();

            for tx in txs {
                let _ = chain.mempool.add_transaction(tx);
            }

            if chain.mempool.len() > 0 {
                pb.println(format!(
                    "📬 {} pending transaction(s) already in mempool",
                    chain.mempool.len()
                ));
            }

            // Collect locked UTXOs from pending transfers
            for tx in txs_clone {
                if let Transaction::Transfer(transfer_tx) = tx {
                    locked_triangles.insert(transfer_tx.input_hash);
                }
            }
        }
    }
    pb.set_message("Finding a suitable triangle...");

    let (input_hash, _input_triangle) = chain
        .state
        .utxo_set
        .iter()
        .find(|(hash, triangle)| {
            triangle.owner == from_address
                && triangle.effective_value() >= amount_coord
                && !locked_triangles.contains(*hash)
        })
        .ok_or("No single triangle with sufficient value found for the transfer")?;

    pb.finish_and_clear();

    let from_display = if from_address.len() > 20 {
        format!(
            "{}...{}",
            &from_address[..10],
            &from_address[from_address.len() - 10..]
        )
    } else {
        from_address.clone()
    };
    let to_display = if to_address.len() > 20 {
        format!(
            "{}...{}",
            &to_address[..10],
            &to_address[to_address.len() - 10..]
        )
    } else {
        to_address.to_string()
    };

    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗".bright_cyan()
    );
    println!(
        "{}",
        "║              🔍 TRANSACTION DETAILS                      ║"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "╠══════════════════════════════════════════════════════════╣".bright_cyan()
    );
    println!("{}", format!("║  👤 From: {:<47} ║", from_display).cyan());
    println!("{}", format!("║  🎯 To: {:<49} ║", to_display).cyan());
    println!("{}", format!("║  💸 Amount: {:<45} ║", amount).cyan());
    if let Some(ref m) = memo {
        let memo_display = if m.len() > 45 {
            format!("{}", &m[..42])
        } else {
            m.to_string()
        };
        println!("{}", format!("║  📝 Memo: {:<47} ║", memo_display).cyan());
    }
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝".bright_cyan()
    );
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    pb.set_message("Creating transaction...");

    let fee = Coord::from_num(0);
    let mut tx = TransferTx::new(
        *input_hash,
        to_address.to_string(),
        from_address.clone(),
        amount_coord,
        fee,
        chain.blocks.len() as u64,
    );

    if let Some(m) = memo {
        tx = tx.with_memo(m.to_string())?;
    }

    pb.set_message("Signing transaction...");

    let message = tx.signable_message();
    let signature = keypair.sign(&message)?;
    let public_key = keypair.public_key.serialize().to_vec();
    tx.sign(signature, public_key);

    let transaction = Transaction::Transfer(tx);
    chain.mempool.add_transaction(transaction.clone())?;

    pb.set_message("Saving mempool...");
    let all_txs = chain.mempool.get_all_transactions();
    std::fs::write("mempool.json", serde_json::to_string(&all_txs)?)?;

    pb.set_message("Broadcasting to network...");

    let network_node = NetworkNode::new(Arc::new(RwLock::new(chain)));
    network_node.broadcast_transaction(&transaction).await;

    pb.finish_and_clear();

    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗".bright_green()
    );
    println!(
        "{}",
        "║              ✅ TRANSACTION SUCCESSFUL!                  ║"
            .bright_green()
            .bold()
    );
    println!(
        "{}",
        "╠══════════════════════════════════════════════════════════╣".bright_green()
    );
    println!(
        "{}",
        "║  Your transaction has been broadcasted to the network   ║".green()
    );
    println!(
        "{}",
        "║  and will be included in the next block!                ║".green()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝".bright_green()
    );
    println!();
    println!(
        "{}",
        "🎉 Transfer complete! The triangle is on its way!".bright_blue()
    );
    println!();

    Ok(())
}

async fn run_history() -> Result<(), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let wallet_file = format!("{}/.trinitychain/wallet.json", home);

    let wallet_content = std::fs::read_to_string(&wallet_file).map_err(|e| {
        eprintln!("{}", "╔══════════════════════════════════════════╗".red());
        eprintln!(
            "{}",
            "║         ❌ Wallet Not Found!            ║".red().bold()
        );
        eprintln!("{}", "╚══════════════════════════════════════════╝".red());
        eprintln!();
        eprintln!("{}", "💡 Run 'wallet new' to create a wallet".yellow());
        format!("No wallet found at {}: {}", wallet_file, e)
    })?;

    let wallet_data: serde_json::Value = serde_json::from_str(&wallet_content)
        .map_err(|e| format!("Failed to parse wallet: {}", e))?;

    let my_address = wallet_data["address"]
        .as_str()
        .ok_or("Wallet address not found in wallet file")?;

    let (_config, chain) = load_blockchain_from_config()?;

    let addr_display = if my_address.len() > 40 {
        format!(
            "{}...{}",
            &my_address[..20],
            &my_address[my_address.len() - 16..]
        )
    } else {
        my_address.to_string()
    };

    println!(
        "{}",
        "┌─────────────────────────────────────────────────────────────┐".bright_cyan()
    );
    println!(
        "{}",
        "│                  📜 TRANSACTION HISTORY                     │"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────────────────┘".bright_cyan()
    );
    println!();
    println!("{}", format!("📍 Address: {}", addr_display).cyan());
    println!();

    let mut tx_count = 0;
    let mut received_count = 0;
    let mut sent_count = 0;
    let mut mining_count = 0;

    struct TxRecord {
        block_height: u64,
        tx_type: String,
        direction: String,
        details: String,
        timestamp: i64,
        color: comfy_table::Color,
    }

    let mut transactions: Vec<TxRecord> = Vec::new();

    // Iterate through all blocks
    for block in &chain.blocks {
        for tx in &block.transactions {
            match tx {
                Transaction::Transfer(transfer_tx) => {
                    let is_sender = transfer_tx.sender == my_address;
                    let is_receiver = transfer_tx.new_owner == my_address;

                    if is_sender || is_receiver {
                        tx_count += 1;

                        let (direction, color) = if is_sender && is_receiver {
                            ("↔️  Self".to_string(), comfy_table::Color::Yellow)
                        } else if is_sender {
                            sent_count += 1;
                            ("📤 Sent".to_string(), comfy_table::Color::Red)
                        } else {
                            received_count += 1;
                            ("📥 Received".to_string(), comfy_table::Color::Green)
                        };

                        let hash_hex = hex::encode(transfer_tx.input_hash);
                        let hash_short = if hash_hex.len() > 16 {
                            format!("{}...", &hash_hex[..13])
                        } else {
                            hash_hex
                        };

                        let other_party = if is_sender {
                            let addr = &transfer_tx.new_owner;
                            if addr.len() > 20 {
                                format!("To: {}...", &addr[..8])
                            } else {
                                format!("To: {}", addr)
                            }
                        } else {
                            let addr = &transfer_tx.sender;
                            if addr.len() > 20 {
                                format!("From: {}...", &addr[..8])
                            } else {
                                format!("From: {}", addr)
                            }
                        };

                        let memo_str = if let Some(memo) = &transfer_tx.memo {
                            if memo.len() > 20 {
                                format!(" | \"{}...\"", &memo[..17])
                            } else {
                                format!(" | \"{}\"", memo)
                            }
                        } else {
                            String::new()
                        };

                        transactions.push(TxRecord {
                            block_height: block.header.height,
                            tx_type: "Transfer".to_string(),
                            direction,
                            details: format!("{} | {}{}", hash_short, other_party, memo_str),
                            timestamp: block.header.timestamp as i64,
                            color,
                        });
                    }
                }
                Transaction::Coinbase(coinbase_tx) => {
                    if coinbase_tx.beneficiary_address == my_address {
                        tx_count += 1;
                        received_count += 1;
                        mining_count += 1;

                        transactions.push(TxRecord {
                            block_height: block.header.height,
                            tx_type: "Mining".to_string(),
                            direction: "⛏️  Reward".to_string(),
                            details: format!("Area: {}", coinbase_tx.reward_area),
                            timestamp: block.header.timestamp as i64,
                            color: comfy_table::Color::Cyan,
                        });
                    }
                }
                Transaction::Subdivision(sub_tx) => {
                    if sub_tx.owner_address == my_address {
                        tx_count += 1;

                        let hash_hex = hex::encode(sub_tx.parent_hash);
                        let hash_short = if hash_hex.len() > 16 {
                            format!("{}...", &hash_hex[..13])
                        } else {
                            hash_hex
                        };

                        transactions.push(TxRecord {
                            block_height: block.header.height,
                            tx_type: "Subdivision".to_string(),
                            direction: "✂️  Split".to_string(),
                            details: format!("{} → {} children", hash_short, sub_tx.children.len()),
                            timestamp: block.header.timestamp as i64,
                            color: comfy_table::Color::Magenta,
                        });
                    }
                }
            }
        }
    }

    if transactions.is_empty() {
        println!(
            "{}",
            "╔══════════════════════════════════════════════════════════╗".yellow()
        );
        println!(
            "{}",
            "║              📭 No Transactions Found                    ║".yellow()
        );
        println!(
            "{}",
            "╠══════════════════════════════════════════════════════════╣".yellow()
        );
        println!(
            "{}",
            "║  No transaction history yet. Start using your wallet!   ║".yellow()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════╝".yellow()
        );
        println!();
        return Ok(())
    }

    // Reverse to show newest first
    transactions.reverse();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Block")
                .fg(comfy_table::Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Type")
                .fg(comfy_table::Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Direction")
                .fg(comfy_table::Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Details")
                .fg(comfy_table::Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Date")
                .fg(comfy_table::Color::Cyan)
                .add_attribute(Attribute::Bold),
        ]);

    for tx in &transactions {
        table.add_row(vec![
            Cell::new(format!("#வுகளை{}", tx.block_height)).fg(comfy_table::Color::White),
            Cell::new(&tx.tx_type).fg(tx.color),
            Cell::new(&tx.direction).fg(tx.color),
            Cell::new(&tx.details).fg(comfy_table::Color::White),
            Cell::new(format_timestamp_short(tx.timestamp)).fg(comfy_table::Color::Grey),
        ]);
    }

    println!("{}", table);
    println!();

    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗".bright_blue()
    );
    println!(
        "{}",
        "║                    📊 TRANSACTION SUMMARY                ║"
            .bright_blue()
            .bold()
    );
    println!(
        "{}",
        "╠══════════════════════════════════════════════════════════╣".bright_blue()
    );
    println!(
        "{}",
        format!("║  📝 Total Transactions: {:<33} ║", tx_count).blue()
    );
    println!(
        "{}",
        format!("║  📥 Received: {:<43} ║", received_count).green()
    );
    println!("{}", format!("║  📤 Sent: {:<47} ║", sent_count).red());
    println!(
        "{}",
        format!("║  ⛏️  Mining Rewards: {:<36} ║", mining_count).cyan()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝".bright_blue()
    );
    println!();

    Ok(())
}

fn format_timestamp_short(timestamp: i64) -> String {
    use chrono::DateTime;

    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        dt.format("%m/%d %H:%M").to_string()
    } else {
        "Invalid".to_string()
    }
}

async fn run_miner() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let config = Arc::new(load_config()?);
    let beneficiary_address = config.miner.beneficiary_address.clone();
    let threads = config.miner.threads;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let stats = Arc::new(Mutex::new(MiningStats::default()));
    let stats_clone = Arc::clone(&stats);
    let beneficiary_clone = beneficiary_address.clone();

    // Create and start network node
    let db_for_network = Database::open(&config.database.path).expect("Failed to open database");
    let chain_for_network = db_for_network
        .load_blockchain()
        .unwrap_or_else(|_| Blockchain::new("".to_string(), 1).expect("Failed to create new blockchain"));
    let network = Arc::new(NetworkNode::new(Arc::new(RwLock::new(chain_for_network))));
    let network_clone = network.clone();
    let config_clone = Arc::clone(&config);

    // Start network server in background
    tokio::spawn(async move {
        let port = config_clone.network.p2p_port;
        println!("🌐 Starting P2P network on port {}...", port);
        if let Err(e) = network_clone.start_server(port).await {
            eprintln!("❌ Network error: {}", e);
        }
    });

    // Spawn mining task
    let mining_handle = tokio::spawn(async move {
        mining_loop(beneficiary_clone, threads, stats_clone, Some(network), config).await;
    });

    // UI loop
    loop {
        // Check for quit key
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Event::Key(key) = event::read().unwrap() {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Draw UI
        let stats_lock = stats.lock().await.clone();
        terminal
            .draw(|f| {
                draw_miner_ui(f, &stats_lock, &beneficiary_address);
            })
            .ok();

        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    mining_handle.abort();

    Ok(())
}

async fn mining_loop(
    beneficiary_address: String,
    _threads: usize,
    stats: Arc<Mutex<MiningStats>>,
    network: Option<Arc<NetworkNode>>,
    config: Arc<Config>,
) {
    let db = Database::open(&config.database.path).expect("Failed to open database");
    let mut chain = db.load_blockchain().unwrap_or_else(|_| Blockchain::new("".to_string(), 1).expect("Failed to create new blockchain"));

    let start_time = Instant::now();
    let mut blocks_mined = 0;

    loop {
        chain = db.load_blockchain().unwrap_or_else(|_| chain.clone());

        let last_block = match chain.blocks.last() {
            Some(block) => block,
            None => {
                warn!("Blockchain is empty, waiting for genesis block...");
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let new_height = last_block.header.height + 1;
        let difficulty = chain.difficulty;
        info!("Mining block #{} with difficulty {}", new_height, difficulty);

        let coinbase_tx = Transaction::Coinbase(CoinbaseTx {
            reward_area: trinitychain::geometry::Coord::from_num(1000),
            beneficiary_address: beneficiary_address.clone(),
        });

        let mut new_block = Block::new(new_height, last_block.hash(), difficulty, vec![coinbase_tx]);

        if new_block.header.timestamp <= last_block.header.timestamp {
            new_block.header.timestamp = last_block.header.timestamp + 1;
        }

        // Update status
        {
            let mut s = stats.lock().await;
            s.mining_status = format!("Mining block #{}...", new_height);
            s.difficulty = difficulty;
        }

        let mine_start = Instant::now();

        // Mine the block
        let new_block = if _threads > 1 {
            mine_block_parallel(new_block)
        } else {
            mine_block(new_block)
        };

        let new_block = match new_block {
            Ok(block) => block,
            Err(e) => {
                warn!("Mining failed: {}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        let mine_duration = mine_start.elapsed().as_secs_f64();
        let hash_hex = hex::encode(new_block.hash());
        info!("Mined block #{} in {:.2}s with hash {}", new_height, mine_duration, hash_hex);

        if let Err(e) = chain.apply_block(new_block.clone()) {
            warn!("Failed to apply block: {}", e);
            sleep(Duration::from_secs(10)).await;
            continue;
        }

        // Broadcast block to network
        if let Some(ref network) = network {
            info!("Broadcasting block #{} to network", new_height);
            network.broadcast_block(&new_block).await;
        }

        if let Err(e) = db.save_blockchain_state(&new_block, &chain.state, chain.difficulty as u64) {
            warn!("Failed to save blockchain state: {}", e);
        }

        blocks_mined += 1;
        let elapsed = start_time.elapsed();

        // Update stats
        {
            let current_height = new_height;
            let current_supply: f64 = chain.blocks.iter().flat_map(|b| &b.transactions).filter_map(|tx| {
                if let Transaction::Coinbase(ctx) = tx {
                    Some(ctx.reward_area.to_num::<f64>())
                } else {
                    None
                }
            }).sum();
            let current_reward = Blockchain::calculate_block_reward(current_height);
            let halving_era = current_height / 210_000;
            let blocks_to_halving = ((halving_era + 1) * 210_000).saturating_sub(current_height);

            let parent_hash_hex = hex::encode(new_block.header.previous_hash);

            let mut s = stats.lock().await;
            s.blocks_mined = blocks_mined;
            s.chain_height = current_height;
            s.uptime_secs = elapsed.as_secs();
            s.avg_block_time = elapsed.as_secs_f64() / blocks_mined as f64;
            s.current_reward = current_reward;
            s.total_earned = blocks_mined as f64 * 1000.0;
            s.current_supply = current_supply;
            s.blocks_to_halving = blocks_to_halving;
            s.halving_era = halving_era;
            s.mining_status = format!("✓ Block #{} mined!", new_height);
            s.last_block_hash = hash_hex.clone();
            s.last_block_time = mine_duration;

            // Add to blockchain tree
            s.recent_blocks
                .push((current_height, hash_hex, parent_hash_hex));
            // Keep only last 10 blocks
            if s.recent_blocks.len() > 10 {
                s.recent_blocks.remove(0);
            }
        }

        sleep(Duration::from_millis(500)).await;
    }
}

fn draw_miner_ui(f: &mut ratatui::Frame, stats: &MiningStats, beneficiary_address: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(10),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("TrinityChain Miner")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let status_text = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(stats.mining_status.clone(), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Beneficiary: ", Style::default().fg(Color::Gray)),
            Span::raw(beneficiary_address),
        ]),
        Line::from(vec![
            Span::styled("Uptime: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{}s", stats.uptime_secs)),
        ]),
        Line::from(vec![
            Span::styled("Blocks Mined: ", Style::default().fg(Color::Gray)),
            Span::raw(stats.blocks_mined.to_string()),
        ]),
    ];

    let status_widget = Paragraph::new(status_text)
        .block(TuiBlock::default().borders(Borders::ALL).title("Miner Info"));
    f.render_widget(status_widget, top_chunks[0]);

    let chain_text = vec![
        Line::from(vec![
            Span::styled("Chain Height: ", Style::default().fg(Color::Gray)),
            Span::raw(stats.chain_height.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::Gray)),
            Span::raw(stats.difficulty.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Avg Block Time: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{:.2}s", stats.avg_block_time)),
        ]),
        Line::from(vec![
            Span::styled("Last Block Hash: ", Style::default().fg(Color::Gray)),
            Span::raw(stats.last_block_hash.chars().take(16).collect::<String>() + "..."),
        ]),
    ];
    let chain_widget = Paragraph::new(chain_text)
        .block(TuiBlock::default().borders(Borders::ALL).title("Blockchain"));
    f.render_widget(chain_widget, top_chunks[1]);

    let block_list: Vec<Line> = stats.recent_blocks.iter().rev().map(|(height, hash, parent)| {
        Line::from(vec![
            Span::styled(format!("#{} ", height), Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} -> {}", parent.chars().take(8).collect::<String>(), hash.chars().take(8).collect::<String>())),
        ])
    }).collect();
    let blocks_widget = Paragraph::new(block_list)
        .block(TuiBlock::default().borders(Borders::ALL).title("Recent Blocks"));
    f.render_widget(blocks_widget, chunks[2]);

    let footer = Paragraph::new("Press 'q' to quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

async fn run_balance(address: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let (_config, chain) = load_blockchain_from_config()?;
    let address_to_check = if let Some(addr) = address {
        addr.to_string()
    } else {
        let wallet_data = wallet::load_default_wallet()?;
        wallet_data.address
    };

    let mut balance = 0.0;
    for triangle in chain.state.utxo_set.values() {
        if triangle.owner == address_to_check {
            balance += triangle.effective_value().to_num::<f64>();
        }
    }

    println!("Balance for {}: {}", address_to_check, balance);
    Ok(())
}

async fn run_new_wallet(name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(wallet_name) = name {
        let wallet_path = wallet::get_named_wallet_path(wallet_name)?;
        if wallet_path.exists() {
            println!("Wallet '{}' already exists.", wallet_name);
        } else {
            wallet::create_named_wallet(wallet_name)?;
            println!("Wallet '{}' created at: {}", wallet_name, wallet_path.display());
        }
    } else {
        let wallet_path = wallet::get_default_wallet_path()?;
        if wallet_path.exists() {
            println!("Default wallet already exists.");
        } else {
            wallet::create_default_wallet()?;
            println!("Default wallet created at: {}", wallet_path.display());
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Node { peer } => {
            run_node(peer.as_deref()).await?;
        }
        Commands::MineBlock => {
            run_mine_block().await?;
        }
        Commands::Miner => {
            run_miner().await?;
        }
        Commands::Send { to_address, amount, from, memo } => {
            run_send(to_address, *amount, from.as_deref(), memo.as_deref()).await?;
        }
        Commands::History => {
            run_history().await?;
        }
        Commands::Balance { address }=> {
            run_balance(address.as_deref()).await?;
        }
        Commands::NewWallet { name }=> {
            run_new_wallet(name.as_deref()).await?;
        }
        Commands::BackupWallet => {
            println!("'backup-wallet' command not yet implemented");
        }
        Commands::RestoreWallet => {
            println!("'restore-wallet' command not yet implemented");
        }
        Commands::AddressBook => {
            println!("'address-book' command not yet implemented");
        }
        Commands::Connect => {
            println!("'connect' command not yet implemented");
        }
        Commands::Server => {
            println!("'server' command not yet implemented");
        }
        Commands::TelegramBot => {
            println!("'telegram-bot' command not yet implemented");
        }
    }

    Ok(())
}