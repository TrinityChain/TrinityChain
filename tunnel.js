#!/usr/bin/env node

// Simple localtunnel wrapper that bypasses the openurl dependency issue
const http = require('http');
const https = require('https');
const { URL } = require('url');

const PORT = 3000;
const LT_SERVER = 'https://localtunnel.me';

async function getTunnel() {
  return new Promise((resolve, reject) => {
    const req = https.request(LT_SERVER + '/api/tunnels?new', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          resolve(parsed);
        } catch (e) {
          reject(e);
        }
      });
    });

    req.on('error', reject);
    req.write(JSON.stringify({
      responseType: 'json'
    }));
    req.end();
  });
}

async function assignTunnel(tunnelId) {
  return new Promise((resolve, reject) => {
    const postData = JSON.stringify({ port: PORT });

    const req = https.request(`${LT_SERVER}/api/tunnels/${tunnelId}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(postData),
      },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          resolve(parsed);
        } catch (e) {
          reject(e);
        }
      });
    });

    req.on('error', reject);
    req.write(postData);
    req.end();
  });
}

async function main() {
  console.log('Starting tunnel for localhost:' + PORT + '...');

  try {
    // First, request a new tunnel
    const tunnel = await getTunnel();
    console.log('Tunnel created:', tunnel);

    if (!tunnel.id) {
      console.error('Failed to get tunnel ID');
      process.exit(1);
    }

    // Assign the tunnel to our port
    const assigned = await assignTunnel(tunnel.id);
    console.log('\n🎉 Tunnel ready!');
    console.log('Public URL:', `https://${tunnel.id}.loca.lt`);
    console.log('Local Port:', PORT);
    console.log('\nTunnel will stay active. Press Ctrl+C to stop.\n');

    // Save the URL to a file for easy access
    require('fs').writeFileSync('tunnel-url.txt', `https://${tunnel.id}.loca.lt`);

    // Keep process alive
    process.stdin.resume();
  } catch (error) {
    console.error('Error creating tunnel:', error.message);
    process.exit(1);
  }
}

main();
