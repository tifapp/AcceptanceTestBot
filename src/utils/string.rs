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
        if str.is_empty() { return self.to_string() }
        str[0..1].make_ascii_uppercase();
        return str
    }
}

impl UppercaseFirstAsciiCharacter for String {}
impl UppercaseFirstAsciiCharacter for &str {}

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
}
