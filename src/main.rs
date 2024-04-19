mod wishlist_db;
mod bot;
mod util;
mod logger;
mod interaction_manager;

use std::sync::Arc;

use bot::init_discord_bot;
use wishlist_db::init_db;

use crate::{logger::{Logger, StdoutLogger}, util::parse_util::parse_secrets};


#[tokio::main]
async fn main() {
    let logger = Arc::new(StdoutLogger);
    
    // Read secrets
    let Some((discord_token, mongodb_url)) = parse_secrets()
    else {
        logger.log_error("Missing credentials");
        return;
    };

    
    logger.log_info("Connecting to database");
    // Init db connection
    let Ok(db_connection) = init_db(logger.clone(), mongodb_url).await 
    else {
        logger.log_error("Unable to connect to database");
        return;
    };
    logger.log_info("Connected to database");
    
    logger.log_info("Initializing Discord bot");
    // Init discord bot api 
    let mut discord_client = init_discord_bot(discord_token, db_connection, logger.clone()).await;
    logger.log_info("Discord bot initialized");


    // Start listening for events by starting a single shard
    if let Err(why) = discord_client.start().await {
        logger.log_error(format!("Client error: {why:?}"));
    }
}