mod wishlist_db;
mod bot;
mod util;
mod logger;

use bot::init_discord_bot;
use wishlist_db::init_db;

use crate::{logger::StdoutLogger, util::parse_util::parse_secrets};


#[tokio::main]
async fn main() {
    // Read secrets
    let opt_secrets = parse_secrets();
    if opt_secrets.is_none() {
        println!("Missing credentials.");
        return;
    }

    let (discord_token, mongodb_url) = opt_secrets.unwrap();

    let logger = StdoutLogger;
    
    print!("Connecting to DB...");
    // Init db connection
    let db_connection = init_db(mongodb_url).await.expect("Unable to connect to database");
    println!("done.");

    print!("Initializing Discord bot...");
    // Init discord bot api 
    let mut discord_client = init_discord_bot(discord_token, db_connection, logger).await;
    println!("done."); 

    // Start listening for events by starting a single shard
    if let Err(why) = discord_client.start().await {
        println!("Client error: {why:?}");
    }
}