use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalLocationNameParsingError {
    Empty
}

/// A valid location name representing a place.
///
/// This type contains helpers for matching the name against a query, and
/// for formatting the name in different contexts.
#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalLocationName {
    pub raw_value: String
}

impl FromStr for RoswaalLocationName {
    type Err = RoswaalLocationNameParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() { return Err(RoswaalLocationNameParsingError::Empty) }
        Ok(Self { raw_value: String::from(s) })
    }
}

impl RoswaalLocationName {
    /// Returns true if the specified string slice is the same as this
    /// name case and whitespace insensitive.
    ///
    /// Ex.
    /// ```rs
    /// let name = RoswaalLocationName::from_str("hello world").unwrap();
    /// assert!(name.matches("  Hello  World  "))
    /// ```
    pub fn matches(&self, str: &str) -> bool {
        self.normalize(&self.raw_value) == self.normalize(str)
    }

    fn normalize(&self, str: &str) -> String {
        str.to_lowercase().split_whitespace().collect::<String>()
    }
}

#[cfg(test)]
mod roswaal_location_tests {
    use super::*;

    #[test]
    fn test_from_str_returns_error_when_empty() {
        let name = RoswaalLocationName::from_str("");
        assert_eq!(name, Err(RoswaalLocationNameParsingError::Empty))
    }

    #[test]
    fn test_from_str_returns_success_when_valid() {
        let name = RoswaalLocationName::from_str("hello world");
        assert!(name.is_ok())
    }

    #[test]
    fn test_matches_returns_false_when_empty_string() {
        let name = RoswaalLocationName::from_str("hello world")
            .expect("Should parse successfully.");
        assert!(!name.matches(""))
    }

    #[test]
    fn test_matches_returns_true_when_exact_same_name() {
        let name = RoswaalLocationName::from_str("hello world")
            .expect("Should parse successfully.");
        assert!(name.matches("hello world"))
    }

    #[test]
    fn test_matches_returns_true_when_same_name_but_uppercased() {
        let name = RoswaalLocationName::from_str("hello world")
            .expect("Should parse successfully.");
        assert!(name.matches("Hello World"))
    }

    #[test]
    fn test_matches_returns_true_when_same_name_but_different_white_spacing_and_uppercased() {
        let name = RoswaalLocationName::from_str("hello world")
            .expect("Should parse successfully.");
        assert!(name.matches("\t  Hello   World\t  "))
    }
}
