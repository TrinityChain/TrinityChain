# TrinityChain Dashboard - Telegram Mini App

This is the TrinityChain blockchain explorer converted to a Telegram Mini App.

## Features

- Real-time blockchain statistics
- Recent blocks viewer
- Genesis block information
- Responsive design optimized for mobile
- Telegram theme integration
- Haptic feedback support

## Setup

### 1. Host the Dashboard

You need to host these files on a publicly accessible HTTPS server. Options include:

- GitHub Pages
- Vercel
- Netlify
- Any web hosting service with HTTPS

### Deploy to GitHub Pages (recommended)

This repository includes a GitHub Actions workflow that will publish the `dashboard/` folder to the `gh-pages` branch on each push to `main`.

1. Push your changes to `main`.
2. The action will publish the `dashboard/` folder to the `gh-pages` branch.
3. Enable GitHub Pages in repository Settings → Pages, choosing the `gh-pages` branch and the root folder.

Your dashboard will then be available at:

```
https://<owner>.github.io/<repo>/
```

For this repo the expected URL (once published) will be:

```
https://TrinityChain.github.io/TrinityChain/
```

### 2. Configure Your Telegram Bot

1. Talk to [@BotFather](https://t.me/botfather) on Telegram
2. Send `/newapp` or `/myapps`
3. Select your TrinityChain bot
4. Choose "Edit Web App URL"
5. Enter the URL where your dashboard is hosted (e.g., `https://yourdomain.com/dashboard/`)

### 3. Update API Endpoint

Edit `app.js` and update the `API_BASE` constant to point to your TrinityChain API server:

```javascript
// Default resolution supports `?api=` query param, Telegram start_param, or relative `/api`.
// To override, set query `?api=https://your-api-server.com/api` or configure your bot start_param.
const API_BASE = '/api';
```

### 4. Add Menu Button to Bot

You can add a menu button to your bot so users can easily access the dashboard:

1. Talk to [@BotFather](https://t.me/botfather)
2. Send `/mybots`
3. Select your bot
4. Choose "Bot Settings" → "Menu Button"
5. Send the URL of your hosted dashboard

## Using with the Telegram Bot

Once configured, users can:
- Open the Mini App from the bot's menu button
- Use the `/dashboard` command in the bot
- Click on inline buttons that open the dashboard

## File Structure

- `index.html` - Main HTML file with Telegram Web App SDK
- `style.css` - Responsive styles with Telegram theme support
- `app.js` - JavaScript with Telegram Web App integration
- `README.md` - This file

## Telegram Theme Variables

The app automatically adapts to the user's Telegram theme using these variables:

- `--tg-theme-bg-color`
- `--tg-theme-text-color`
- `--tg-theme-hint-color`
- `--tg-theme-link-color`
- `--tg-theme-button-color`
- `--tg-theme-button-text-color`
- `--tg-theme-secondary-bg-color`

## Development

To test locally:

1. Serve the dashboard folder with any HTTP server
2. Open in a web browser (some Telegram features won't work outside Telegram)
3. For full Telegram integration testing, deploy to a public HTTPS server

## Notes

- The dashboard requires a running TrinityChain API server
- HTTPS is required for Telegram Mini Apps in production
- The app will work in regular browsers but Telegram-specific features will be disabled
