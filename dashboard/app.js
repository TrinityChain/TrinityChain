// Telegram Web App initialization
let tg = window.Telegram?.WebApp;

// API base URL - can be configured via Telegram Bot `start_param`, query param `?api=` or defaults to relative
function resolveApiBase() {
    // 1) start_param (bot can set start_param to the API base)
    if (tg && tg.initDataUnsafe && tg.initDataUnsafe.start_param) {
        return tg.initDataUnsafe.start_param;
    }

    // 2) query param ?api=
    try {
        const urlParams = new URLSearchParams(window.location.search);
        const apiParam = urlParams.get('api');
        if (apiParam) return apiParam;
    } catch (e) {
        // ignore
    }

    // 3) fallback to /api (relative path for same-server deployments)
    return '/api';
}

const API_BASE = resolveApiBase();

// Debug logging
console.log('🔺 TrinityChain Dashboard Debug Info:');
console.log('API_BASE:', API_BASE);
console.log('URL:', window.location.href);
console.log('Telegram Web App:', tg ? 'Available' : 'Not available');
if (tg && tg.initDataUnsafe) {
    console.log('User:', tg.initDataUnsafe.user?.username || tg.initDataUnsafe.user?.first_name || 'Unknown');
}

// Initialize Telegram Web App
if (tg) {
    tg.ready();
    tg.expand();

    // Apply Telegram theme colors
    document.documentElement.style.setProperty('--tg-theme-bg-color', tg.themeParams.bg_color || '#0a0e27');
    document.documentElement.style.setProperty('--tg-theme-text-color', tg.themeParams.text_color || '#e0e0e0');
    document.documentElement.style.setProperty('--tg-theme-hint-color', tg.themeParams.hint_color || '#888');
    document.documentElement.style.setProperty('--tg-theme-link-color', tg.themeParams.link_color || '#00ff88');
    document.documentElement.style.setProperty('--tg-theme-button-color', tg.themeParams.button_color || '#00ff88');
    document.documentElement.style.setProperty('--tg-theme-button-text-color', tg.themeParams.button_text_color || '#ffffff');
    document.documentElement.style.setProperty('--tg-theme-secondary-bg-color', tg.themeParams.secondary_bg_color || '#1a1f3a');
}

// Format timestamp
function formatTime(timestamp) {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
}

// Format hash (show first 8 and last 8 characters)
function formatHash(hash) {
    if (hash.length > 16) {
        return `${hash.substring(0, 8)}...${hash.substring(hash.length - 8)}`;
    }
    return hash;
}

// Fetch blockchain stats
async function fetchStats() {
    const statsUrl = `${API_BASE}/blockchain/stats`;
    console.log(`[fetchStats] Fetching from: ${statsUrl}`);
    try {
        const response = await fetch(statsUrl);
        console.log(`[fetchStats] Response status: ${response.status}`);
        if (!response.ok) {
            console.error(`[fetchStats] HTTP error: ${response.status} ${response.statusText}`);
            throw new Error(`HTTP ${response.status}`);
        }
        const data = await response.json();
        console.log('[fetchStats] Data received:', data);

        document.getElementById('blockHeight').textContent = data.height || '0';
        document.getElementById('totalTriangles').textContent = data.utxo_count || '0';
        document.getElementById('difficulty').textContent = data.difficulty || '2';

        // Calculate total area from UTXO set (approximate)
        document.getElementById('totalArea').textContent = (data.utxo_count * 100).toFixed(2);

        // Provide haptic feedback on successful data load
        if (tg) {
            tg.HapticFeedback.impactOccurred('light');
        }
    } catch (error) {
        console.error('[fetchStats] Error:', error);
        document.getElementById('blockHeight').textContent = 'Offline';
        document.getElementById('totalTriangles').textContent = 'Offline';
        document.getElementById('totalArea').textContent = 'Offline';
        document.getElementById('difficulty').textContent = 'Offline';

        // Notify user of error in Telegram
        if (tg) {
            tg.showAlert('Unable to connect to blockchain API. Please check your connection.');
        }
    }
}

