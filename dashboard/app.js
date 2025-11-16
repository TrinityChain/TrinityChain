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

    // 3) fallback to relative path for deployments behind same domain or GitHub Pages proxy
    return '/api';
}

const API_BASE = resolveApiBase();

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
    try {
        const response = await fetch(`${API_BASE}/blockchain/stats`);
        const data = await response.json();

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
        console.error('Error fetching stats:', error);
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
    try {
        const statsResponse = await fetch(`${API_BASE}/blockchain/stats`);
        const statsData = await statsResponse.json();

        const blocksContainer = document.getElementById('recentBlocks');

        if (!statsData.recent_blocks || statsData.recent_blocks.length === 0) {
            blocksContainer.innerHTML = '<p class="loading">No blocks yet. Start mining!</p>';
            return;
        }

        // Fetch full block details for each recent block
        const blockPromises = statsData.recent_blocks.slice(0, 5).map(async (blockInfo) => {
            try {
                const blockResponse = await fetch(`${API_BASE}/blockchain/block/by-height/${blockInfo.height}`);
                return await blockResponse.json();
            } catch (e) {
                console.error('Error fetching block:', e);
                return null;
            }
        });

        const blocks = (await Promise.all(blockPromises)).filter(b => b !== null);

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
        console.error('Error fetching blocks:', error);
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
    startAutoUpdate();

    // Log Telegram user info if available
    if (tg && tg.initDataUnsafe.user) {
        console.log('Telegram User:', tg.initDataUnsafe.user.username || tg.initDataUnsafe.user.first_name);
    }
});
