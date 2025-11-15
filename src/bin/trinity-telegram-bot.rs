use teloxide::{prelude::*, utils::command::BotCommands};
use log::info;
use trinitychain::persistence::Database;

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
}

async fn answer(bot: Bot, message: Message, command: Command) -> ResponseResult<()> {
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
                            let timestamp = chrono::NaiveDateTime::from_timestamp_opt(block.header.timestamp, 0)
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
                            let timestamp = chrono::NaiveDateTime::from_timestamp_opt(header.timestamp, 0)
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
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting TrinityChain Telegram Bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, Update::filter_message().filter_command::<Command>().endpoint(answer))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("TrinityChain Telegram Bot stopped.");
}
