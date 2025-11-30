# TrinityChain API Endpoints

Base URL: `http://localhost:3000` (for local dev)

## Blockchain Endpoints

### GET `/api/blockchain/height`
Get current blockchain height.

**Response:**
```json
123
```

### GET `/api/blockchain/blocks`
Get recent blocks from the blockchain. Supports pagination.

**Query Parameters:**
- `page` (optional, default: 0) - Page number to retrieve.
- `limit` (optional, default: 10) - Number of blocks per page.

**Response:**
```json
{
  "blocks": [
    {
      "header": {
        "height": 123,
        "timestamp": 1672531200000,
        "previous_hash": "0000...",
        "merkle_root": "...",
        "difficulty": 4,
        "nonce": 12345
      },
      "transactions": []
    }
  ],
  "total": 124,
  "page": 0,
  "limit": 1
}
```

### GET `/api/blockchain/block/:height`
Get block by height.

**Response:**
A single block object (see `/api/blockchain/blocks`).

### GET `/api/blockchain/stats`
Get blockchain statistics.

**Response:**
```json
{
  "height": 123,
  "difficulty": 4,
  "mempool_size": 10,
  "total_blocks": 123
}
```

## Transaction Endpoints

### POST `/api/transaction`
Submit a new transaction.

**Request Body:**
A `Transaction` object.
```json
{
  "Transfer": {
    "input_hash": "...",
    "new_owner": "...",
    "sender": "...",
    "amount": "100.0",
    "fee_area": "1.0",
    "nonce": 0,
    "signature": "...",
    "public_key": "..."
  }
}
```

**Response:**
```json
{
  "message": "Transaction submitted successfully"
}
```

### GET `/api/transaction/:hash`
Get transaction status by hash.

**Response:**
A `Transaction` object.

### GET `/api/mempool`
Get pending transactions in mempool.

**Response:**
```json
{
  "count": 1,
  "transactions": [ ... ]
}
```

## Mining Endpoints

### POST `/api/mining/start`
Start mining.

**Request Body:**
```json
{
  "miner_address": "your-miner-address"
}
```

**Response:**
```json
{
  "message": "Mining started successfully"
}
```

### POST `/api/mining/stop`
Stop mining.

**Response:**
```json
{
  "message": "Mining stopped successfully"
}
```

### GET `/api/mining/status`
Get current mining status.

**Response:**
```json
{
  "is_mining": false,
  "blocks_mined": 0
}
```

## Network Endpoints

### GET `/api/network/peers`
Get connected peers.

**Response:**
```json
{
  "count": 1,
  "peers": [ ... ]
}
```

### GET `/api/network/info`
Get network information.

**Response:**
```json
{
  "peer_count": 1,
  "peers": [ ... ],
  "protocol_version": "1.0"
}
```

## Address & Balance Endpoints

### GET `/api/address/:addr/balance`
Get balance for an address.

**Response:**
```json
{
  "balance": "5000.0",
  "address": "your-address"
}
```

### GET `/api/address/:addr/transactions`
Get transaction history for an address.

**Response:**
```json
{
  "address": "your-address",
  "count": 1,
  "transactions": [ ... ]
}
```

## Wallet Endpoints

### POST `/api/wallet/create`
Create a new wallet.

**Response:**
```json
{
  "address": "...",
  "public_key": "..."
}
```

## System Endpoints

### GET `/health`
Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "..."
}
```

### GET `/stats`
Get API server statistics.

**Response:**
```json
{
  "total_requests": 100,
  "successful_requests": 98,
  "failed_requests": 2,
  "mining_starts": 1,
  "mining_stops": 0,
  "transactions_submitted": 5,
  "uptime_seconds": 3600,
  "blocks_mined": 10,
  "is_mining": true
}
```
