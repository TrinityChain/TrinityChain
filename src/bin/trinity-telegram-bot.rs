use teloxide::{prelude::*, utils::command::BotCommands};
use log::info;

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
            bot.send_message(message.chat.id, "Welcome to TrinityChain Telegram Bot!").await?;
            info!("Handled /start command for user: {:?}", message.from());
        }
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
            info!("Handled /help command for user: {:?}", message.from());
        }
        Command::Stats => {
            bot.send_message(message.chat.id, "Blockchain statistics: coming soon...").await?;
            info!("Handled /stats command for user: {:?}", message.from());
        }
        Command::Balance(address) => {
            bot.send_message(message.chat.id, format!("Balance for {}: coming soon...", address)).await?;
            info!("Handled /balance command for user: {:?}", message.from());
        }
        Command::Blocks => {
            bot.send_message(message.chat.id, "Recent blocks: coming soon...").await?;
            info!("Handled /blocks command for user: {:?}", message.from());
        }
        Command::Genesis => {
            bot.send_message(message.chat.id, "Genesis block: coming soon...").await?;
            info!("Handled /genesis command for user: {:?}", message.from());
        }
        Command::Triangles => {
            bot.send_message(message.chat.id, "Total triangles in UTXO: coming soon...").await?;
            info!("Handled /triangles command for user: {:?}", message.from());
        }
        Command::Difficulty => {
            bot.send_message(message.chat.id, "Current mining difficulty: coming soon...").await?;
            info!("Handled /difficulty command for user: {:?}", message.from());
        }
        Command::Height => {
            bot.send_message(message.chat.id, "Current blockchain height: coming soon...").await?;
            info!("Handled /height command for user: {:?}", message.from());
        }
        Command::About => {
            bot.send_message(message.chat.id, "TrinityChain is a blockchain project.").await?;
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
