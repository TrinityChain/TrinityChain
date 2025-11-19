
    let miner_address = req.miner_address;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router, http::StatusCode, response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tokio::task::JoinHandle;

use crate::blockchain::{Blockchain, Block};
use crate::persistence::Database;
use crate::transaction::Transaction;
use crate::crypto::KeyPair;
use crate::miner;
use crate::network::Node;

/// Mining state that tracks the current mining operation
#[derive(Clone)]
struct MiningState {
    is_mining: Arc<AtomicBool>,
    blocks_mined: Arc<AtomicU64>,
    last_block_time: Arc<Mutex<Option<Instant>>>,
    mining_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Default for MiningState {
    fn default() -> Self {
        Self {
            is_mining: Arc::new(AtomicBool::new(false)),
            blocks_mined: Arc::new(AtomicU64::new(0)),
            last_block_time: Arc::new(Mutex::new(None)),
            mining_task: Arc::new(Mutex::new(None)),
        }
    }
}

/// Network state that tracks peers and node information
#[derive(Clone, Default)]
struct NetworkState {
    peers: Arc<Mutex<Vec<Node>>>,
    node_id: Arc<Mutex<String>>,
    listening_port: Arc<Mutex<u16>>,
}

#[derive(Clone)]
struct AppState {
    blockchain: Arc<Mutex<Blockchain>>,
    db: Arc<Mutex<Database>>,
    mining: MiningState,
    network: NetworkState,
}

pub async fn run_api_server() {
    let db = match Database::open("trinitychain.db") {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database: {}. Ensure trinitychain.db is accessible.", e);
            std::process::exit(1);
        }
    };
    let blockchain = match db.load_blockchain() {
        Ok(bc) => bc,
        Err(e) => {
            eprintln!("Failed to load blockchain from database: {}. Database may be corrupted.", e);
            std::process::exit(1);
        }
    };

    let app_state = AppState {
        blockchain: Arc::new(Mutex::new(blockchain)),
        db: Arc::new(Mutex::new(db)),
        mining: MiningState::default(),
        network: NetworkState::default(),
    };

    // Initialize network state with default values
    {
        let mut node_id = match app_state.network.node_id.lock() {
            Ok(lock) => lock,
            Err(e) => {
                eprintln!("FATAL: node_id lock is poisoned: {}", e);
                std::process::exit(1);
            }
        };
        *node_id = format!("trinity-node-{}", rand::random::<u32>());
        let mut port = match app_state.network.listening_port.lock() {
            Ok(lock) => lock,
            Err(e) => {
                eprintln!("FATAL: listening_port lock is poisoned: {}", e);
                std::process::exit(1);
            }
        };
        *port = 8333;
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        // Blockchain endpoints
        .route("/blockchain/height", get(get_blockchain_height))
        .route("/blockchain/stats", get(get_blockchain_stats))
        .route("/blockchain/blocks", get(get_recent_blocks))
        .route("/blockchain/block/:hash", get(get_block_by_hash))
        .route("/blockchain/block/by-height/:height", get(get_block_by_height))
        .route("/blockchain/reward/:height", get(get_block_reward_info))
        // Address & Balance
        .route("/address/:addr/balance", get(get_address_balance))
        .route("/address/:addr/triangles", get(get_address_triangles))
        .route("/address/:addr/history", get(get_address_history))
        // Transactions
        .route("/transaction", post(submit_transaction))
        .route("/transaction/:hash", get(get_transaction_status))
        .route("/transactions/pending", get(get_pending_transactions))
        .route("/transactions/mempool-stats", get(get_mempool_stats))
        // Wallet
        .route("/wallet/create", post(create_wallet))
        .route("/wallet/import", post(import_wallet))
        // Mining
        .route("/mining/status", get(get_mining_status))
        .route("/mining/start", post(start_mining))
        .route("/mining/stop", post(stop_mining))
        // Network
        .route("/network/peers", get(get_peers))
        .route("/network/info", get(get_network_info))
        .with_state(app_state)
        .layer(cors.clone());

    // Serve static files from dashboard directory
    let serve_dir = ServeDir::new("dashboard");

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(serve_dir)
        .layer(cors);

    // Use PORT env var (for Render.com) or default to 3000
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    // Bind to 0.0.0.0 to accept external connections (required for Render.com)
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}. Port may already be in use.", addr, e);
            std::process::exit(1);
        }
    };
    println!("API server listening on http://{}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("API server encountered a fatal error: {}", e);
    }
}

