use std::ops::Range;

pub trait ToAsciiCamelCase {
    fn to_ascii_camel_case(&self) -> String;
}

impl ToAsciiCamelCase for String {
    fn to_ascii_camel_case(&self) -> String {
        self.split_whitespace()
            .map(|str| str.to_lowercase())
            .reduce(|mut acc, str| {
                acc.push_str(str.uppercase_first_ascii_char().as_str());
                return acc
            })
            .unwrap_or(self.to_string())
    }
}

pub trait UppercaseFirstAsciiCharacter: ToString {
    fn uppercase_first_ascii_char(&self) -> String {
        let mut str = self.to_string();
        if str.is_empty() { return str }
        str[0..1].make_ascii_uppercase();
        return str
    }

    fn to_ascii_pascal_case(&self) -> String {
        self.to_string()
            .split_whitespace()
            .map(|s| s.uppercase_first_ascii_char())
            .collect::<String>()
    }
}

impl UppercaseFirstAsciiCharacter for String {}
impl UppercaseFirstAsciiCharacter for &str {}

pub trait FirstRangeOfString: ToString {
    fn first_range_of_string(
        &self,
        string: &str
    ) -> Option<Range<usize>> {
        self.to_string()
            .match_indices(string)
            .next()
            .map(|(i, _)| i..(i + string.len()))
    }
}

impl FirstRangeOfString for &str {}
impl FirstRangeOfString for String {}

#[cfg(test)]
mod string_utils_tests {
    use super::*;

    #[test]
    fn test_first_range_of_none_when_characters_not_found() {
        let range = "hello".first_range_of_string("yay");
        assert_eq!(range, None)
    }

    #[test]
    fn test_first_range_of_returns_range_of_first_occurrence() {
        let range = "hel hel".first_range_of_string("hel");
        assert_eq!(range, Some(0..3))
    }

    #[test]
    fn test_make_first_uppercase_empty() {
        let str = String::from("");
        assert_eq!(str.uppercase_first_ascii_char(), str)
    }

    #[test]
    fn test_make_first_uppercase_basic() {
        let str = String::from("hello");
        let expected_str = String::from("Hello");
        assert_eq!(str.uppercase_first_ascii_char(), expected_str)
    }

    #[test]
    fn test_string_to_camel_case_empty() {
        let str = String::from("");
        assert_eq!(str.to_ascii_camel_case(), str)
    }

    #[test]
    fn test_string_to_camel_case_single_word() {
        let str = String::from("hello");
        assert_eq!(str.to_ascii_camel_case(), str)
    }

    #[test]
    fn test_string_to_camel_case_with_spaced_words() {
        let str = String::from("Hello world this is a test");
        assert_eq!(str.to_ascii_camel_case(), String::from("helloWorldThisIsATest"))
    }

    #[test]
    fn test_pascal_case_uppercases_first_letter_of_single_word_name() {
        assert_eq!("oakland".to_ascii_pascal_case(), String::from("Oakland"))
    }

    #[test]
    fn test_pascal_case_uppercases_first_letter_of_all_words_and_trims_white_space() {
        assert_eq!("santa cruz".to_ascii_pascal_case(), String::from("SantaCruz"))
    }
}
