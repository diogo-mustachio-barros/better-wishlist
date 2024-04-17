use std::env;

use regex::Regex;



const DISCORD_TOKEN_KEY:&str = "DISCORD_TOKEN";
const MONGODB_URL_KEY  :&str = "MONGODB_URL";



pub fn parse_secrets() -> Option<(String, String)> {
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

pub fn parse_card_from_drop(line: &str) -> Option<(&str, &str)> {
    
    let re = Regex::new(r".+?•.+?•\s\*\*([^•]+?)\*\*\s•(.*)").unwrap();

    match re.captures(&line) {
        Some(matches) => {
            let (_, [card, series]) = matches.extract(); 
            Some((series.trim(), card.trim()))
        },
        None => None
    }
}

pub fn parse_series_cards(line: &str) -> Option<(&str, Vec<&str>)> {
    let re = Regex::new(r"\s+(.+)\s*\|\|\s*(.+)").unwrap();

    re.captures(&line)
      .map(|capt| {
        let (_, [series, cards]) = capt.extract();
        let card_names:Vec<&str> = cards.split(",")
            .map(str::trim)
            .collect();

        (series.trim(), card_names)
      })
}