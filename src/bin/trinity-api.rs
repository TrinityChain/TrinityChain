use trinitychain::api::run_api_server;
use trinitychain::persistence::Database;
use trinitychain::blockchain::Blockchain;

#[tokio::main]
async fn main() {
    let db = Database::open("trinitychain.db").unwrap();
    if db.load_blockchain().is_err() {
        let chain = Blockchain::new();
        db.save_blockchain_state(&chain.blocks[0], &chain.state, chain.difficulty).unwrap();
    }

    println!("Starting the TrinityChain API server...");
    run_api_server().await;
}