async fn get_blockchain_height(State(state): State<AppState>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    Json(blockchain.blocks.len() as u64).into_response()
}

async fn get_block_by_hash(State(state): State<AppState>, Path(hash): Path<String>) -> Result<Json<Option<Block>>, Response> {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response()),
    };
    let hash_bytes = match hex::decode(hash) {
        Ok(bytes) => bytes,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid hash format").into_response()),
    };
    let mut hash_arr = [0u8; 32];
    if hash_bytes.len() != 32 {
        return Err((StatusCode::BAD_REQUEST, "Invalid hash length").into_response());
    }
    hash_arr.copy_from_slice(&hash_bytes);
    let block = blockchain.block_index.get(&hash_arr).cloned();
    Ok(Json(block))
}

#[derive(Serialize, Deserialize)]
pub struct BalanceResponse {
    pub triangles: Vec<String>,
    pub total_area: f64,
}

#[derive(Serialize, Deserialize)]
pub struct RecentBlock {
    pub height: u64,
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct StatsResponse {
    pub height: u64,
    pub difficulty: u64,
    pub utxo_count: usize,
    pub mempool_size: usize,
    pub recent_blocks: Vec<RecentBlock>,
}

async fn get_blockchain_stats(State(state): State<AppState>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    let recent_blocks = blockchain.blocks.iter().rev().take(6).map(|b| RecentBlock {
        height: b.header.height,
        hash: hex::encode(b.hash),
    }).collect();

    Json(StatsResponse {
        height: blockchain.blocks.len() as u64,
        difficulty: blockchain.difficulty,
        utxo_count: blockchain.state.utxo_set.len(),
        mempool_size: blockchain.mempool.len(),
        recent_blocks,
    }).into_response()
}

async fn get_address_balance(State(state): State<AppState>, Path(addr): Path<String>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    let mut triangles = Vec::new();
    let mut total_area = 0.0;

    for (hash, triangle) in &blockchain.state.utxo_set {
        if triangle.owner == addr {
            triangles.push(hex::encode(hash));
            total_area += triangle.area();
        }
    }

    Json(BalanceResponse {
        triangles,
        total_area,
    }).into_response()
}

async fn submit_transaction(State(state): State<AppState>, Json(tx): Json<Transaction>) -> impl IntoResponse {
    let mut blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    let tx_hash = tx.hash_str();
    match blockchain.mempool.add_transaction(tx) {
        Ok(_) => Json(tx_hash).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, format!("Failed to add transaction: {}", e)).into_response(),
    }
}

async fn get_transaction_status(State(state): State<AppState>, Path(hash): Path<String>) -> Result<Json<Option<Transaction>>, Response> {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response()),
    };
    let hash_bytes = match hex::decode(hash) {
        Ok(bytes) => bytes,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid hash format").into_response()),
    };
    let mut hash_arr = [0u8; 32];
    if hash_bytes.len() != 32 {
        return Err((StatusCode::BAD_REQUEST, "Invalid hash length").into_response());
    }
    hash_arr.copy_from_slice(&hash_bytes);
    if let Some(tx) = blockchain.mempool.get_transaction(&hash_arr).cloned() {
        return Ok(Json(Some(tx)));
    }

    for block in &blockchain.blocks {
        if let Some(tx) = block.transactions.iter().find(|tx| tx.hash() == hash_arr) {
            return Ok(Json(Some(tx.clone())));
        }
    }

    Ok(Json(None))
}

// New endpoints

async fn get_recent_blocks(State(state): State<AppState>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    let blocks: Vec<RecentBlock> = blockchain.blocks.iter().rev().take(20).map(|b| RecentBlock {
        height: b.header.height,
        hash: hex::encode(b.hash),
    }).collect();
    Json(blocks).into_response()
}

async fn get_block_by_height(State(state): State<AppState>, Path(height): Path<u64>) -> Result<Json<Option<Block>>, Response> {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response()),
    };
    let block = blockchain.blocks.iter().find(|b| b.header.height == height).cloned();
    Ok(Json(block))
}

