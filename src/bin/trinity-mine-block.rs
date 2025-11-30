//! Mine a new block by subdividing a triangle

use clap::Parser;
use secp256k1::SecretKey;
use std::collections::HashSet;
use trinitychain::config::load_config;
use trinitychain::crypto::KeyPair;
use trinitychain::miner::{mine_block, mine_block_parallel};
use trinitychain::persistence::Database;
use trinitychain::transaction::{CoinbaseTx, SubdivisionTx, Transaction};
use trinitychain::wallet;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The name of the wallet to use for mining
    #[arg(long)]
    wallet: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    println!("⛏️  Mining Block...\n");

    let config = load_config()?;
    let threads = config.miner.threads;

    // Load wallet first to get the beneficiary address
    let wallet_data = match cli.wallet {
        Some(name) => {
            println!("👛 Using named wallet: {}", name);
            wallet::load_named_wallet(&name)?
        }
        None => {
            println!("👛 Using default wallet");
            let wallet_path = wallet::get_default_wallet_path()?;
            if !wallet_path.exists() {
                println!("   -> No default wallet found. Creating a new one...");
                wallet::create_default_wallet()?;
                println!("   -> New wallet created at: {}", wallet_path.display());
            }
            wallet::load_default_wallet()?
        }
    };

    let address = wallet_data.address.clone();
    let secret_hex = wallet_data.secret_key_hex;
    let secret_bytes = hex::decode(secret_hex)?;
    let secret_key = SecretKey::from_slice(&secret_bytes)?;
    let keypair = KeyPair::from_secret_key(secret_key);

    // Now load the blockchain, using the wallet's address for the genesis block if needed
    let db = Database::open(&config.database.path)?;
    let mut chain = db.load_blockchain().unwrap_or_else(|_| {
        trinitychain::blockchain::Blockchain::new(address.clone(), 1)
            .expect("Failed to create new blockchain")
    });

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

    let mut mempool_txs = chain.mempool.get_all_transactions();
    mempool_txs.sort_by_key(|b| std::cmp::Reverse(b.fee()));
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
        .ok_or("Parent triangle not found")?
        .clone();

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
