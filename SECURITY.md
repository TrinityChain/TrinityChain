# Siertrichain Security Implementation Guide

This document describes the comprehensive security features implemented in TrinityChain v0.1.0+, including wallet encryption, peer authentication, firewall rules, VPN support, and rate limiting.

## Table of Contents

1. [Wallet Encryption](#wallet-encryption)
2. [Peer Authentication](#peer-authentication)
3. [Firewall Rules](#firewall-rules)
4. [VPN Support](#vpn-support)
5. [Rate Limiting](#rate-limiting)
6. [Security Best Practices](#security-best-practices)
7. [Configuration Examples](#configuration-examples)

## Wallet Encryption

### Overview

Wallet encryption uses **AES-256-GCM** with **Argon2** password-based key derivation for maximum security. All wallet files are encrypted by default.

### How It Works

1. **Key Derivation**: Your password is hashed using Argon2 (memory-hard, resistant to GPU attacks)
2. **Encryption**: Secret key is encrypted with AES-256-GCM (authenticated encryption)
3. **Storage**: Encrypted wallet stored as JSON with:
   - `encrypted_secret_key`: Base64-encoded ciphertext
   - `salt`: Salt for Argon2 derivation
   - `nonce`: Random nonce for GCM
   - `address`: Your wallet address (unencrypted for identification)

### Creating an Encrypted Wallet

```bash
# Create new wallet with encryption
wallet-new

# You'll be prompted for a password
# Enter a strong password (16+ characters recommended)
# Confirm the password

# Wallet saved to ~/.TrinityChain/wallet.json
```

### Loading an Encrypted Wallet

```bash
# Load wallet (you'll be prompted for password)
wallet

# Enter password when prompted
```

### Encryption Parameters

Default Argon2 parameters (secure and resistant to attacks):
- Memory: 19MB
- Iterations: 2
- Parallelism: 1

To customize, set environment variables:

```bash
export SIERTRI_ARGON2_MEMORY=65540
export SIERTRI_ARGON2_ITERATIONS=3
export SIERTRI_ARGON2_PARALLELISM=2
```

### Wallet Backup and Restore

Backups are encrypted with the same password as your wallet:

```bash
# Create encrypted backup
wallet-backup

# Restore from backup
wallet-restore
```

## Peer Authentication

### Overview

Peers must authenticate with each other using **ECDSA signatures** before data exchange. This prevents:
- Unauthorized peer connections
- Man-in-the-middle attacks
- Peer spoofing

### How Authentication Works

1. **Challenge**: Node A sends a random 32-byte challenge to Node B
2. **Response**: Node B signs the challenge with its private key and returns signature
3. **Verification**: Node A verifies the signature against Node B's public key
4. **Trust**: Peer is marked as authenticated and trusted

### Enabling Peer Authentication

Peer authentication is **enabled by default**. To disable (not recommended for production):

```bash
export SIERTRI_REQUIRE_AUTH=false
node-release 8333
```

### Monitoring Authenticated Peers

Peers are tracked with reputation scores:

```rust
struct PeerIdentity {
    address: String,
    public_key: Vec<u8>,
    authenticated: bool,
    failed_attempts: u32,
    last_seen: u64,
}
```

Peers with 3+ failed authentication attempts are blocked.

### Public Key Exchange

Each node's public key is included in authentication messages. Store trusted peer public keys:

```bash
# Save peer's public key for reference
mkdir -p ~/.TrinityChain/trusted_peers
echo "peer1_pubkey" > ~/.TrinityChain/trusted_peers/peer1.pub
```

## Firewall Rules

### Overview

Network firewall enforces IP-based access control with allow/deny rules. Evaluated in order - first match wins.

### Configuration

Configure firewall rules via environment variables:

```bash
# Allow connections from private network
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.0.0/16"},
  {"rule":"Allow","network":"127.0.0.1/8"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'

node-release 8333
```

### Rule Evaluation

Rules are checked in order. First matching rule determines access:

```
Rules:
  1. Allow 192.168.0.0/16  ← Matches → ALLOW
  2. Deny 0.0.0.0/0        ← Skipped

Results in:
  ✓ 192.168.1.100 allowed
  ✓ 192.168.1.200 allowed
  ✗ 10.0.0.1 denied (no matching allow rule)
```

### Practical Examples

**Allow only trusted peers:**
```bash
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Allow","network":"192.168.1.235/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'
```

**Allow local network only:**
```bash
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"127.0.0.1/8"},
  {"rule":"Allow","network":"192.168.0.0/16"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'
```

**Allow all (development):**
```bash
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"0.0.0.0/0"}
]'
```

## VPN Support

### Overview

Siertrichain supports running over VPN connections for enhanced privacy and security.

### Configuring VPN

Detect and bind to VPN interface:

```bash
# Specify VPN interface (e.g., tun0 for OpenVPN)
export SIERTRI_VPN_INTERFACE=tun0

node-release 8333
```

### SOCKS5 Proxy

Route outbound peer connections through SOCKS5 proxy:

```bash
# Connect to peers via SOCKS5
export SIERTRI_SOCKS5_PROXY=127.0.0.1:9050

node-release 8333
```

### Example: Tor Network

Run node over Tor for anonymity:

```bash
# 1. Start Tor (if not running)
# On Linux: tor --SocksPort 9050
# On Mac: brew services start tor

# 2. Start node over Tor
export SIERTRI_SOCKS5_PROXY=127.0.0.1:9050
node-release 8333
```

### Example: OpenVPN

```bash
# 1. Connect to OpenVPN
sudo openvpn --config config.ovpn --daemon

# 2. Start node on VPN interface
export SIERTRI_VPN_INTERFACE=tun0
node-release 8333
```

### Verifying VPN Connection

Check that node is using VPN:

```bash
# Check current IP (should be VPN IP)
curl ifconfig.me

# Compare with:
curl -x socks5://127.0.0.1:9050 http://ifconfig.me
```

## Rate Limiting

### Overview

Rate limiting prevents abuse and DoS attacks by throttling requests per peer/client.

### Default Limits

- **Peer requests**: 100 requests/second per peer
- **API requests**: 50 requests/second per IP
- **Transactions**: 10 transactions/second per wallet

### Configuration

```bash
# Set custom limits (tokens per second)
export SIERTRI_PEER_RATE_LIMIT=200
export SIERTRI_API_RATE_LIMIT=100
export SIERTRI_TX_RATE_LIMIT=20

node-release 8333
```

### Token Bucket Algorithm

Rate limiting uses token bucket algorithm:

1. Each entity (peer/IP/wallet) has a bucket with N tokens
2. Token bucket refills at rate limit per second
3. Each request consumes 1 token
4. Request denied if no tokens available

**Example:**
```
Rate: 10 requests/second
Initial: 10 tokens

Request 1: 10 → 9 tokens (allowed)
Request 2: 9 → 8 tokens (allowed)
...
Request 10: 1 → 0 tokens (allowed)
Request 11: 0 tokens (denied, must wait)

After 1 second: 10 tokens (refilled)
```

### Per-Entity Tracking

Rate limits tracked separately per entity:

```
Peer 192.168.1.230:8333  → 45/100 requests used
Peer 192.168.1.235:8334  → 92/100 requests used
API Client 192.168.1.1   → 30/50 requests used
Wallet abc123...         → 8/10 transactions used
```

### Monitoring Rate Limit Status

```bash
# API shows rate limit headers (if implemented)
curl -v http://localhost:3000/blockchain/height
# Returns: X-RateLimit-Remaining: 49
```

## Security Best Practices

### 1. Wallet Security

✅ **DO:**
- Use strong passwords (16+ characters, mixed case, numbers, symbols)
- Store backups in secure offline location
- Rotate passwords regularly
- Keep wallet files with restrictive permissions (0o600)

❌ **DON'T:**
- Share wallet files or passwords
- Store wallets on shared/public machines
- Use wallet on untrusted networks without VPN
- Commit wallets to version control

### 2. Network Security

✅ **DO:**
- Enable peer authentication (default)
- Use firewall rules to restrict access
- Authenticate all peer connections
- Monitor authenticated peer list
- Use rate limiting in production
- Enable VPN for remote connections

❌ **DON'T:**
- Disable peer authentication
- Allow all IPs (use firewall rules)
- Expose P2P port to public internet without authentication
- Run without rate limiting
- Mix authenticated and unauthenticated peers

### 3. Deployment Security

✅ **DO:**
- Run nodes in containers with resource limits
- Monitor node logs for authentication failures
- Set up alerts for unusual activity
- Use dedicated hardware/VPS for nodes
- Keep software updated
- Test security regularly

❌ **DON'T:**
- Run nodes as root
- Expose all ports publicly
- Disable security features for convenience
- Ignore authentication errors
- Mix sensitive data in logs

### 4. API Security

✅ **DO:**
- Bind API to localhost only (default: 127.0.0.1:3000)
- Use rate limiting (default enabled)
- Validate all inputs
- Log API access
- Use HTTPS for remote APIs
- Implement API authentication (JWT recommended)

❌ **DON'T:**
- Expose API to public without authentication
- Send private keys via API
- Trust unauthenticated API calls
- Disable rate limiting
- Log sensitive data

## Configuration Examples

### Example 1: Secure Private Network

Multi-node setup on private LAN with authentication and firewall:

```bash
# Node A (Bootstrap)
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.0/24"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'
node-release 8333

# Node B (Peer)
export SIERTRI_REQUIRE_AUTH=true
node-release 8334 --peer 192.168.1.230:8333

# Node C (Peer)
export SIERTRI_REQUIRE_AUTH=true
node-release 8335 --peer 192.168.1.230:8333
```

### Example 2: Tor Anonymity

Run node anonymously over Tor network:

```bash
# Start Tor
tor --SocksPort 9050

# Run node over Tor
export SIERTRI_SOCKS5_PROXY=127.0.0.1:9050
export SIERTRI_REQUIRE_AUTH=true
node-release 8333
```

### Example 3: VPN + Firewall + Rate Limiting

Maximum security configuration:

```bash
# Connect to VPN
sudo openvpn --config prod.ovpn --daemon

# Run node with all security features
export SIERTRI_VPN_INTERFACE=tun0
export SIERTRI_REQUIRE_AUTH=true
export SIERTRI_FIREWALL_RULES='[
  {"rule":"Allow","network":"192.168.1.230/32"},
  {"rule":"Allow","network":"192.168.1.235/32"},
  {"rule":"Deny","network":"0.0.0.0/0"}
]'
export SIERTRI_PEER_RATE_LIMIT=50
export SIERTRI_API_RATE_LIMIT=25

node-release 8333
```

## Security Audit Checklist

Before production deployment:

- [ ] All wallets encrypted with strong passwords
- [ ] Peer authentication enabled
- [ ] Firewall rules configured
- [ ] Rate limiting enabled
- [ ] VPN configured (if remote nodes)
- [ ] API bound to localhost
- [ ] Logs monitored for failures
- [ ] Backups tested and stored securely
- [ ] Software updated to latest version
- [ ] Security settings documented
- [ ] Incident response plan created
- [ ] Regular security reviews scheduled

## Troubleshooting

### Peer Won't Connect

```bash
# Check if peer is authenticated
# Look for: "New peer connection from X.X.X.X:XXXX"
# Check authentication failures in logs

# Verify IP is allowed by firewall
# Check SIERTRI_FIREWALL_RULES environment variable

# Check rate limit isn't exceeded
# Monitor rate limiter state
```

### Wallet Decryption Fails

```bash
# Wrong password error
# Make sure you're entering the correct password

# File corruption
# Restore from backup:
wallet-restore

# Check file permissions
ls -la ~/.TrinityChain/wallet.json
chmod 600 ~/.TrinityChain/wallet.json
```

### API Rate Limit Exceeded

```bash
# Getting "rate limit exceeded" errors?
# Wait for token refill (1 second)
# Or increase rate limit:
export SIERTRI_API_RATE_LIMIT=100
```

## Reporting Security Issues

Found a security vulnerability? Please report it responsibly:

1. **Don't** create a public GitHub issue
2. **Do** email security@TrinityChain.dev with:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We'll respond within 48 hours and coordinate a fix.

## References

- [AES-256-GCM Encryption](https://en.wikipedia.org/wiki/Galois/Counter_Mode)
- [Argon2 Password Hashing](https://argon2.online/)
- [ECDSA Signatures](https://en.wikipedia.org/wiki/Elliptic_Curve_Digital_Signature_Algorithm)
- [Rate Limiting Patterns](https://en.wikipedia.org/wiki/Token_bucket)
- [Firewall Rule Evaluation](https://en.wikipedia.org/wiki/Firewall_%28computing%29)

---

**Last Updated**: November 13, 2025
**Version**: 0.2.0 (Security Release)
