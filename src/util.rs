use regex::Regex;

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