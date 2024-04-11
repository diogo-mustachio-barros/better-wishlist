mod util;
mod wishlist_db;
mod bot;
mod bot_util;

use std::env;
use bot::init_discord_bot;
use wishlist_db::init_db;


/* 
[1] • ❤️ •    1 • Charloss • One Piece
[2] • ❤️ •    2 • Megumi Kitaniji • The World Ends with You
[3] • ❤️ •    2 • Shizuku • Omamori Himari
*/

const DISCORD_TOKEN_KEY:&str = "DISCORD_TOKEN";
const MONGODB_URL_KEY  :&str = "MONGODB_URL";


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


fn parse_secrets() -> Option<(String, String)> {
    // Main args have priority
    let mut args: Vec<String> = env::args().collect();
    if args.len() >= 3 {
        let mongodb_url = args.remove(2);
        let discord_token = args.remove(1);
        return Some((discord_token, mongodb_url))
    }

    // If no main args, try searching environment variables
    match (env::var(DISCORD_TOKEN_KEY), env::var(MONGODB_URL_KEY)) {
        (Ok(discord_token), Ok(mongodb_url)) => return Some((discord_token, mongodb_url)),
        _ => ()
    }

    // If everything fails, return none
    None
}