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

// #[cfg(test)]
// mod test_fn {
//     #[test]
//     fn test() {
//     }
// }