#[derive(Serialize, Deserialize)]
pub struct TriangleInfo {
    pub hash: String,
    pub area: f64,
    pub vertices: Vec<(f64, f64)>,
}

async fn get_address_triangles(State(state): State<AppState>, Path(addr): Path<String>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    let triangles: Vec<TriangleInfo> = blockchain.state.utxo_set.iter()
        .filter(|(_, triangle)| triangle.owner == addr)
        .map(|(hash, triangle)| TriangleInfo {
            hash: hex::encode(hash),
            area: triangle.area(),
            vertices: vec![
                (triangle.a.x, triangle.a.y),
                (triangle.b.x, triangle.b.y),
                (triangle.c.x, triangle.c.y),
            ],
        })
        .collect();
    Json(triangles).into_response()
}

#[derive(Serialize, Deserialize)]
pub struct TransactionHistory {
    pub tx_hash: String,
    pub block_height: u64,
    pub timestamp: i64,
    pub tx_type: String,
}

async fn get_address_history(State(state): State<AppState>, Path(addr): Path<String>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    let mut history = Vec::new();

    for block in &blockchain.blocks {
        for tx in &block.transactions {
            let involves_address = match tx {
                Transaction::Subdivision(tx) => tx.owner_address == addr,
                Transaction::Transfer(tx) => tx.sender == addr || tx.new_owner == addr,
                Transaction::Coinbase(tx) => tx.beneficiary_address == addr,
            };

            if involves_address {
                history.push(TransactionHistory {
                    tx_hash: tx.hash_str(),
                    block_height: block.header.height,
                    timestamp: block.header.timestamp,
                    tx_type: match tx {
                        Transaction::Subdivision(_) => "Subdivision".to_string(),
                        Transaction::Transfer(_) => "Transfer".to_string(),
                        Transaction::Coinbase(_) => "Coinbase".to_string(),
                    },
                });
            }
        }
    }

    Json(history).into_response()
}

async fn get_pending_transactions(State(state): State<AppState>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    Json(blockchain.mempool.get_all_transactions()).into_response()
}

#[derive(Serialize, Deserialize)]
pub struct WalletResponse {
    pub address: String,
    pub public_key: String,
    pub private_key: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StartMiningRequest {
    pub miner_address: String,
}



async fn create_wallet() -> Result<Json<WalletResponse>, Response> {
    match KeyPair::generate() {
        Ok(keypair) => {
            let address = keypair.address();
            let public_key = hex::encode(keypair.public_key.serialize());
            let private_key = hex::encode(keypair.secret_key.secret_bytes());

            Ok(Json(WalletResponse {
                address,
                public_key,
                private_key,
            }))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate keypair: {}", e)).into_response()),
    }
}

#[derive(Serialize, Deserialize)]
pub struct ImportWalletRequest {
    pub private_key: String,
}

async fn import_wallet(Json(req): Json<ImportWalletRequest>) -> Result<Json<WalletResponse>, Response> {
    let private_key_bytes = match hex::decode(&req.private_key) {
        Ok(bytes) => bytes,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid private key format").into_response()),
    };

    match KeyPair::from_secret_bytes(&private_key_bytes) {
        Ok(keypair) => {
            let address = keypair.address();
            let public_key = hex::encode(keypair.public_key.serialize());

            Ok(Json(WalletResponse {
                address,
                public_key,
                private_key: req.private_key,
            }))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("Invalid private key: {}", e)).into_response()),
    }
}

#[derive(Serialize, Deserialize)]
pub struct MiningStatus {
    pub is_mining: bool,
    pub blocks_mined: u64,
    pub hashrate: f64,
}

async fn get_mining_status(State(state): State<AppState>) -> impl IntoResponse {
    let is_mining = state.mining.is_mining.load(Ordering::Relaxed);
    let blocks_mined = state.mining.blocks_mined.load(Ordering::Relaxed);

    // Calculate approximate hashrate based on last block time
    let hashrate = if is_mining {
        let last_time = match state.mining.last_block_time.lock() {
            Ok(lock) => lock,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get mining state lock").into_response(),
        };
        if let Some(instant) = *last_time {
            let elapsed = instant.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                // Estimate based on difficulty and time
                let blockchain = match state.blockchain.lock() {
                    Ok(lock) => lock,
                    Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
                };
                let difficulty = blockchain.difficulty;
                let expected_hashes = 16_u64.pow(difficulty as u32) as f64;
                expected_hashes / elapsed
            } else {
                0.0
            }
        } else {
            0.0
        }
    } else {
        0.0
    };

