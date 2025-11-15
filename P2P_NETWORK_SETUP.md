# Siertrichain P2P Network Setup Guide (Secure)

A comprehensive step-by-step guide to set up and run a **secure distributed TrinityChain P2P network** across multiple machines with peer authentication, firewall protection, VPN support, and rate limiting.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Part 1: Initial Setup](#part-1-initial-setup)
3. [Part 2: Security Configuration](#part-2-security-configuration)
4. [Part 3: Finding Network IPs](#part-3-finding-network-ips)
5. [Part 4: Starting the Secure Network](#part-4-starting-the-secure-network)
6. [Part 5: Using the Network](#part-5-using-the-network)
7. [Part 6: Advanced Secure Configurations](#part-6-advanced-secure-configurations)
8. [Part 7: Monitoring and Troubleshooting](#part-7-monitoring-and-troubleshooting)
9. [Security Checklist](#security-checklist)

## Prerequisites

- Rust 1.90+ (stable toolchain)
- Node.js and npm (for dashboard)
- Two or more machines on the same local network
- Basic command line knowledge
- Understanding of network security concepts

## Part 1: Initial Setup

### 1.1 Clone the Repository

```bash
git clone https://github.com/littlekickoffkittie/trinitychain.git
cd trinitychain
```

### 1.2 Build the Project

```bash
cargo build --release
```

This compiles all binaries optimized for production. Takes 4-5 minutes on first run.

### 1.3 Load Shell Shortcuts (Optional but Recommended)

```bash
source ~/trinitychain/shortcuts.sh
```

For permanent setup, add to your shell config:

```bash
# For Bash
echo "source ~/trinitychain/shortcuts.sh" >> ~/.bashrc
source ~/.bashrc

# For Zsh
echo "source ~/trinitychain/shortcuts.sh" >> ~/.zshrc
source ~/.zshrc
```

Available shortcuts:
- `wallet`, `wallet-new`, `wallet-backup`, `wallet-restore`
- `send`, `balance`, `history`
- `miner`, `mine-block`
- `node`, `api`, `addressbook`
- Release variants: `miner-release`, `node-release`, `api-release`

### 1.4 Install Dashboard Dependencies (Optional)

If you want to run the web dashboard:

```bash
cd dashboard
npm install
npm run dev
```

## Part 2: Security Configuration

### 2.1 Create Encrypted Wallets

Each machine should have an encrypted wallet with a strong password.

```bash
# Create a new encrypted wallet
wallet-new

# You'll be prompted for a password
# Enter a STRONG password (16+ characters recommended)
# Example: "MySecureP@ss123!Siertri"
# Confirm the password

# Wallet saved to ~/.TrinityChain/wallet.json
```

**Password Requirements:**
- ✅ At least 16 characters
- ✅ Mix of uppercase, lowercase, numbers, symbols
- ✅ No dictionary words
- ✅ Unique per wallet

### 2.2 Backup Your Encrypted Wallet

```bash
# Create encrypted backup
wallet-backup

# Backup saved to ~/.TrinityChain/wallet_backup.json
# Store this file in a secure location offline
```

### 2.3 Configure Peer Authentication

Peer authentication is **enabled by default**. Each node will verify other nodes before accepting connections.

**To verify authentication is enabled:**

```bash
# Default: enabled
# To explicitly enable (recommended)
export SIERTRI_REQUIRE_AUTH=true

node-release 8333
```

**To disable (NOT recommended for production):**

```bash
export SIERTRI_REQUIRE_AUTH=false
node-release 8333
```

### 2.4 Configure Firewall Rules

Set up firewall rules to control which IPs can connect to your node.

**Example: Allow only specific peers**

```bash
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Allow","network":"192.168.1.235/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

node-release 8333
```

**Example: Allow entire local network**

```bash
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"127.0.0.1/8"},
  {"rule":"Allow","network":"192.168.1.0/24"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

node-release 8333
```

**Example: Development (allow all)**

```bash
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"0.0.0.0/0"}
]'

node-release 8333
```

### 2.5 Configure Rate Limiting

Rate limiting prevents abuse and DoS attacks.

**Default limits:**
- Peer requests: 100 req/sec per peer
- API requests: 50 req/sec per IP
- Transactions: 10 tx/sec per wallet

**Custom limits:**

```bash
export SIERTRI_PEER_RATE_LIMIT=200
export SIERTRI_API_RATE_LIMIT=100
export SIERTRI_TX_RATE_LIMIT=20

node-release 8333
```

### 2.6 Optional: Configure VPN Support

For enhanced privacy, route your node through a VPN.

**VPN Interface Binding:**

```bash
# Connect to OpenVPN first
sudo openvpn --config config.ovpn --daemon

# Bind node to VPN tunnel
export SIERTRI_VPN_INTERFACE=tun0
node-release 8333
```

**SOCKS5 Proxy (Tor):**

```bash
# Start Tor
tor --SocksPort 9050

# Route node through Tor
export SIERTRI_SOCKS5_PROXY=127.0.0.1:9050
node-release 8333
```

## Part 3: Finding Network IPs

Each machine needs to know its own IP and the IPs of peer nodes.

### 3.1 Find Your Local IP

**Option A: Android/Termux**
- Settings → WiFi → Connected network → IP address

**Option B: Linux/Mac**
```bash
hostname -I
# or
ifconfig | grep "inet " | grep -v "127.0.0.1"
```

**Option C: Ask Router to Scan**
```bash
nmap -sn 192.168.1.0/24
```

This shows all devices on the 192.168.1.x network with their IPs.

### 3.2 Document Your Secure Network

Create a network map with security settings:

```
Network: 192.168.1.0/24

Machine 1 (Node A - Bootstrap):
  - Local IP: 192.168.1.230
  - Port: 8333
  - Role: Bootstrap node (starts first)
  - Auth: Enabled
  - Firewall: Allow 192.168.1.0/24, Deny others
  - Rate Limit: Standard

Machine 2 (Node B - Peer):
  - Local IP: 192.168.1.235
  - Port: 8334
  - Role: Peer (connects to Node A)
  - Auth: Enabled
  - Firewall: Allow 192.168.1.230, Allow self
  - Rate Limit: Standard

Machine 3 (Node C - Peer):
  - Local IP: 192.168.1.240
  - Port: 8335
  - Role: Peer (connects to Node A)
  - Auth: Enabled
  - Firewall: Allow 192.168.1.230, Allow self
  - Rate Limit: Standard
```

## Part 4: Starting the Secure Network

### 4.1 Start the Bootstrap Node (Machine 1)

On the first machine (192.168.1.230):

```bash
# Set security configuration
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.0/24"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'
export SIERTRI_PEER_RATE_LIMIT=100
export SIERTRI_API_RATE_LIMIT=50

# Start the bootstrap node
node-release 8333
```

**Expected output:**
```
🔺 trinity-node v0.1.0
   Starting on port 8333...

📊 Current height: 684
💾 UTXO count: 2053

🌐 Node listening on 0.0.0.0:8333
🔐 Peer authentication: ENABLED
🛡️ Firewall: ENABLED (allow 192.168.1.0/24)
⚡ Rate limiting: ENABLED (100 peer req/sec)
```

**Note the blockchain height** - peer nodes will sync to this.

### 4.2 Start Peer Nodes (Machine 2, 3, etc.)

**Machine 2 (192.168.1.235):**

```bash
# Set security configuration
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Allow","network":"192.168.1.235/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

# Connect to bootstrap node
node-release 8334 --peer 192.168.1.230:8333
```

**Machine 3 (192.168.1.240):**

```bash
# Set security configuration
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Allow","network":"192.168.1.240/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

# Connect to bootstrap node
node-release 8335 --peer 192.168.1.230:8333
```

### 4.3 Verify Secure Network Connections

**Bootstrap node output should show:**
```
[INFO] New peer connection from 192.168.1.235:8334
[INFO] Authenticating peer...
[INFO] ✅ Peer authenticated successfully
[INFO] Blockchain synced with peer
```

**Peer node output should show:**
```
[INFO] Connecting to peer 192.168.1.230:8333...
[INFO] Sending authentication challenge...
[INFO] ✅ Authentication successful
[INFO] Syncing blockchain (684 blocks)...
[INFO] Sync complete
```

If you see authentication failures:
1. Check firewall rules allow the connection
2. Verify IPs are correct
3. Ensure both nodes are running
4. Check rate limit isn't exceeded

## Part 5: Using the Network

### 5.1 Create Encrypted Wallets

On each machine (or one machine for multiple wallets):

```bash
wallet-new

# Enter password when prompted
# Wallet created: ~/.TrinityChain/wallet.json
```

### 5.2 Mine Blocks

Start mining on any node to create blocks for the network:

```bash
# Single block
mine-block

# Continuous mining
miner-release
```

All connected nodes will automatically sync new blocks via authenticated peers.

### 5.3 Check Balance

```bash
balance

# Output:
# Total area: 1234.5
# Triangles owned: 3
#   - triangle_hash_1: 500.0
#   - triangle_hash_2: 400.5
#   - triangle_hash_3: 334.0
```

### 5.4 Send Transactions

```bash
send <recipient_address> <triangle_hash>
```

Example:
```bash
send abc123def456xyz789 triangle_hash_98765
```

Transaction is broadcast to all authenticated peers and included in next block.

### 5.5 View History

```bash
history

# Shows all transactions for your wallet
```

## Part 6: Advanced Secure Configurations

### 6.1 High-Security Private Network

For enterprise/sensitive deployments:

```bash
# Machine A (Bootstrap)
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Allow","network":"192.168.1.235/32"},
  {"rule":"Allow","network":"192.168.1.240/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'
export SIERTRI_PEER_RATE_LIMIT=50
export SIERTRI_API_RATE_LIMIT=25
export SIERTRI_TX_RATE_LIMIT=5

node-release 8333

# Machine B (Peer)
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

node-release 8334 --peer 192.168.1.230:8333
```

### 6.2 VPN-Protected Network

All nodes routed through VPN for privacy:

```bash
# All machines: Connect to VPN first
sudo openvpn --config production.ovpn --daemon

# Then start nodes bound to VPN interface
export SIERTRI_VPN_INTERFACE=tun0
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"10.8.0.0/24"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

node-release 8333
```

### 6.3 Tor Network (Maximum Anonymity)

Run entire network anonymously:

```bash
# All machines: Start Tor
tor --SocksPort 9050

# Then start nodes over Tor
export SIERTRI_SOCKS5_PROXY=127.0.0.1:9050
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"0.0.0.0/0"}
]'

node-release 8333
```

### 6.4 Multiple Peer Connections

Node connects to multiple peers for redundancy:

```bash
node-release 8333 \
  --peer 192.168.1.230:8333 \
  --peer 192.168.1.235:8334 \
  --peer 192.168.1.240:8335
```

### 6.5 REST API Server with Security

Start API with rate limiting:

```bash
# Set rate limit for API
export SIERTRI_API_RATE_LIMIT=100

api-release

# API runs on http://127.0.0.1:3000 (localhost only)
```

Endpoints with rate limiting:
- `GET /blockchain/height`
- `GET /blockchain/block/:hash`
- `GET /address/:addr/balance`
- `POST /transaction`
- `GET /transaction/:hash`

### 6.6 Dashboard UI with Security

Start web dashboard:

```bash
cd dashboard
npm run dev

# Access at http://localhost:5173
```

Build for production:
```bash
npm run build
npm run preview
```

## Part 7: Monitoring and Troubleshooting

### 7.1 Monitor Node Status

View real-time node status:

```bash
# Check blockchain height
balance

# Compare across nodes (should be same)
# Run on each machine and verify height matches
```

### 7.2 Check Authenticated Peers

View peers your node has authenticated:

```bash
# Check logs for authentication messages
# Look for:
# [INFO] ✅ Peer authenticated successfully
# [INFO] New peer connection from X.X.X.X:XXXX
```

### 7.3 Verify Rate Limiting

Test rate limit is working:

```bash
# Make rapid requests
for i in {1..200}; do
  curl http://localhost:3000/blockchain/height
done

# Some requests should be rate-limited after quota exceeded
```

### 7.4 Check Firewall Rules

Verify firewall is blocking unauthorized IPs:

```bash
# Try connecting from blocked IP (should fail)
# Try connecting from allowed IP (should succeed)

# Monitor logs for:
# [ERROR] Peer IP blocked by firewall
```

### 7.5 Database Inspection

Inspect the blockchain database:

```bash
sqlite3 ~/.TrinityChain/TrinityChain.db

# Then query tables:
.tables                    # List tables
SELECT COUNT(*) FROM blocks;     # Count blocks
SELECT * FROM blocks LIMIT 5;    # View first 5 blocks
SELECT COUNT(*) FROM utxo_set;   # Count UTXOs
.quit                      # Exit
```

### 7.6 Troubleshooting Authentication Failures

```bash
# If peers won't authenticate:

# 1. Check firewall allows connection
# Verify SIERTRI_FIREWALL_RULES environment variable

# 2. Check rate limit isn't exceeded
# Verify SIERTRI_PEER_RATE_LIMIT setting

# 3. Verify both nodes can reach each other
ping 192.168.1.230
ping 192.168.1.235

# 4. Check network connectivity
netstat -an | grep 8333

# 5. Verify authentication is enabled
# Look for: "Peer authentication: ENABLED"
```

### 7.7 Troubleshooting Wallet Issues

```bash
# Wrong password error
# Make sure you're entering the correct password

# Wallet not found
ls -la ~/.TrinityChain/wallet.json

# Check file permissions
chmod 600 ~/.TrinityChain/wallet.json

# Restore from backup
wallet-restore
```

## Security Checklist

**Before going live, verify:**

- [ ] All wallets created with strong passwords
- [ ] Wallet backups created and stored securely
- [ ] Peer authentication enabled (`SIERTRI_REQUIRE_AUTH=true`)
- [ ] Firewall rules configured correctly
  - [ ] Allow trusted peers only
  - [ ] Deny all by default
  - [ ] Rules tested
- [ ] Rate limiting configured
  - [ ] Peer limits set appropriately
  - [ ] API limits configured
  - [ ] Transaction limits set
- [ ] VPN configured (if using remote nodes)
  - [ ] VPN connected before starting nodes
  - [ ] `SIERTRI_VPN_INTERFACE` set correctly
- [ ] All nodes synchronized
  - [ ] Blockchain heights match
  - [ ] UTXO counts match
  - [ ] Peers authenticated
- [ ] API bound to localhost only
  - [ ] Verify listening on `127.0.0.1:3000`
  - [ ] Not exposed to public internet
- [ ] Logs monitored for errors
  - [ ] No authentication failures
  - [ ] No firewall blocks
  - [ ] No rate limit errors
- [ ] Software updated to latest version
  - [ ] `cargo build --release` successful
  - [ ] All tests pass: `cargo test`
- [ ] Incident response plan created
  - [ ] Know who to contact if compromised
  - [ ] Have backup recovery procedure
  - [ ] Know how to rotate keys

## Example: Complete 3-Node Secure Setup

### Terminal 1: Bootstrap Node (Machine A: 192.168.1.230)

```bash
ssh user@machine-a
cd ~/trinitychain

# Create encrypted wallet
wallet-new
# Enter password: "SecurePass123!@#"

# Configure and start node
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Allow","network":"192.168.1.235/32"},
  {"rule":"Allow","network":"192.168.1.240/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'
export SIERTRI_PEER_RATE_LIMIT=100

node-release 8333
```

### Terminal 2: Peer Node B (Machine B: 192.168.1.235)

```bash
ssh user@machine-b
cd ~/trinitychain

# Create encrypted wallet
wallet-new
# Enter password: "SecurePass456!@#"

# Configure and connect to bootstrap
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

node-release 8334 --peer 192.168.1.230:8333
```

### Terminal 3: Peer Node C (Machine C: 192.168.1.240)

```bash
ssh user@machine-c
cd ~/trinitychain

# Create encrypted wallet
wallet-new
# Enter password: "SecurePass789!@#"

# Configure and connect to bootstrap
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

node-release 8335 --peer 192.168.1.230:8333
```

### Terminal 4: Mining (Back on Machine A)

```bash
# Start mining
miner-release

# Blocks will be created and synced to all peers
```

### Terminal 5: Verify Sync (On Machine B)

```bash
# Check balance (should grow as blocks mined)
watch -n 1 balance

# Or check blockchain height
watch -n 1 'echo "Height: " && curl -s http://localhost:3000/blockchain/height'
```

## See Also

- **Security Guide**: [SECURITY.md](SECURITY.md)
- **Project README**: [README.md](README.md)
- **Project Status**: [PROJECT_STATUS.md](PROJECT_STATUS.md)

## Support

- **Issues**: https://github.com/littlekickoffkittie/trinitychain/issues
- **Discussions**: https://github.com/littlekickoffkittie/trinitychain/discussions
- **Security**: See [SECURITY.md](SECURITY.md) for responsible disclosure

---

**Last Updated**: November 13, 2025
**Version**: 0.2.0 (Security Release)
**Status**: Production Ready with Full Security Features

🔐 **Secure by default** | 🔺 **Peer authenticated** | 🛡️ **Firewall protected** | 🌐 **VPN ready**
