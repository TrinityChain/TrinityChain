use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::{Mutex, RwLock};
use trinitychain::network::NetworkNode;
use trinitychain::persistence::Database;

type RateLimiter = Arc<Mutex<HashMap<i64, std::time::Instant>>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "start the bot and see welcome message")]
    Start,
    #[command(description = "show all available commands")]
    Help,
    #[command(description = "view blockchain statistics")]
    Stats,
    #[command(description = "check wallet balance (address)")]
    Balance,
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
    #[command(description = "show network peer list")]
    Peers,
    #[command(description = "show node status and stats")]
    Status,
    #[command(description = "broadcast raw tx hex to peers")]
    Broadcast(String),
}

async fn answer(
    bot: Bot,
    message: Message,
    command: Command,
    node_opt: Option<Arc<NetworkNode>>,
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
            bot.send_message(message.chat.id, Command::descriptions().to_string())
                .await?;
            info!("Handled /help command for user: {:?}", message.from());
        }
        Command::Stats => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => {
                        let height = chain.blocks.last().map_or(0, |b| b.header.height);
                        let total_supply: f64 = chain.blocks.iter().flat_map(|b| &b.transactions).filter_map(|tx| {
                            if let trinitychain::transaction::Transaction::Coinbase(ctx) = tx {
                                Some(ctx.reward_area.to_num::<f64>())
                            } else {
                                None
                            }
                        }).sum();
                        let current_reward =
                            trinitychain::blockchain::Blockchain::calculate_block_reward(height);
                        let triangles = chain.state.utxo_set.len();

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
                        format!(
                            "📥 Mempool: {} transactions\nTop fees (sample): {:?}",
                            pool_size, top_fees
                        )
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
                let peers_count = node.list_peers().await.len();
                let response = format!("🌐 Connected peers: {}", peers_count);
                bot.send_message(message.chat.id, response).await?;
            } else {
                bot.send_message(
                    message.chat.id,
                    "🌐 Network node not initialized on this bot.",
                )
                .await?;
            }
            info!("Handled /node command for user: {:?}", message.from());
        }
        Command::Peers => {
            if let Some(node) = node_opt.as_ref() {
                let peers = node.list_peers().await;
                if peers.is_empty() {
                    bot.send_message(message.chat.id, "📋 No connected peers yet.")
                        .await?;
                } else {
                    let peer_list = peers
                        .iter()
                        .map(|p| format!("• {}", p.addr()))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let response = format!("📋 Connected Peers ({})\n\n{}", peers.len(), peer_list);
                    bot.send_message(message.chat.id, response).await?;
                }
            } else {
                bot.send_message(
                    message.chat.id,
                    "📋 Network node not initialized on this bot.",
                )
                .await?;
            }
            info!("Handled /peers command for user: {:?}", message.from());
        }
        Command::Status => {
            let response = match Database::open("trinitychain.db") {
                Ok(db) => match db.load_blockchain() {
                    Ok(chain) => {
                        let height = chain.blocks.last().map_or(0, |b| b.header.height);
                        let peers_count = if let Some(node) = node_opt.as_ref() {
                            node.list_peers().await.len()
                        } else {
                            0
                        };
                        let mempool_size = chain.mempool.len();
                        let utxo_count = chain.state.utxo_set.len();

                        format!(
                            "📊 Node Status:\n\n\
                            🏔️ Height: {}\n\
                            🌐 Peers: {}\n\
                            📥 Mempool: {} txs\n\
                            🔺 UTXO Count: {}\n\
                            ⚡ Difficulty: {}",
                            height, peers_count, mempool_size, utxo_count, chain.difficulty
                        )
                    }
                    Err(_) => "Could not load blockchain data.".to_string(),
                },
                Err(_) => "Could not open blockchain database.".to_string(),
            };
            bot.send_message(message.chat.id, response).await?;
            info!("Handled /status command for user: {:?}", message.from());
        }
        Command::Broadcast(txhex) => {
            if let Some(from) = message.from() {
                let uid = from.id.0 as i64;
                let mut rl = rate_limiter.lock().await;
                if let Some(last) = rl.get(&uid) {
                    if last.elapsed() < std::time::Duration::from_secs(60) {
                        bot.send_message(
                            message.chat.id,
                            "⏳ Rate limit: please wait before broadcasting again.",
                        )
                        .await?;
                        return Ok(());
                    }
                }
                rl.insert(uid, std::time::Instant::now());
            }

            let mut provided_token: Option<&str> = None;
            let mut hex_part = txhex.as_str();
            if let Some(pos) = txhex.find(':') {
                provided_token = Some(&txhex[..pos]);
                hex_part = &txhex[pos + 1..];
            }

            if let Some(cfg_token) = admin_token.as_ref() {
                match provided_token {
                    Some(t) if t == cfg_token => {}
                    _ => {
                        warn!("Unauthorized broadcast attempt from {:?}", message.from());
                        bot.send_message(
                            message.chat.id,
                            "❌ Unauthorized: missing or invalid admin token for /broadcast",
                        )
                        .await?;
                        return Ok(());
                    }
                }
            }

            match hex::decode(hex_part) {
                Ok(bytes) => {
                    match bincode::deserialize::<trinitychain::transaction::Transaction>(&bytes) {
                        Ok(tx) => {
                            if let Some(node) = node_opt.as_ref() {
                                node.broadcast_transaction(&tx).await;
                                let msg = format!(
                                    "✅ Broadcasted transaction {} to peers",
                                    tx.hash_str()
                                );
                                bot.send_message(message.chat.id, msg).await?;
                            } else {
                                bot.send_message(
                                    message.chat.id,
                                    "❌ Network node not initialized; cannot broadcast.",
                                )
                                .await?;
                            }
                        }
                        Err(_) => {
                            bot.send_message(
                                message.chat.id,
                                "Invalid transaction bytes; could not deserialize.",
                            )
                            .await?;
                        }
                    }
                }
                Err(_) => {
                    bot.send_message(message.chat.id, "Invalid hex provided.")
                        .await?;
                }
            }

            info!("Handled /broadcast command for user: {:?}", message.from());
        }
        _ => {
            bot.send_message(message.chat.id, "Command not implemented yet.")
                .await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting TrinityChain Telegram Bot...");

    let bot = Bot::from_env();
    let admin_token = std::env::var("BOT_ADMIN_TOKEN").ok();
    let rate_limiter: RateLimiter = Arc::new(Mutex::new(HashMap::new()));

    let node_opt: Option<Arc<NetworkNode>> = match Database::open("trinitychain.db") {
        Ok(db) => match db.load_blockchain() {
            Ok(chain) => {
                let node = NetworkNode::new(Arc::new(RwLock::new(chain)));
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

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(answer),
    )
    .dependencies(dptree::deps![node_opt, admin_token, rate_limiter])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    info!("TrinityChain Telegram Bot stopped.");
}
