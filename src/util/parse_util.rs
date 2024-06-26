use std::env;

use regex::Regex;

const DISCORD_TOKEN_KEY : &str = "DISCORD_TOKEN";
const MONGODB_URL_KEY   : &str = "MONGODB_URL";

const HAS_CARD_EMOJI : &str = "☑️";

const CARDS_ANALYSIS_REGEX : &str = r"^[^•]+•[^•]+•[^•]+•[^•]+•\s\*\*([^•]+?)\*\*\s•([^•]*).*";
const SERIES_ANALYSIS_REGEX : &str = r"^[^•]+?•[^•]+?•\s+([^ɢ`•\*]+)$";
const SERIES_LOOKUP_REGEX : &str = r"[^•]+?•[^•]+?•\s([^•]+?)\s•[^•]+?•[^•]+?•\s\*\*([^•]+?)\*\*$";

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

pub fn parse_series_card_from_analysis(line: &str) -> Option<(&str, &str)> {
    
    let re = Regex::new(CARDS_ANALYSIS_REGEX).unwrap();

    match re.captures(line) {
        Some(matches) => {
            let (_, [card, series]) = matches.extract(); 
            Some((series.trim(), card.trim()))
        },
        None => None
    }
}

pub fn parse_series_from_analysis(line: &str) -> Option<&str> {
    
    let re = Regex::new(SERIES_ANALYSIS_REGEX).unwrap();

    match re.captures(line) {
        Some(matches) => {
            let (_, [series]) = matches.extract(); 
            Some(series.trim())
        },
        None => None
    }
}

pub fn is_series_analysis(line: &str) -> bool {
    let re = Regex::new(SERIES_ANALYSIS_REGEX).unwrap();

    re.is_match(line)
}

pub fn parse_card_from_series_lookup(line: &str) -> Option<(bool, &str)> {
    
    let re = Regex::new(SERIES_LOOKUP_REGEX).unwrap();

    match re.captures(line) {
        Some(matches) => {
            let (_, [has_card_emoji, card]) = matches.extract(); 
            Some((has_card_emoji.trim() == HAS_CARD_EMOJI, card.trim()))
        },
        None => None
    }
}

pub fn parse_series_from_embed_description(description: &str) -> Option<&str> {
    
    let re = Regex::new(r"Name:\s\*\*(.+?)\*\*").unwrap();

    match re.captures(description) {
        Some(matches) => {
            let (_, [series]) = matches.extract(); 
            Some(series.trim())
        },
        None => None
    }
}

pub fn parse_series_cards(line: &str) -> Option<(&str, Vec<&str>)> {
    let re = Regex::new(r"\s*(.+)\s*\|\|\s*(.+)").unwrap();

    re.captures(line.trim())
      .map(|capt| {
        let (_, [series, cards]) = capt.extract();
        let card_names:Vec<&str> = cards.split(",")
            .map(str::trim)
            .collect();

        (series.trim(), card_names)
      })
}

pub fn parse_series_from_give_command(description: &str) -> Option<(&str, &str)> {
    let re = Regex::new(r"Name: \*\*(.+)\*\*\nSeries: \*\*(.+)\*\*.*").unwrap();

    match re.captures(description) {
        Some(matches) => {
            let (_, [card, series]) = matches.extract(); 
            Some((series.trim(), card.trim()))
        },
        None => None
    }
}