use trinitychain::api::run_api_server;
use trinitychain::persistence::Database;
use trinitychain::blockchain::Blockchain;

#[tokio::main]
async fn main() {
    let db = Database::open("trinitychain.db")
        .expect("Failed to open database. Ensure trinitychain.db is accessible.");

    if db.load_blockchain().is_err() {
        println!("No existing blockchain found. Initializing genesis block...");
        let chain = Blockchain::new();
        db.save_blockchain_state(&chain.blocks[0], &chain.state, chain.difficulty)
            .expect("Failed to save genesis block to database.");
        println!("Genesis block created successfully.");
    }

    println!("Starting the TrinityChain API server...");
    run_api_server().await;
}
