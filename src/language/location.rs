use std::str::FromStr;

use super::normalize::RoswaalNormalize;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RoswaalLocationNameParsingError {
    Empty
}

pub type RoswaalLocationParsingResult = Result<
    RoswaalLocationName,
    RoswaalLocationNameParsingError
>;

/// A valid location name representing a place.
///
/// This type contains helpers for matching the name against a query, and
/// for formatting the name in different contexts.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RoswaalLocationName {
    raw_value: String
}

impl RoswaalLocationName {
    pub fn name(&self) -> &str {
        &self.raw_value
    }
}

impl FromStr for RoswaalLocationName {
    type Err = RoswaalLocationNameParsingError;

    fn from_str(s: &str) -> RoswaalLocationParsingResult {
        if s.is_empty() { return Err(RoswaalLocationNameParsingError::Empty) }
        Ok(Self { raw_value: s.to_string() })
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
    pub fn matches(&self, other: &Self) -> bool {
        self.name().roswaal_normalize() == other.name().roswaal_normalize()
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
        let name = RoswaalLocationName::from_str("hello world").unwrap();
        assert_eq!(name.name(), "hello world")
    }

    #[test]
    fn test_matches_returns_true_when_exact_same_name() {
        let name = RoswaalLocationName::from_str("hello world").unwrap();
        assert!(name.matches(&"hello world".parse().unwrap()))
    }

    #[test]
    fn test_matches_returns_true_when_same_name_but_uppercased() {
        let name = RoswaalLocationName::from_str("hello world").unwrap();
        assert!(name.matches(&"Hello World".parse().unwrap()))
    }

    #[test]
    fn test_matches_returns_true_when_same_name_but_different_white_spacing_and_uppercased() {
        let name = RoswaalLocationName::from_str("hello world").unwrap();
        assert!(name.matches(&"\t  Hello   World\t  ".parse().unwrap()))
    }
}
