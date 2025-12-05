use trinitychain::blockchain::{Blockchain, Block};
use trinitychain::crypto::address_from_string;
use trinitychain::transaction::{Transaction, CoinbaseTx};
use trinitychain::persistence::Database;
use trinitychain::miner::mine_block;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <wallet_name>", args[0]);
        return Ok(());
    }
    let wallet_name = &args[1];

    let db = Database::open("trinitychain.db")?;
    let mut chain = db.load_blockchain().unwrap_or_else(|_| {
        println!("No chain found – creating genesis");
        Blockchain::new(address_from_string(wallet_name), 1).unwrap()
    });

    let last_block = chain.blocks.last().cloned().unwrap();
    let new_height = last_block.header.height + 1;

    let coinbase_tx = Transaction::Coinbase(CoinbaseTx {
        reward_area: trinitychain::geometry::Coord::from_num(1000),
        beneficiary_address: address_from_string(wallet_name),
    });

    let transactions = vec![coinbase_tx];

    let mut new_block = Block::new(
        new_height,
        last_block.hash(),
        chain.difficulty,
        transactions,
    );

    if new_block.header.timestamp <= last_block.header.timestamp {
        new_block.header.timestamp = last_block.header.timestamp + 1;
    }

    println!("Mining block {}...", new_block.header.height);
    let new_block = mine_block(new_block)?;

    chain.apply_block(new_block.clone())?;
    db.save_blockchain_state(&new_block, &chain.state, chain.difficulty as u64)?;
    
    println!("Mined block {}", new_block.header.height);
    println!("   Hash : {}", hex::encode(new_block.hash()));
    println!("   Nonce: {}", new_block.header.nonce);
    println!("   UTXOs: {}", chain.state.utxo_set.len());

    Ok(())
}