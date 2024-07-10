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

pub trait ToAsciiKebabCase: ToString {
    fn to_ascii_kebab_case(&self) -> String {
        self.to_string().trim().replace(" ", "-")
    }
}

impl ToAsciiKebabCase for String {}
impl ToAsciiKebabCase for &str {}

#[cfg(test)]
mod string_utils_tests {
    use super::*;

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

    #[test]
    fn to_ascii_kebab_case() {
        let strings = vec![
            ("hello world", "hello-world"),
            ("Hello world", "Hello-world"),
            ("", ""),
            ("test", "test"),
            ("it's really cold outside", "it's-really-cold-outside"),
            ("hello + world", "hello-+-world"),
            ("hello@world", "hello@world"),
            ("   hello world   ", "hello-world"),
            ("hello    world", "hello----world")
        ];
        for (before, after) in strings {
            assert_eq!(before.to_ascii_kebab_case(), after.to_string())
        }
    }
}
