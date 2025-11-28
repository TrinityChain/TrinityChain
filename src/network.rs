//! P2P Networking for TrinityChain

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::blockchain::Blockchain;
use crate::error::ChainError;

/// Maximum message size to prevent DoS attacks (10MB)
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub host: String,
    pub port: u16,
}

impl Node {
    pub fn new(host: String, port: u16) -> Self {
        Node { host, port }
    }
    
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Manages a pool of active P2P connections
struct ConnectionPool {
    connections: RwLock<HashMap<String, Arc<RwLock<TcpStream>>>>,
}

impl ConnectionPool {
    fn new() -> Self {
        ConnectionPool {
            connections: RwLock::new(HashMap::new()),
        }
    }

    /// Add a new connection to the pool
    async fn add(&self, node: &Node, stream: TcpStream) {
        let mut connections = self.connections.write().await;
        connections.insert(node.addr(), Arc::new(RwLock::new(stream)));
    }

    /// Remove a connection from the pool
    async fn remove(&self, node: &Node) {
        let mut connections = self.connections.write().await;
        connections.remove(&node.addr());
    }

    /// Broadcast a message to all connected peers
    async fn broadcast(&self, message: &NetworkMessage) {
        let connections = self.connections.read().await;
        let data = match bincode::serialize(message) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("❌ Failed to serialize message for broadcast: {}", e);
                return;
            }
        };
        let len = data.len() as u32;

        for (addr, stream_lock) in connections.iter() {
            let mut stream = stream_lock.write().await;
            if let Err(e) = stream.write_all(&len.to_be_bytes()).await {
                eprintln!("❌ Failed to write len to {}: {}", addr, e);
                continue;
            }
            if let Err(e) = stream.write_all(&data).await {
                eprintln!("❌ Failed to write data to {}: {}", addr, e);
            }
        }
    }

    /// Get a list of all peer nodes
    async fn list_peers(&self) -> Vec<Node> {
        self.connections.read().await
            .keys()
            .map(|addr| {
                let parts: Vec<&str> = addr.split(':').collect();
                Node::new(parts[0].to_string(), parts[1].parse().unwrap_or(0))
            })
            .collect()
    }
}

pub struct NetworkNode {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pool: Arc<ConnectionPool>,
}

impl NetworkNode {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        NetworkNode {
            blockchain,
            pool: Arc::new(ConnectionPool::new()),
        }
    }
    
    pub async fn start_server(self: Arc<Self>, port: u16) -> Result<(), ChainError> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| ChainError::NetworkError(format!("Failed to bind: {}", e)))?;
        
        println!("🌐 Node listening on {}", addr);
        
        loop {
            let (socket, peer_addr) = listener.accept().await
                .map_err(|e| ChainError::NetworkError(format!("Accept error: {}", e)))?;

            println!("📡 New connection from {}", peer_addr);
            let node = Node::new(peer_addr.ip().to_string(), peer_addr.port());
            self.pool.add(&node, socket).await;

            let self_clone = self.clone();
            tokio::spawn(async move {
                if let Err(e) = self_clone.handle_connection(&node).await {
                    eprintln!("❌ Connection error with {}: {}", node.addr(), e);
                    self_clone.pool.remove(&node).await;
                }
            });
        }
    }
    
    pub async fn connect_peer(self: Arc<Self>, host: String, port: u16) -> Result<(), ChainError> {
        let addr = format!("{}:{}", host, port);
        println!("🔗 Connecting to peer: {}", addr);

        let stream = TcpStream::connect(&addr).await
            .map_err(|e| ChainError::NetworkError(format!("Failed to connect: {}", e)))?;

        let node = Node::new(host, port);
        self.pool.add(&node, stream).await;

        let self_clone = self.clone();
        tokio::spawn(async move {
            if let Err(e) = self_clone.handle_connection(&node).await {
                eprintln!("❌ Connection error with {}: {}", node.addr(), e);
                self_clone.pool.remove(&node).await;
            }
        });

        Ok(())
    }

    async fn handle_connection(&self, node: &Node) -> Result<(), ChainError> {
        let stream_lock = self.pool.connections.read().await.get(&node.addr()).cloned()
            .ok_or_else(|| ChainError::NetworkError("Connection not in pool".to_string()))?;

        loop {
            let mut stream = stream_lock.write().await;
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes).await?;
            let len = u32::from_be_bytes(len_bytes) as usize;

            if len > MAX_MESSAGE_SIZE {
                return Err(ChainError::NetworkError("Message too large".to_string()));
            }

            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer).await?;

            let message: NetworkMessage = bincode::deserialize(&buffer)?;

            match message {
                NetworkMessage::GetBlockHeaders { after_height } => {
                    let chain = self.blockchain.read().await;
                    let headers = chain.blocks.iter().filter(|b| b.header.height > after_height).map(|b| b.header.clone()).collect();
                    let response = NetworkMessage::BlockHeaders(headers);
                    self.send_message(node, &response).await?;
                }
                NetworkMessage::GetBlock(hash) => {
                    let chain = self.blockchain.read().await;
                    if let Some(block) = chain.block_index.get(&hash) {
                        let response = NetworkMessage::Block(Box::new(block.clone()));
                        self.send_message(node, &response).await?;
                    }
                }
                NetworkMessage::GetPeers => {
                    let peers = self.list_peers().await;
                    let response = NetworkMessage::Peers(peers);
                    self.send_message(node, &response).await?;
                }
                NetworkMessage::Peers(peers) => {
                    for _peer in peers {
                        // self.clone().connect_peer(peer.host, peer.port).await?;
                    }
                }
                _ => {} // Implement other message types
            }
        }
    }

    async fn send_message(&self, node: &Node, message: &NetworkMessage) -> Result<(), ChainError> {
        let stream_lock = self.pool.connections.read().await.get(&node.addr()).cloned()
            .ok_or_else(|| ChainError::NetworkError("Connection not in pool".to_string()))?;

        let data = bincode::serialize(message)?;
        let len = data.len() as u32;

        let mut stream = stream_lock.write().await;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&data).await?;
        Ok(())
    }

    pub async fn broadcast_transaction(&self, tx: &crate::transaction::Transaction) {
        let message = NetworkMessage::NewTransaction(Box::new(tx.clone()));
        self.pool.broadcast(&message).await;
    }

    pub async fn broadcast_block(&self, block: &crate::blockchain::Block) {
        let message = NetworkMessage::NewBlock(Box::new(block.clone()));
        self.pool.broadcast(&message).await;
    }

    pub async fn list_peers(&self) -> Vec<Node> {
        self.pool.list_peers().await
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    GetBlockHeaders { after_height: u64 },
    BlockHeaders(Vec<crate::blockchain::BlockHeader>),
    GetBlock(crate::blockchain::Sha256Hash),
    Block(Box<crate::blockchain::Block>),
    NewBlock(Box<crate::blockchain::Block>),
    NewTransaction(Box<crate::transaction::Transaction>),
    GetPeers,
    Peers(Vec<Node>),
}
