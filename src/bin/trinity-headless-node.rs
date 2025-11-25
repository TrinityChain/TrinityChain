use trinitychain::blockchain::Blockchain;
use trinitychain::persistence::Database;
use trinitychain::network::NetworkNode;
use trinitychain::discovery::{PeerDiscovery, mainnet_dns_seeds};
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: trinity-headless-node <port> [--peer host:port]");
        return Ok(());
    }

    let port: u16 = args[1].parse().expect("Invalid port number");
    let mut bootstrap: Option<String> = None;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--peer" && i + 1 < args.len() {
            bootstrap = Some(args[i+1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let db_path = "trinitychain.db".to_string();
    let db = Database::open(&db_path).expect("Failed to open database");
    let blockchain = db.load_blockchain().unwrap_or_else(|_| Blockchain::new());

    let node = NetworkNode::new(blockchain, db_path.clone());

    // Start server
    let server = node.clone();
    tokio::spawn(async move {
        if let Err(e) = server.start_server(9090).await {
            eprintln!("Node server error: {}", e);
        }
    });

    // Discovery
    let discover_node = node.clone();
    tokio::spawn(async move {
        let mut discovery = PeerDiscovery::new();
        for seed in mainnet_dns_seeds() {
            discovery.add_dns_seed(seed);
        }

        // Add bootstrap if provided
        if let Some(peer) = bootstrap {
            if let Some((host, port_s)) = peer.split_once(':') {
                if let Ok(p) = port_s.parse::<u16>() {
                    discovery.add_bootstrap_peer(trinitychain::network::Node::new(host.to_string(), p));
                }
            }
        }

        loop {
            match discovery.discover_peers().await {
                Ok(peers) => {
                    for peer in peers {
                        let host = peer.host.clone();
                        let port = peer.port;
                        let n = discover_node.clone();
                        tokio::spawn(async move {
                            if let Err(e) = n.connect_peer(host, port).await {
                                eprintln!("Auto-connect failed: {}", e);
                            }
                        });
                    }
                }
                Err(e) => eprintln!("Peer discovery failed: {}", e),
            }
            sleep(Duration::from_secs(30)).await;
        }
    });

    // Periodic status
    let stats_node = node.clone();
    tokio::spawn(async move {
        loop {
            let peers = stats_node.peers_count().await;
            let height = stats_node.get_height().await;
            println!("[node] height={} peers={}", height, peers);
            sleep(Duration::from_secs(10)).await;
        }
    });

    // Keep running
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
