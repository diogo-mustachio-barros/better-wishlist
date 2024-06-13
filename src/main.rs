mod bot;
mod util;
mod components;
mod commands;
mod integrations;
mod tests;

use std::sync::Arc;
use bot::start_bot;
use components::{logger::StdoutLogger, logger::Logger, wishlist_db::{self, init_db}};

use util::parse_util::parse_secrets;


#[tokio::main]
async fn main() {
    let logger = Arc::new(StdoutLogger);
    
    // Read secrets
    let Some((discord_token, mongodb_url)) = parse_secrets()
    else {
        logger.log_error("Missing credentials");
        return;
    };

    
    // Init db connection
    logger.log_info("Connecting to database");
    let Ok(db_connection) = init_db(logger.clone(), mongodb_url).await 
    else {
        logger.log_error("Unable to connect to database");
        return;
    };
    logger.log_info("Connected to database");
    
    // Init discord bot api 
    logger.log_info("Initializing Discord bot");
    let mut discord_client = start_bot(discord_token, db_connection, logger.clone()).await;
    logger.log_info("Discord bot initialized");


    // Start listening for events by starting a single shard
    if let Err(why) = discord_client.start().await {
        logger.log_error(format!("Client error: {why:?}"));
    }
}