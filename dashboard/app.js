// Telegram Web App initialization
let tg = window.Telegram?.WebApp;

// API base URL - can be configured via Telegram Bot
const API_BASE = tg?.initDataUnsafe?.start_param || 'http://localhost:3000/api';

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
        const response = await fetch(`${API_BASE}/blockchain/height`);
        const data = await response.json();

        document.getElementById('blockHeight').textContent = data.height || '0';

        // Fetch more stats
        const balanceResponse = await fetch(`${API_BASE}/balance/all`);
        if (balanceResponse.ok) {
            const balanceData = await balanceResponse.json();
            document.getElementById('totalTriangles').textContent = balanceData.total_triangles || '0';
            document.getElementById('totalArea').textContent =
                balanceData.total_area ? balanceData.total_area.toFixed(2) : '0.00';
        }

        // Fetch difficulty
        const difficultyResponse = await fetch(`${API_BASE}/blockchain/difficulty`);
        if (difficultyResponse.ok) {
            const difficultyData = await difficultyResponse.json();
            document.getElementById('difficulty').textContent = difficultyData.difficulty || '2';
        }

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
        const response = await fetch(`${API_BASE}/blockchain/recent`);
        const data = await response.json();

        const blocksContainer = document.getElementById('recentBlocks');

        if (!data.blocks || data.blocks.length === 0) {
            blocksContainer.innerHTML = '<p class="loading">No blocks yet. Start mining!</p>';
            return;
        }

        blocksContainer.innerHTML = data.blocks.slice(0, 10).map(block => `
            <div class="block-item">
                <div class="block-header">
                    <span class="block-height">Block #${block.height}</span>
                    <span class="block-time">${formatTime(block.timestamp)}</span>
                </div>
                <div class="block-hash">
                    Hash: ${formatHash(block.hash)}
                </div>
                <div style="margin-top: 10px; color: #888; font-size: 0.9rem;">
                    ${block.transactions} transaction(s) • Difficulty: ${block.difficulty}
                </div>
            </div>
        `).join('');
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
