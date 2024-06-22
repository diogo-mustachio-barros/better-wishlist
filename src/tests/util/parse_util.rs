#[cfg(test)]
mod parse_series_card_from_analysis {
    use crate::util::parse_util::parse_series_card_from_analysis;

    #[test]
    fn empty_string() {
        assert_eq!(parse_series_card_from_analysis(""), None);
    }

    #[test]
    fn unrelated_string() {
        assert_eq!(parse_series_card_from_analysis("Nothing"), None);
    }

    #[test]
    fn less_separators() {
        assert_eq!(parse_series_card_from_analysis("A • A • A •"), None);
    } 

    #[test]
    fn too_many_separators() {
        assert_eq!(parse_series_card_from_analysis("A • A • A • A • A • A"), None);
    }

    #[test]
    fn no_card_name() {
        assert_eq!(parse_series_card_from_analysis("A • A • A •• A"), None);
    }

    #[test]
    fn no_series_name() {
        assert_eq!(parse_series_card_from_analysis("A • A • A • A •"), None);
    }

    #[test]
    fn too_many_separators_with_drop() {
        assert_eq!(parse_series_card_from_analysis("A • A • A • A • A • **card** • series"), None);
    }

    #[test]
    fn simple_drop() {
        assert_eq!(
            parse_series_card_from_analysis("A • A • A • A • **card** • series"), 
            Some(("series", "card")));
    }

    #[test]
    fn big_drop() {
        assert_eq!(
            parse_series_card_from_analysis("A • A • A • A • **a big card** • a large series"), 
            Some(("a large series", "a big card")));
    }

    #[test]
    fn drop_with_padding() {
        assert_eq!(
            parse_series_card_from_analysis("A • A • A • A • **  spaced card  ** •   spaced series   "), 
            Some(("spaced series", "spaced card")));
    }
}

#[cfg(test)]
mod parse_series_from_analysis {
    use crate::util::parse_util::parse_series_from_analysis;

    #[test]
    fn empty_string() {
        assert_eq!(parse_series_from_analysis(""), None);
    }

    #[test]
    fn unrelated_string() {
        assert_eq!(parse_series_from_analysis("Nothing"), None);
    }

    #[test]
    fn less_separators() {
        assert_eq!(parse_series_from_analysis("A • A"), None);
    }

    #[test]
    fn too_many_separators() {
        assert_eq!(parse_series_from_analysis("A • A • A • A"), None);
    }

    #[test]
    fn no_series_name() {
        assert_eq!(parse_series_from_analysis("A • A •"), None);
    }

    #[test]
    fn one_word_series() {
        assert_eq!(parse_series_from_analysis("A • A • series"), Some("series"));
    }

    #[test]
    fn multiple_word_series() {
        assert_eq!(parse_series_from_analysis("A • A • a big series"), Some("a big series"));
    }

    #[test]
    fn series_with_padding() {
        assert_eq!(parse_series_from_analysis("A • A •    spaced series   "), Some("spaced series"));
    }
}

#[cfg(test)]
mod is_series_analysis {
    use crate::util::parse_util::is_series_analysis;

    #[test]
    fn empty_string() {
        assert_eq!(is_series_analysis(""), false);
    }

    #[test]
    fn unrelated_string() {
        assert_eq!(is_series_analysis("Nothing"), false);
    }

    #[test]
    fn less_separators() {
        assert_eq!(is_series_analysis("A • A"), false);
    }

    #[test]
    fn too_many_separators() {
        assert_eq!(is_series_analysis("A • A • A • A"), false);
    }

    #[test]
    fn no_series_name() {
        assert_eq!(is_series_analysis("A • A •"), false);
    }

    #[test]
    fn one_word_series() {
        assert!(is_series_analysis("A • A • series"));
    }

    #[test]
    fn multiple_word_series() {
        assert!(is_series_analysis("A • A • a big series"));
    }

    #[test]
    fn series_with_padding() {
        assert!(is_series_analysis("A • A •    spaced series   "));
    }
}

#[cfg(test)]
mod parse_card_from_series_lookup {
    use crate::util::parse_util::parse_card_from_series_lookup;

    #[test]
    fn empty_string() {
        assert_eq!(parse_card_from_series_lookup(""), None);
    }

    #[test]
    fn unrelated_string() {
        assert_eq!(parse_card_from_series_lookup("Nothing"), None);
    }