    Json(MiningStatus {
        is_mining,
        blocks_mined,
        hashrate,
    }).into_response()
}

async fn start_mining(State(state): State<AppState>, Json(req): Json<StartMiningRequest>) -> impl IntoResponse {
    // Check if already mining
    if state.mining.is_mining.load(Ordering::Relaxed) {
        return (StatusCode::BAD_REQUEST, "Mining already in progress").into_response();
    }

    // Get a wallet address for mining rewards
    let wallet_path = std::env::var("HOME").unwrap_or_else(|_| ".".to_string()) + "/.trinitychain/wallet.json";
    let wallet_data = match std::fs::read_to_string(&wallet_path) {
        Ok(data) => data,
        Err(_) => return (StatusCode::BAD_REQUEST, "No wallet found. Create a wallet first using trinity-wallet-new").into_response(),
    };

    let wallet: serde_json::Value = match serde_json::from_str(&wallet_data) {
        Ok(w) => w,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid wallet format: {}", e)).into_response(),
    };

    let miner_address = match wallet.get("address").and_then(|a| a.as_str()) {
        Some(addr) => addr.to_string(),
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "Wallet missing address").into_response(),
    };

    // Set mining flag
    state.mining.is_mining.store(true, Ordering::Relaxed);

    // Spawn mining task
    let blockchain_clone = state.blockchain.clone();
    let db_clone = state.db.clone();
    let mining_state = state.mining.clone();

    let task = tokio::spawn(async move {
        loop {
            // Check if we should stop
            if !mining_state.is_mining.load(Ordering::Relaxed) {
                break;
            }

            // Get pending transactions
            let block = {
                let blockchain = match blockchain_clone.lock() {
                    Ok(lock) => lock,
                    Err(e) => {
                        eprintln!("Failed to acquire blockchain lock in mining task: {}", e);
                        mining_state.is_mining.store(false, Ordering::Relaxed); // Stop mining
                        break;
                    }
                };
                let transactions = blockchain.mempool.get_all_transactions();

                // Create coinbase transaction
                let reward_area = 100u64;
                let coinbase = Transaction::Coinbase(crate::transaction::CoinbaseTx {
                    reward_area,
                    beneficiary_address: miner_address.clone(),
                });

                let mut all_txs = vec![coinbase];
                all_txs.extend(transactions);

                let height = blockchain.blocks.len() as u64;
                let previous_hash = blockchain.blocks.last().expect("Blockchain should have at least a genesis block").hash;
                let difficulty = blockchain.difficulty;

                Block::new(height, previous_hash, difficulty, all_txs)
            };

            // Mine the block (this is CPU intensive)
            let start = Instant::now();
            match miner::mine_block(block) {
                Ok(mined_block) => {
                    // Update last block time
                    {
                        let mut last_time = match mining_state.last_block_time.lock() {
                            Ok(lock) => lock,
                            Err(e) => {
                                eprintln!("Failed to acquire mining state lock for last_block_time: {}", e);
                                mining_state.is_mining.store(false, Ordering::Relaxed); // Stop mining
                                break;
                            }
                        };
                        *last_time = Some(start);
                    }

                    // Add block to blockchain
                    {
                        let mut blockchain = match blockchain_clone.lock() {
                            Ok(lock) => lock,
                            Err(e) => {
                                eprintln!("Failed to acquire blockchain lock for applying block: {}", e);
                                mining_state.is_mining.store(false, Ordering::Relaxed); // Stop mining
                                break;
                            }
                        };
                        if let Err(e) = blockchain.apply_block(mined_block.clone()) {
                            eprintln!("Failed to apply mined block: {}", e);
                            continue;
                        }

                        // Save to database
                        let db = match db_clone.lock() {
                            Ok(lock) => lock,
                            Err(e) => {
                                eprintln!("Failed to acquire database lock for saving block: {}", e);
                                mining_state.is_mining.store(false, Ordering::Relaxed); // Stop mining
                                break;
                            }
                        };
                        if let Err(e) = db.save_block(&mined_block) {
                            eprintln!("Failed to save block: {}", e);
                        }
                        if let Err(e) = db.save_utxo_set(&blockchain.state) {
                            eprintln!("Failed to save UTXO set: {}", e);
                        }
                    }

                    // Increment blocks mined counter
                    mining_state.blocks_mined.fetch_add(1, Ordering::Relaxed);

                    println!("✅ Mined block at height {}", mined_block.header.height);
                }
                Err(e) => {
                    eprintln!("Mining error: {}", e);
                    mining_state.is_mining.store(false, Ordering::Relaxed); // Stop mining on mining error
                    break;
                }
            }
        }

        println!("Mining stopped");
    });

    // Store the task handle
    {
        let mut task_handle = match state.mining.mining_task.lock() {
            Ok(lock) => lock,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to acquire mining task lock: {}", e)).into_response(),
        };
        *task_handle = Some(task);
    }

    Json("Mining started successfully".to_string()).into_response()
}

