pub trait ToAsciiCamelCase {
    fn to_ascii_camel_case(&self) -> String;
}

impl ToAsciiCamelCase for String {
    fn to_ascii_camel_case(&self) -> String {
        self.split_whitespace()
            .map(|str| str.to_lowercase())
            .reduce(|mut acc, mut str| {
                str[0..1].make_ascii_uppercase();
                acc.push_str(str.as_str());
                return acc
            })
            .unwrap_or(self.to_string())
    }
}

#[cfg(test)]
mod string_utils_tests {
    use super::*;

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