    #[test]
    fn too_few_separators() {
        assert_eq!(parse_card_from_series_lookup("A • A • ☑️ • A • **card**"), None);
    }

    #[test]
    fn too_many_separators() {
        assert_eq!(parse_card_from_series_lookup("A • A • ☑️ • A • A • **card** • A"), None);
    }

    #[test]
    fn lookup_has_card() {
        assert_eq!(parse_card_from_series_lookup("A • A • ☑️ • A • A • **card**"), Some((true, "card")));
    }

    #[test]
    fn lookup_not_has_card() {
        assert_eq!(parse_card_from_series_lookup("A • A • ` ` • A • A • **card**"), Some((false, "card")));
    }

    #[test]
    fn lookup_not_has_card_big() {
        assert_eq!(parse_card_from_series_lookup("A • A • ` ` • A • A • **a big card**"), Some((false, "a big card")));
    }

    #[test]
    fn lookup_not_has_card_spaced() {
        assert_eq!(parse_card_from_series_lookup("A • A • ` ` • A • A • **   a spaced card   **"), Some((false, "a spaced card")));
    }
}

#[cfg(test)]
mod parse_series_from_embed_description {
    use crate::util::parse_util::parse_series_from_embed_description;

    #[test]
    fn empty_string() {
        assert_eq!(parse_series_from_embed_description(""), None);
    }

    #[test]
    fn unrelated_string() {
        assert_eq!(parse_series_from_embed_description("Nothing"), None);
    }

    #[test]
    fn no_series() {
        assert_eq!(parse_series_from_embed_description("Name: ****"), None);
    }

    #[test]
    fn simple_series() {
        assert_eq!(parse_series_from_embed_description("Name: **series**"), Some("series"));
    }

    #[test]
    fn big_series() {
        assert_eq!(parse_series_from_embed_description("Name: **a big series**"), Some("a big series"));
    }

    #[test]
    fn spaced_series() {
        assert_eq!(parse_series_from_embed_description("Name: **   spaced series   **"), Some("spaced series"));
    }
}

#[cfg(test)]
mod parse_series_cards {
    use crate::util::parse_util::parse_series_cards;

    #[test]
    fn empty_string() {
        assert_eq!(parse_series_cards(""), None);
    }

    #[test]
    fn unrelated_string() {
        assert_eq!(parse_series_cards("Nothing"), None);
    }

    #[test]
    fn only_pipe() {
        assert_eq!(parse_series_cards("||"), None);
    }

    #[test]
    fn only_series() {
        assert_eq!(parse_series_cards("series || "), None);
    }

    #[test]
    fn one_card() {
        assert_eq!(parse_series_cards("series || card_1"), Some(("series", vec!["card_1"])));
    }

    #[test]
    fn one_card_no_spaces() {
        assert_eq!(parse_series_cards("series||card_1"), Some(("series", vec!["card_1"])));
    }

    #[test]
    fn one_big_card() {
        assert_eq!(parse_series_cards("series || a big card"), Some(("series", vec!["a big card"])));
    }

    #[test]
    fn one_spaced_card() {
        assert_eq!(parse_series_cards("series ||    spaced card   "), Some(("series", vec!["spaced card"])));
    }

    #[test]
    fn multiple_cards() {
        assert_eq!(parse_series_cards("series || card_1, card_2, card_3"), Some(("series", vec!["card_1", "card_2", "card_3"])));
    }

    #[test]
    fn multiple_cards_no_spaces() {
        assert_eq!(parse_series_cards("series||card_1,card_2,card_3"), Some(("series", vec!["card_1", "card_2", "card_3"])));
    }
}

#[cfg(test)]
mod parse_series_from_give_command {
    use crate::util::parse_util::parse_series_from_give_command;

    #[test]
    fn empty_string() {
        assert_eq!(parse_series_from_give_command(""), None);
    }

    #[test]
    fn unrelated_string() {
        assert_eq!(parse_series_from_give_command("Nothing"), None);
    }

    #[test]
    fn simple_give_one_line() {
        assert_eq!(parse_series_from_give_command("Name: **card** Series: **series**"), None);
    }

    #[test]
    fn simple_give() {
        assert_eq!(parse_series_from_give_command("Name: **card**\nSeries: **series**"), Some(("series", "card")));
    }
}