async fn stop_mining(State(state): State<AppState>) -> impl IntoResponse {
    // Check if mining is active
    if !state.mining.is_mining.load(Ordering::Relaxed) {
        return (StatusCode::BAD_REQUEST, "Mining is not active").into_response();
    }

    // Signal the mining task to stop
    state.mining.is_mining.store(false, Ordering::Relaxed);

    // Wait for the task to complete (with timeout)
    let task_handle = match state.mining.mining_task.lock() {
        Ok(mut lock) => lock.take(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to acquire mining task lock: {}", e)).into_response(),
    };
    if let Some(handle) = task_handle {
        // Wait up to 5 seconds for the task to finish
        match tokio::time::timeout(Duration::from_secs(5), handle).await {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Warning: Mining task didn't stop within timeout");
            }
        }
    }

    Json("Mining stopped successfully".to_string()).into_response()
}

#[derive(Serialize, Deserialize)]
pub struct PeerInfo {
    pub address: String,
    pub last_seen: i64,
}

async fn get_peers(State(state): State<AppState>) -> impl IntoResponse {
    let peers = match state.network.peers.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get network peers lock").into_response(),
    };
    let peer_info: Vec<PeerInfo> = peers.iter().map(|peer| PeerInfo {
        address: peer.addr(),
        last_seen: chrono::Utc::now().timestamp(), // In a real implementation, track actual last seen time
    }).collect();
    Json(peer_info).into_response()
}

#[derive(Serialize, Deserialize)]
pub struct NetworkInfo {
    pub peers_count: usize,
    pub node_id: String,
    pub listening_port: u16,
}

async fn get_network_info(State(state): State<AppState>) -> impl IntoResponse {
    let peers = match state.network.peers.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get network peers lock").into_response(),
    };
    let node_id = match state.network.node_id.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get node ID lock").into_response(),
    };
    let listening_port = match state.network.listening_port.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get listening port lock").into_response(),
    };

    Json(NetworkInfo {
        peers_count: peers.len(),
        node_id: node_id.clone(),
        listening_port: *listening_port,
    }).into_response()
}

// New endpoints for enhanced block explorer functionality

#[derive(Serialize)]
struct MempoolStatsResponse {
    transaction_count: usize,
    total_fees: u64,
    avg_fee: f64,
    highest_fee: u64,
    lowest_fee: u64,
}

async fn get_mempool_stats(State(state): State<AppState>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    let txs = blockchain.mempool.get_all_transactions();

    let fees: Vec<u64> = txs.iter().map(|tx| tx.fee()).collect();
    let total_fees: u64 = fees.iter().sum();
    let avg_fee = if !fees.is_empty() {
        total_fees as f64 / fees.len() as f64
    } else {
        0.0
    };
    let highest_fee = fees.iter().max().copied().unwrap_or(0);
    let lowest_fee = fees.iter().min().copied().unwrap_or(0);

    Json(MempoolStatsResponse {
        transaction_count: txs.len(),
        total_fees,
        avg_fee,
        highest_fee,
        lowest_fee,
    }).into_response()
}

#[derive(Serialize)]
struct RewardInfoResponse {
    current_height: u64,
    current_reward: u64,
    next_halving_height: u64,
    blocks_until_halving: u64,
    reward_after_halving: u64,
}

