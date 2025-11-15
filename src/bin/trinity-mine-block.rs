//! Mine a new block by subdividing a triangle

use trinitychain::persistence::Database;
use trinitychain::transaction::{Transaction, SubdivisionTx, CoinbaseTx};
use trinitychain::crypto::KeyPair;
use trinitychain::miner::mine_block;
use secp256k1::SecretKey;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⛏️  Mining Block...\n");

    let db = Database::open("trinitychain.db")?;
    let mut chain = db.load_blockchain()?;

    let current_height = chain.blocks.last()
        .map(|b| b.header.height)
        .ok_or("Blockchain is empty")?;
    println!("📊 Current height: {}", current_height);

    let home = std::env::var("HOME")?;
    let wallet_file = format!("{}/.trinitychain/wallet.json", home);

    let wallet_content = std::fs::read_to_string(&wallet_file)?;
    let wallet_data: serde_json::Value = serde_json::from_str(&wallet_content)?;

    let address = wallet_data["address"].as_str()
        .ok_or("Wallet address not found")?
        .to_string();
    let secret_hex = wallet_data["secret_key"].as_str()
        .ok_or("Secret key not found")?;
    let secret_bytes = hex::decode(secret_hex)?;
    let secret_key = SecretKey::from_slice(&secret_bytes)?;
    let keypair = KeyPair::from_secret_key(secret_key);

    let parent_hash = *chain.state.utxo_set.keys().next()
        .ok_or("No UTXOs available")?;
    let parent_triangle = chain.state.utxo_set.get(&parent_hash)
        .ok_or("Parent triangle not found")?
        .clone();

    let hash_hex = hex::encode(parent_hash);
    let hash_prefix = &hash_hex[..16];
    println!("🔺 Subdividing triangle {}...", hash_prefix);
    let children = parent_triangle.subdivide();

    let mut tx = SubdivisionTx::new(parent_hash, children.to_vec(), address.clone(), 0, chain.blocks.len() as u64);
    let message = tx.signable_message();
    let signature = keypair.sign(&message)?;
    let public_key = keypair.public_key.serialize().to_vec();
    tx.sign(signature, public_key);

    let coinbase = CoinbaseTx { reward_area: 1000, beneficiary_address: address };

    // Include pending transactions from mempool (prioritized by fee)
    let mempool_txs = chain.mempool.get_transactions_by_fee(100); // Get up to 100 highest-fee transactions

    let mut transactions = vec![Transaction::Coinbase(coinbase)];

    // Add mempool transactions first (they have fees!)
    transactions.extend(mempool_txs);

    // Then add our subdivision transaction
    transactions.push(Transaction::Subdivision(tx));

    println!("⛏️  Mining block (difficulty {})...", chain.difficulty);

    let last_block = chain.blocks.last()
        .ok_or("Blockchain is empty")?;
    let mut new_block = trinitychain::blockchain::Block::new(
        last_block.header.height + 1,
        last_block.hash,
        chain.difficulty,
        transactions,
    );

    new_block = mine_block(new_block)?;

    let new_hash_hex = hex::encode(new_block.hash);
    let new_hash_prefix = &new_hash_hex[..16];
    println!("✅ Block mined! Hash: {}", new_hash_prefix);

    chain.apply_block(new_block.clone())?;

    db.save_block(&new_block)?;
    db.save_utxo_set(&chain.state)?;

    println!("\n🎉 Block {} mined successfully!", chain.blocks.len() - 1);
    println!("   UTXOs: {}", chain.state.count());

    Ok(())
}
