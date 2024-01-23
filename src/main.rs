// This bot throws a dice on each incoming message.

use dotenv::dotenv;
use std::env;
use teloxide::prelude::*;

mod command;

#[tokio::main]
async fn main() {

    dotenv().ok();
    
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot_token = env::var("BOT_TOKEN").expect("Bot token is not set");
    let bot = Bot::new(bot_token);

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(command::message_handler));

    Dispatcher::builder(bot, handler).enable_ctrlc_handler().build().dispatch().await;
    // Ok(())

}