async fn get_block_reward_info(State(state): State<AppState>, Path(height): Path<u64>) -> impl IntoResponse {
    let blockchain = match state.blockchain.lock() {
        Ok(lock) => lock,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get blockchain lock").into_response(),
    };
    let current_height = blockchain.blocks.len() as u64;
    let query_height = if height == 0 { current_height } else { height };

    let current_reward = Blockchain::calculate_block_reward(query_height);
    let halving_interval = 210_000u64;
    let next_halving_height = ((query_height / halving_interval) + 1) * halving_interval;
    let blocks_until_halving = next_halving_height.saturating_sub(query_height);
    let reward_after_halving = Blockchain::calculate_block_reward(next_halving_height);

    Json(RewardInfoResponse {
        current_height: query_height,
        current_reward,
        next_halving_height,
        blocks_until_halving,
        reward_after_halving,
    }).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;

    fn test_app() -> Router {
        let blockchain = Blockchain::new();
        let db = match Database::open(":memory:") {
            Ok(db) => db,
            Err(e) => panic!("Failed to open in-memory database for test: {}", e),
        };

        let app_state = AppState {
            blockchain: Arc::new(Mutex::new(blockchain)),
            db: Arc::new(Mutex::new(db)),
            mining: MiningState::default(),
            network: NetworkState::default(),
        };

        Router::new()
            .route("/blockchain/height", get(get_blockchain_height))
            .route("/blockchain/stats", get(get_blockchain_stats)) // Added missing route
            .route("/blockchain/block/:hash", get(get_block_by_hash))
            .route("/address/:addr/balance", get(get_address_balance))
            .route("/transaction", post(submit_transaction))
            .route("/transaction/:hash", get(get_transaction_status))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_get_blockchain_height() {
        let server = TestServer::new(test_app()).expect("Test server setup failed");
        let response = server.get("/blockchain/height").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.json::<u64>(), 1);
    }

    #[tokio::test]
    async fn test_get_block_by_hash() {
        let server = TestServer::new(test_app()).expect("Test server setup failed");
        
        // First, get stats to find the genesis block
        let stats_response = server.get("/blockchain/stats").await;
        assert_eq!(stats_response.status_code(), StatusCode::OK);
        let stats: StatsResponse = stats_response.json();
        
        // The genesis block should be in recent_blocks (last one, since it's height 0)
        let genesis_block_info = stats.recent_blocks.last().expect("Should have genesis block");
        let genesis_hash = &genesis_block_info.hash;
        
        // Get the genesis block using its actual hash
        let response = server.get(&format!("/blockchain/block/{}", genesis_hash)).await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let block: Option<Block> = response.json();
        assert!(block.is_some());
        let block = block.expect("Block should be present");
        assert_eq!(block.header.height, 0);
    }

    use crate::transaction::SubdivisionTx;
    use crate::crypto::KeyPair;

    #[tokio::test]
    async fn test_get_address_balance() {
        let server = TestServer::new(test_app()).expect("Test server setup failed");
        let genesis_owner = "genesis_owner";
        let response = server.get(&format!("/address/{}/balance", genesis_owner)).await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let balance: BalanceResponse = response.json();
        assert_eq!(balance.triangles.len(), 1);
        assert!(balance.total_area > 0.0);
    }

    #[tokio::test]
    async fn test_submit_and_get_transaction() {
        let server = TestServer::new(test_app()).expect("Test server setup failed");
        let blockchain = Blockchain::new();
        let _genesis = blockchain.blocks[0].clone();
        let keypair = KeyPair::generate().expect("Keypair generation should succeed in test");
        let address = keypair.address();
        let parent_hash = *blockchain.state.utxo_set.keys().next().expect("UTXO set should not be empty in test");
        let children = blockchain.state.utxo_set.values().next().expect("UTXO set should not be empty in test").subdivide();
        let mut tx = SubdivisionTx::new(parent_hash, children.to_vec(), address, 0, 1);
        let message = tx.signable_message();
        let signature = keypair.sign(&message).expect("Signing message should succeed in test");
        let public_key = keypair.public_key.serialize().to_vec();
        tx.sign(signature, public_key);
        let transaction = Transaction::Subdivision(tx);

        let response = server.post("/transaction").json(&transaction).await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let tx_hash: String = response.json();
        assert!(!tx_hash.is_empty());

        let response = server.get(&format!("/transaction/{}", tx_hash)).await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let tx_status: Option<Transaction> = response.json();
        assert!(tx_status.is_some());
    }
}