// Fetch recent blocks
async function fetchRecentBlocks() {
    const statsUrl = `${API_BASE}/blockchain/stats`;
    console.log(`[fetchRecentBlocks] Fetching from: ${statsUrl}`);
    try {
        const statsResponse = await fetch(statsUrl);
        console.log(`[fetchRecentBlocks] Response status: ${statsResponse.status}`);
        if (!statsResponse.ok) {
            throw new Error(`HTTP ${statsResponse.status}`);
        }
        const statsData = await statsResponse.json();
        console.log('[fetchRecentBlocks] Stats data:', statsData);

        const blocksContainer = document.getElementById('recentBlocks');

        if (!statsData.recent_blocks || statsData.recent_blocks.length === 0) {
            blocksContainer.innerHTML = '<p class="loading">No blocks yet. Start mining!</p>';
            return;
        }

        // Fetch full block details for each recent block
        const blockPromises = statsData.recent_blocks.slice(0, 5).map(async (blockInfo) => {
            try {
                const blockUrl = `${API_BASE}/blockchain/block/by-height/${blockInfo.height}`;
                console.log(`[fetchRecentBlocks] Fetching block from: ${blockUrl}`);
                const blockResponse = await fetch(blockUrl);
                return await blockResponse.json();
            } catch (e) {
                console.error('[fetchRecentBlocks] Error fetching block:', e);
                return null;
            }
    // --- Mining Control Logic ---

    const minerAddressInput = document.getElementById('minerAddressInput');
    const loadWalletButton = document.getElementById('loadWalletButton');
    const startMiningButton = document.getElementById('startMiningButton');
    const stopMiningButton = document.getElementById('stopMiningButton');
    const minerAddressDisplay = document.getElementById('minerAddress');
    const miningStatusDisplay = document.getElementById('miningStatus');
    const blocksMinedDisplay = document.getElementById('blocksMined');
    const hashRateDisplay = document.getElementById('hashRate');

    // Inject CLI wallet address from Python kernel to make it available in JS
    let cliMinerAddress = '7339ba1f28a194fe5d099a9d7551e1aa78a633e85f3c846b4e046d7cbe43f434';

    function updateMiningStatusDisplay(status) {
        miningStatusDisplay.innerText = status.is_mining ? 'Active' : 'Inactive';
        blocksMinedDisplay.innerText = status.blocks_mined;
        hashRateDisplay.innerText = `${status.hashrate.toFixed(2)} H/s`;
        miningStatusDisplay.style.color = status.is_mining ? '#00ff88' : '#ff0044';
        
        // Update button states based on mining status
        startMiningButton.disabled = status.is_mining;
        stopMiningButton.disabled = !status.is_mining;
        loadWalletButton.disabled = status.is_mining;
        minerAddressInput.disabled = status.is_mining;
    }

    async function fetchMiningStatus() {
        try {
            const response = await fetch(`${apiBase}/mining/status`);
            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`HTTP error! status: ${response.status}, message: ${errorText}`);
            }
            const status = await response.json();
            updateMiningStatusDisplay(status);
        } catch (error) {
            console.error('Error fetching mining status:', error);
            miningStatusDisplay.innerText = 'Error';
            miningStatusDisplay.style.color = '#ff0044';
            startMiningButton.disabled = false;
            stopMiningButton.disabled = true;
            loadWalletButton.disabled = false;
            minerAddressInput.disabled = false;
        }
    }

    async function startMining() {
        const address = minerAddressInput.value.trim();
        if (!address) {
            alert('Please enter a miner address.');
            return;
        }
        try {
            startMiningButton.disabled = true;
            stopMiningButton.disabled = true; // Disable until status is confirmed
            loadWalletButton.disabled = true;
            minerAddressInput.disabled = true;

            const response = await fetch(`${apiBase}/mining/start`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ miner_address: address })
            });
            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`HTTP error! status: ${response.status}, message: ${errorText}`);
            }
            const result = await response.text(); // Assuming response is text, not JSON, based on previous API calls
            console.log('Mining started:', result);
            // Refresh status immediately
            fetchMiningStatus();
        } catch (error) {
            console.error('Error starting mining:', error);
            alert(`Failed to start mining: ${error.message}`);
            fetchMiningStatus(); // Fetch status to reflect actual state and re-enable buttons correctly
        }
    }

    async function stopMining() {
        try {
            stopMiningButton.disabled = true;
            startMiningButton.disabled = true; // Disable until status is confirmed
            loadWalletButton.disabled = true;
            minerAddressInput.disabled = true;

            const response = await fetch(`${apiBase}/mining/stop`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' }
            });
            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`HTTP error! status: ${response.status}, message: ${errorText}`);
            }
            const result = await response.text(); // Assuming response is text, not JSON
            console.log('Mining stopped:', result);
            // Refresh status immediately
            fetchMiningStatus();
        } catch (error) {
            console.error('Error stopping mining:', error);
            alert(`Failed to stop mining: ${error.message}`);
            fetchMiningStatus(); // Fetch status to reflect actual state and re-enable buttons correctly
        }
    }

    // Function to handle loading CLI wallet address
    function loadCliWalletAddress() {
        if (cliMinerAddress && cliMinerAddress !== '') {
            minerAddressInput.value = cliMinerAddress;
            minerAddressDisplay.innerText = cliMinerAddress;
            alert('CLI wallet address loaded successfully!');
        } else {
            alert('CLI wallet address not available or empty. Please ensure it was created via CLI and try again, or enter it manually.');
        }
    }

    // Set up mining controls if elements exist
    if (minerAddressInput && loadWalletButton && startMiningButton && stopMiningButton) {
        loadWalletButton.addEventListener('click', loadCliWalletAddress);
        startMiningButton.addEventListener('click', startMining);
        stopMiningButton.addEventListener('click', stopMining);

        // Pre-fill miner address if available (e.g., from Python injection)
        if (cliMinerAddress && cliMinerAddress !== '') {
            minerAddressInput.value = cliMinerAddress;
            minerAddressDisplay.innerText = cliMinerAddress;
        } else {
            minerAddressInput.value = 'Paste miner address here'; // Placeholder instruction
            minerAddressDisplay.innerText = 'Not Set';
        }

        // Fetch initial mining status and then poll
        fetchMiningStatus();
        setInterval(fetchMiningStatus, 5000); // Poll every 5 seconds
    }
    // --- End Mining Control Logic --- 

        });

        const blocks = (await Promise.all(blockPromises)).filter(b => b !== null);
        console.log('[fetchRecentBlocks] Blocks loaded:', blocks.length);

        blocksContainer.innerHTML = blocks.map((block, index) => {
            const blockInfo = statsData.recent_blocks[index];
            return `
                <div class="block-item">
                    <div class="block-header">
                        <span class="block-height">Block #${block.header.height}</span>
                        <span class="block-time">${formatTime(block.header.timestamp)}</span>
                    </div>
                    <div class="block-hash">
                        Hash: ${formatHash(blockInfo.hash)}
                    </div>
                    <div style="margin-top: 10px; color: #888; font-size: 0.9rem;">
                        ${block.transactions.length} transaction(s) • Difficulty: ${block.header.difficulty}
                    </div>
                </div>
            `;
        }).join('');
    } catch (error) {
        console.error('[fetchRecentBlocks] Error:', error);
        document.getElementById('recentBlocks').innerHTML =
            '<p class="loading">Unable to fetch blocks. Is the API server running?</p>';
    }
}

// Update data periodically
function startAutoUpdate() {
    fetchStats();
    fetchRecentBlocks();

    // Update every 10 seconds
    setInterval(() => {
        fetchStats();
        fetchRecentBlocks();
    }, 10000);
}

// Start when page loads
document.addEventListener('DOMContentLoaded', () => {
    // Update debug panel
    document.getElementById('debugApiBase').textContent = API_BASE;
    document.getElementById('debugStatus').textContent = 'Fetching data...';
    
    startAutoUpdate();

    // Log Telegram user info if available
    if (tg && tg.initDataUnsafe.user) {
        console.log('Telegram User:', tg.initDataUnsafe.user.username || tg.initDataUnsafe.user.first_name);
    }
});

// Toggle debug panel visibility
function toggleDebugPanel() {
    const panel = document.getElementById('debugPanel');
    panel.style.display = panel.style.display === 'none' ? 'block' : 'none';
}
