use teloxide::{prelude::*, utils::command::BotCommands};
use log::{info, warn};
use trinitychain::persistence::Database;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

type RateLimiter = Arc<Mutex<HashMap<i64, std::time::Instant>>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "start the bot and see welcome message")]
    Start,
    #[command(description = "show all available commands")]
    Help,
    #[command(description = "view blockchain statistics")]
    Stats,
    #[command(description = "check wallet balance (address)")]
    Balance(String),
    #[command(description = "view recent blocks")]
    Blocks,
    #[command(description = "see genesis block")]
    Genesis,
    #[command(description = "count total triangles in UTXO")]
    Triangles,
    #[command(description = "current mining difficulty")]
    Difficulty,
    #[command(description = "current blockchain height")]
    Height,
    #[command(description = "learn about TrinityChain")]
    About,
    #[command(description = "open the blockchain explorer dashboard")]
    Dashboard,
    #[command(description = "show mempool statistics")]
    Mempool,
    #[command(description = "show connected peer count")]
    Node,
    #[command(description = "broadcast raw tx hex to peers")]
    Broadcast(String),
}

async fn answer(
    bot: Bot,
    message: Message,
    command: Command,
    node_opt: Option<Arc<trinitychain::network::NetworkNode>>,
    admin_token: Option<String>,
    rate_limiter: RateLimiter,
) -> ResponseResult<()> {
    match command {
        Command::Start => {
            let welcome_msg = "🔺 Welcome to TrinityChain Bot! 🔺\n\n\
                TrinityChain is a unique blockchain based on triangle geometry.\n\n\
                Use /help to see all available commands.\n\
                Use /stats to view blockchain statistics.\n\
                Use /about to learn more about TrinityChain.";
            bot.send_message(message.chat.id, welcome_msg).await?;
            info!("Handled /start command for user: {:?}", message.from());
        }
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
            info!("Handled /help command for user: {:?}", message.from());
        }
        Command::Stats => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => {
                        let height = chain.blocks.last().map_or(0, |b| b.header.height);
                        let total_supply = trinitychain::blockchain::Blockchain::calculate_current_supply(height);
                        let current_reward = trinitychain::blockchain::Blockchain::calculate_block_reward(height);
                        let triangles = chain.state.count();

                        format!(
                            "📊 Blockchain Statistics:\n\n\
                            🏔️ Height: {}\n\
                            💰 Total Supply: {} area\n\
                            🎁 Current Block Reward: {} area\n\
                            🔺 Active Triangles: {}\n\
                            ⚡ Mining Difficulty: {}",
                            height, total_supply, current_reward, triangles, chain.difficulty
                        )
                    }
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /stats command for user: {:?}", message.from());
        }
        Command::Mempool => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => {
                        let pool_size = chain.mempool.len();
                        let txs = chain.mempool.get_all_transactions();
                        let top_fees: Vec<_> = txs.iter().take(5).map(|tx| tx.fee()).collect();
                        format!("📥 Mempool: {} transactions\nTop fees (sample): {:?}", pool_size, top_fees)
                    }
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /mempool command for user: {:?}", message.from());
        }
        Command::Node => {
            if let Some(node) = node_opt.as_ref() {
                let peers_count = node.peers_count().await;
                let response = format!("🌐 Connected peers: {}", peers_count);
                bot.send_message(message.chat.id, response).await?;
            } else {
                bot.send_message(message.chat.id, "🌐 Network node not initialized on this bot.").await?;
            }
            info!("Handled /node command for user: {:?}", message.from());
        }
        Command::Broadcast(txhex) => {
            // Rate limiting: allow one broadcast per 60s per user
            if let Some(from) = message.from() {
                let uid = from.id.0 as i64;
                let mut rl = rate_limiter.lock().await;
                if let Some(last) = rl.get(&uid) {
                    if last.elapsed() < std::time::Duration::from_secs(60) {
                        bot.send_message(message.chat.id, "⏳ Rate limit: please wait before broadcasting again.").await?;
                        return Ok(());
                    }
                }
                // update last seen now
                rl.insert(uid, std::time::Instant::now());
            }

            // Support sending admin token as prefix: TOKEN:HEX
            let mut provided_token: Option<&str> = None;
            let mut hex_part = txhex.as_str();
            if let Some(pos) = txhex.find(':') {
                provided_token = Some(&txhex[..pos]);
                hex_part = &txhex[pos+1..];
            }

            // If admin token is configured, require it matches provided token
            if let Some(cfg_token) = admin_token.as_ref() {
                match provided_token {
                    Some(t) if t == cfg_token => {
                        // ok
                    }
                    _ => {
                        warn!("Unauthorized broadcast attempt from {:?}", message.from());
                        bot.send_message(message.chat.id, "❌ Unauthorized: missing or invalid admin token for /broadcast").await?;
                        return Ok(());
                    }
                }
            }

            match hex::decode(hex_part) {
                Ok(bytes) => match Database::open("trinitychain.db") {
                    Ok(_db) => match _db.load_blockchain() {
                        Ok(chain) => {
                            match bincode::deserialize::<trinitychain::transaction::Transaction>(&bytes) {
                                Ok(tx) => {
                                    if let Some(node) = node_opt.as_ref() {
                                        match node.broadcast_transaction(&tx).await {
                                            Ok(_) => {
                                                let msg = format!("✅ Broadcasted transaction {} to peers", tx.hash_str());
                                                bot.send_message(message.chat.id, msg).await?;
                                            }
                                            Err(e) => {
                                                let msg = format!("❌ Failed to broadcast: {}", e);
                                                bot.send_message(message.chat.id, msg).await?;
                                            }
                                        }
                                    } else {
                                        bot.send_message(message.chat.id, "❌ Network node not initialized; cannot broadcast.").await?;
                                    }
                                }
                                Err(_) => {
                                    bot.send_message(message.chat.id, "Invalid transaction bytes; could not deserialize.").await?;
                                }
                            }
                        }
                        Err(_) => { bot.send_message(message.chat.id, "Could not load blockchain data.").await?; }
                    },
                    Err(_) => { bot.send_message(message.chat.id, "Could not open blockchain database.").await?; }
                },
                Err(_) => { bot.send_message(message.chat.id, "Invalid hex provided.").await?; }
            }

            info!("Handled /broadcast command for user: {:?}", message.from());
        }
        Command::Balance(address) => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => {
                        let triangles_owned: Vec<_> = chain.state.utxo_set.iter()
                            .filter(|(_, triangle)| triangle.owner == address)
                            .collect();

                        let balance: f64 = triangles_owned.iter()
                            .map(|(_, triangle)| triangle.area())
                            .sum();

                        format!(
                            "💰 Balance for {}:\n\n\
                            Total Area: {:.6} area\n\
                            Number of Triangles: {}",
                            if address.len() > 20 {
                                format!("{}...{}", &address[..8], &address[address.len()-8..])
                            } else {
                                address.clone()
                            },
                            balance,
                            triangles_owned.len()
                        )
                    }
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /balance command for user: {:?}", message.from());
        }
        Command::Blocks => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => {
                        let num_blocks = chain.blocks.len().min(5);
                        let recent_blocks = &chain.blocks[chain.blocks.len().saturating_sub(num_blocks)..];

                        let mut msg = format!("📦 Recent {} Blocks:\n\n", num_blocks);
                        for block in recent_blocks.iter().rev() {
                            let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(block.header.timestamp, 0)
                                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "Invalid".to_string());
                            let hash_hex = hex::encode(block.hash);
                            let hash_display = format!("{}...{}", &hash_hex[..8], &hash_hex[hash_hex.len()-8..]);

                            msg.push_str(&format!(
                                "🔺 Block #{}\n  Hash: {}\n  Time: {}\n  Txs: {}\n\n",
                                block.header.height,
                                hash_display,
                                timestamp,
                                block.transactions.len()
                            ));
                        }
                        msg
                    }
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /blocks command for user: {:?}", message.from());
        }
        Command::Genesis => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => {
                        if let Some(genesis_block) = chain.blocks.get(0) {
                            let header = &genesis_block.header;
                            let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(header.timestamp, 0)
                                .map(|t| t.to_string())
                                .unwrap_or_else(|| "Invalid timestamp".to_string());
                            let headline = header.headline.as_deref().unwrap_or("N/A");
                            format!(
                                "Genesis Block:\n- Timestamp: {}\n- Headline: {}\n- Hash: {}",
                                timestamp,
                                headline,
                                hex::encode(genesis_block.hash)
                            )
                        } else {
                            "Genesis block not found.".to_string()
                        }
                    }
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /genesis command for user: {:?}", message.from());
        }
        Command::Triangles => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => format!("Total triangles in UTXO set: {}", chain.state.count()),
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /triangles command for user: {:?}", message.from());
        }
        Command::Difficulty => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => format!("Current mining difficulty: {}", chain.difficulty),
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /difficulty command for user: {:?}", message.from());
        }
        Command::Height => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => {
                        let height = chain.blocks.last().map_or(0, |b| b.header.height);
                        format!("Current blockchain height: {}", height)
                    }
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /height command for user: {:?}", message.from());
        }
        Command::About => {
            let about_msg = "🔺 About TrinityChain 🔺\n\n\
                TrinityChain is an innovative blockchain built on triangle geometry.\n\n\
                Key Features:\n\
                • Geometric-based ownership using triangles\n\
                • Proof-of-Work consensus mechanism\n\
                • Triangle subdivision for value transfer\n\
                • Bitcoin-style supply curve (21M total)\n\
                • Halving events every 210,000 blocks\n\n\
                Each unit of value is represented as a geometric triangle with an area.\n\
                The blockchain maintains a UTXO (Unspent Triangle Output) set.\n\n\
                Mining is currently active and rewards decrease over time through halvings.";
            bot.send_message(message.chat.id, about_msg).await?;
            info!("Handled /about command for user: {:?}", message.from());
        }
        Command::Dashboard => {
            let dashboard_msg = "🔺 TrinityChain Dashboard 🔺\n\n\
                The blockchain explorer dashboard is available as a Telegram Mini App!\n\n\
                To set it up:\n\
                1. Host the dashboard files from the /dashboard folder on HTTPS\n\
                2. Configure the Web App URL with @BotFather\n\
                3. Access it via the bot menu button\n\n\
                The dashboard shows:\n\
                • Real-time blockchain statistics\n\
                • Recent blocks\n\
                • Genesis block info\n\
                • And more!\n\n\
                For setup instructions, see dashboard/README.md in the repository.";
            bot.send_message(message.chat.id, dashboard_msg).await?;
            info!("Handled /dashboard command for user: {:?}", message.from());
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting TrinityChain Telegram Bot...");

    // Initialize bot and dependencies
    let bot = Bot::from_env();

    // Load admin token from env (optional)
    let admin_token = std::env::var("BOT_ADMIN_TOKEN").ok();

    // Initialize rate limiter (in-memory)
    let rate_limiter: RateLimiter = Arc::new(Mutex::new(HashMap::new()));

    // Try to initialize a persistent NetworkNode if DB is available
    let node_opt: Option<Arc<trinitychain::network::NetworkNode>> = match Database::open("trinitychain.db") {
        Ok(db) => match db.load_blockchain() {
            Ok(chain) => {
                let node = trinitychain::network::NetworkNode::new(chain, "trinitychain.db".to_string());
                Some(Arc::new(node))
            }
            Err(e) => {
                warn!("Could not load blockchain for NetworkNode: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Could not open database for NetworkNode: {}", e);
            None
        }
    };

    Dispatcher::builder(bot, Update::filter_message().filter_command::<Command>().endpoint(answer))
        .dependencies(dptree::deps![node_opt, admin_token, rate_limiter])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("TrinityChain Telegram Bot stopped.");
}
