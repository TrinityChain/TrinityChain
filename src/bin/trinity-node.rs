//! Network node for TrinityChain

use trinitychain::blockchain::Blockchain;
use trinitychain::persistence::Database;
use trinitychain::network::NetworkNode;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let port: u16 = args[1].parse().expect("Invalid port number");
    let db_path = "trinitychain.db".to_string();
    
    println!("🔺 trinity-node v0.1.0");
    println!("   Starting on port {}...\n", port);
    
    let db = Database::open(&db_path).expect("Failed to open database");
    let blockchain = db.load_blockchain().unwrap_or_else(|_| {
        println!("⚠️  No blockchain found, creating genesis...");
        Blockchain::new()
    });
    
    println!("📊 Current height: {}", blockchain.blocks.last().unwrap().header.height);
    println!("💾 UTXO count: {}\n", blockchain.state.count());
    
    let node = NetworkNode::new(blockchain, db_path);
    
    if args.len() >= 4 && args[2] == "--peer" {
        let peer_addr = &args[3];
        let parts: Vec<&str> = peer_addr.split(':').collect();
        if parts.len() == 2 {
            let peer_host = parts[0].to_string();
            let peer_port: u16 = parts[1].parse().expect("Invalid peer port");
            
            println!("🔗 Connecting to peer {}:{}...", peer_host, peer_port);
            if let Err(e) = node.connect_peer(peer_host, peer_port).await {
                eprintln!("❌ Failed to connect to peer: {}", e);
            }
        }
    }
    
    println!("🌐 Ready to accept connections!\n");
    if let Err(e) = node.start_server(port).await {
        eprintln!("❌ Server error: {}", e);
    }
}

fn print_usage() {
    println!("Usage: trinity-node <port> [--peer <host:port>]");
    println!("\nExamples:");
    println!("  trinity-node 8333");
    println!("  trinity-node 8334 --peer 192.168.1.100:8333");
}
