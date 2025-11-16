//! Peer discovery and bootstrap module
//!
//! Handles finding and connecting to peers via DNS seeds and manual configuration

use crate::error::ChainError;
use crate::network::Node;
use std::collections::HashSet;

/// DNS seed configuration for peer discovery
#[derive(Debug, Clone)]
pub struct DnsSeed {
    pub hostname: String,
    pub port: u16,
}

impl DnsSeed {
    pub fn new(hostname: String, port: u16) -> Self {
        Self { hostname, port }
    }

    /// Resolve DNS seed to peer addresses (async)
    pub async fn resolve(&self) -> Result<Vec<Node>, ChainError> {
        use std::net::ToSocketAddrs;

        let addr_str = format!("{}:{}", self.hostname, self.port);

        // For now, return a simple implementation
        // In production, you'd use async DNS lookup
        match addr_str.to_socket_addrs() {
            Ok(addrs) => {
                let nodes = addrs
                    .map(|addr| {
                        let host = addr.ip().to_string();
                        Node::new(host, self.port)
                    })
                    .collect();
                Ok(nodes)
            }
            Err(_) => Err(ChainError::NetworkError(
                format!("Failed to resolve DNS seed: {}", self.hostname),
            )),
        }
    }
}

/// Peer discovery manager
pub struct PeerDiscovery {
    dns_seeds: Vec<DnsSeed>,
    bootstrap_peers: Vec<Node>,
    known_peers: HashSet<String>,
}

impl PeerDiscovery {
    /// Create a new peer discovery manager
    pub fn new() -> Self {
        Self {
            dns_seeds: Vec::new(),
            bootstrap_peers: Vec::new(),
            known_peers: HashSet::new(),
        }
    }

    /// Add a DNS seed
    pub fn add_dns_seed(&mut self, seed: DnsSeed) {
        self.dns_seeds.push(seed);
    }

    /// Add bootstrap peers (fallback peers if DNS fails)
    pub fn add_bootstrap_peer(&mut self, peer: Node) {
        self.bootstrap_peers.push(peer);
    }

    /// Add a known peer
    pub fn add_known_peer(&mut self, peer: Node) {
        self.known_peers.insert(peer.addr());
    }

    /// Get all known peers
    pub fn get_known_peers(&self) -> Vec<Node> {
        // In real implementation, would reconstruct from HashSet
        // For now, return bootstrap peers
        self.bootstrap_peers.clone()
    }

    /// Discover peers from DNS seeds
    pub async fn discover_peers(&mut self) -> Result<Vec<Node>, ChainError> {
        let mut discovered = Vec::new();

        for seed in &self.dns_seeds {
            match seed.resolve().await {
                Ok(nodes) => {
                    for node in nodes {
                        if !self.known_peers.contains(&node.addr()) {
                            self.known_peers.insert(node.addr());
                            discovered.push(node);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to resolve DNS seed: {}", e);
                }
            }
        }

        // If no peers discovered, use bootstrap peers
        if discovered.is_empty() && !self.bootstrap_peers.is_empty() {
            for peer in &self.bootstrap_peers {
                if !self.known_peers.contains(&peer.addr()) {
                    self.known_peers.insert(peer.addr());
                    discovered.push(peer.clone());
                }
            }
        }

        Ok(discovered)
    }

    /// Get random peers for connection attempts
    pub fn get_random_peers(&self, count: usize) -> Vec<Node> {
        use rand::seq::SliceRandom;

        let mut peers: Vec<Node> = self
            .known_peers
            .iter()
            .filter_map(|addr| {
                let parts: Vec<&str> = addr.split(':').collect();
                if parts.len() == 2 {
                    let host = parts[0].to_string();
                    let port = parts[1].parse().ok()?;
                    Some(Node::new(host, port))
                } else {
                    None
                }
            })
            .collect();

        let mut rng = rand::thread_rng();
        peers.shuffle(&mut rng);
        peers.into_iter().take(count).collect()
    }

    /// Peer count
    pub fn peer_count(&self) -> usize {
        self.known_peers.len()
    }

    /// Clear all peers
    pub fn clear(&mut self) {
        self.known_peers.clear();
    }
}

impl Default for PeerDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Mainnet DNS seeds (official TrinityChain nodes)
pub fn mainnet_dns_seeds() -> Vec<DnsSeed> {
    vec![
        DnsSeed::new("seeds1.trinitychain.io".to_string(), 8333),
        DnsSeed::new("seeds2.trinitychain.io".to_string(), 8333),
        DnsSeed::new("seeds3.trinitychain.io".to_string(), 8333),
    ]
}

/// Testnet DNS seeds
pub fn testnet_dns_seeds() -> Vec<DnsSeed> {
    vec![
        DnsSeed::new("testnet-seeds1.trinitychain.io".to_string(), 18333),
        DnsSeed::new("testnet-seeds2.trinitychain.io".to_string(), 18333),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_discovery_creation() {
        let discovery = PeerDiscovery::new();
        assert_eq!(discovery.peer_count(), 0);
    }

    #[test]
    fn test_add_bootstrap_peer() {
        let mut discovery = PeerDiscovery::new();
        let peer = Node::new("127.0.0.1".to_string(), 8333);

        discovery.add_bootstrap_peer(peer.clone());
        discovery.add_known_peer(peer); // Also add to known peers
        assert_eq!(discovery.peer_count(), 1);
    }

    #[test]
    fn test_add_multiple_peers() {
        let mut discovery = PeerDiscovery::new();

        for i in 0..5 {
            let peer = Node::new(format!("127.0.0.{}", i), 8333 + i as u16);
            discovery.add_bootstrap_peer(peer.clone());
            discovery.add_known_peer(peer);
        }

        assert_eq!(discovery.peer_count(), 5);
    }

    #[test]
    fn test_duplicate_peer_not_added() {
        let mut discovery = PeerDiscovery::new();
        let peer = Node::new("127.0.0.1".to_string(), 8333);

        discovery.add_known_peer(peer.clone());
        discovery.add_known_peer(peer);

        assert_eq!(discovery.peer_count(), 1);
    }

    #[test]
    fn test_get_random_peers() {
        let mut discovery = PeerDiscovery::new();

        for i in 0..10 {
            let peer = Node::new(format!("127.0.0.{}", i), 8333);
            discovery.add_bootstrap_peer(peer.clone());
            discovery.add_known_peer(peer);
        }

        let random = discovery.get_random_peers(5);
        assert_eq!(random.len(), 5);
    }
}
