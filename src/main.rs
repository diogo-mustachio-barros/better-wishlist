mod wishlist_db;
mod bot;
mod parse_util;

use bot::init_discord_bot;
use wishlist_db::init_db;

use crate::parse_util::parse_secrets;


#[tokio::main]
async fn main() {
    // Read secrets
    let opt_secrets = parse_secrets();
    if opt_secrets.is_none() {
        println!("Missing credentials.")
    }

    let (discord_token, mongodb_url) = opt_secrets.unwrap();

    print!("Connecting to DB...");
    // Init db connection
    let db_connection = init_db(mongodb_url).await.expect("Unable to connect to database");
    println!("done.");

    print!("Initializing Discord bot...");
    // Init discord bot api 
    let mut discord_client = init_discord_bot(discord_token, db_connection).await;
    println!("done."); 

    // Start listening for events by starting a single shard
    if let Err(why) = discord_client.start().await {
        println!("Client error: {why:?}");
    }
}