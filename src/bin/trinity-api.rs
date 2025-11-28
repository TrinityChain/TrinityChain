use std::sync::Arc;
use trinitychain::api::{run_api_server, Node};
use trinitychain::blockchain::Blockchain;
use trinitychain::persistence::Database;

#[tokio::main]
async fn main() {
    let db = Database::open("trinitychain.db")
        .expect("Failed to open database. Ensure trinitychain.db is accessible.");

    let blockchain = db.load_blockchain().unwrap_or_else(|_| {
        println!("No existing blockchain found. Initializing genesis block...");
        let chain = Blockchain::new();
        db.save_blockchain_state(&chain.blocks[0], &chain.state, chain.difficulty)
            .expect("Failed to save genesis block to database.");
        println!("Genesis block created successfully.");
        chain
    });

    let node = Arc::new(Node::new(blockchain));

    // Start the P2P server in the background
    let network_node = node.network.clone();
    tokio::spawn(async move {
        if let Err(e) = network_node.start_server(8333).await {
            // Default port
            eprintln!("P2P server error: {}", e);
        }
    });

    println!("Starting the TrinityChain API server...");
    run_api_server(node).await;
}
