use trinitychain::blockchain::{Blockchain, Block};
use trinitychain::persistence::Database;
use trinitychain::transaction::{Transaction, CoinbaseTx};
use std::env;
use std::time::{Duration, Instant};

fn print_usage() {
    println!("Usage: trinity-headless-miner <beneficiary_address> [--blocks N] [--threads N]");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let beneficiary = args[1].clone();
    let mut blocks_to_mine: usize = 1;
    let mut threads: usize = 1;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--blocks" && i + 1 < args.len() {
            if let Ok(n) = args[i+1].parse::<usize>() { blocks_to_mine = n; }
            i += 2;
        } else if (args[i] == "--threads" || args[i] == "-t") && i + 1 < args.len() {
            if let Ok(n) = args[i+1].parse::<usize>() { threads = n.max(1); }
            i += 2;
        } else {
            i += 1;
        }
    }

    println!("Starting headless miner: beneficiary={} blocks={} threads={}", beneficiary, blocks_to_mine, threads);

    let db = Database::open("trinitychain.db").expect("Failed to open database");
    let mut chain = db.load_blockchain().unwrap_or_else(|_| Blockchain::new());

    let mut mined = 0usize;
    while mined < blocks_to_mine {
        chain = db.load_blockchain().unwrap_or_else(|_| chain.clone());
        let last_block = chain.blocks.last().expect("chain must have genesis");
        let new_height = last_block.header.height + 1;
        let difficulty = chain.difficulty;

        let coinbase_tx = Transaction::Coinbase(CoinbaseTx {
            reward_area: trinitychain::geometry::Coord::from_num(1000),
            beneficiary_address: beneficiary.clone(),
        });

        let mut new_block = Block::new(new_height, last_block.hash, difficulty, vec![coinbase_tx]);
        if new_block.header.timestamp <= last_block.header.timestamp {
            new_block.header.timestamp = last_block.header.timestamp + 1;
        }

        let mine_start = Instant::now();
        loop {
            new_block.hash = new_block.calculate_hash();
            if new_block.verify_proof_of_work() {
                break;
            }
            new_block.header.nonce += 1;
        }

        let mine_duration = mine_start.elapsed();
        println!("Mined block #{} in {:.2?} (nonce={})", new_height, mine_duration, new_block.header.nonce);

        if let Err(e) = chain.apply_block(new_block.clone()) {
            eprintln!("Failed to apply block: {}", e);
            break;
        }

        if let Err(e) = db.save_blockchain_state(&new_block, &chain.state, chain.difficulty) {
            eprintln!("Failed to save blockchain state: {}", e);
        }

        mined += 1;
    }

    println!("Headless miner finished, mined {} block(s)", mined);
}
