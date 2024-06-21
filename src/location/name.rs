use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

use crate::utils::{normalize::RoswaalNormalize, string::UppercaseFirstAsciiCharacter};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RoswaalLocationNameParsingError {
    Empty,
    InvalidFormat
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
    pub(super) raw_value: String
}

impl RoswaalLocationName {
    pub fn raw_name(&self) -> &str {
        &self.raw_value
    }
}

static LOCATION_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^(?:[a-zA-Z0-9_\s]*[a-zA-Z]+[a-zA-Z0-9_\s]*)$")
        .build()
        .expect("Failed to compile location name regex.")
});

impl FromStr for RoswaalLocationName {
    type Err = RoswaalLocationNameParsingError;

    fn from_str(s: &str) -> RoswaalLocationParsingResult {
        if s.is_empty() {
            Err(RoswaalLocationNameParsingError::Empty)
        } else if LOCATION_NAME_REGEX.is_match(s) {
            Ok(Self { raw_value: s.to_string() })
        } else {
            Err(RoswaalLocationNameParsingError::InvalidFormat)
        }
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
        self.raw_name().roswaal_normalize() == other.raw_name().roswaal_normalize()
    }
}

impl RoswaalLocationName {
    /// Returns this name in `PascalCase` as a raw String.
    pub fn to_ascii_pascal_case_string(&self) -> String {
        self.raw_name().to_ascii_pascal_case()
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
    fn test_from_str_returns_error_when_invalid_format() {
        let strings = [
            ")(U(*U(*)",
            "San, Francisco",
            "|}{|DOJON SIBI",
            "Santa | Cruz",
            "Na(mek)",
            "1234567890",
            "     198397939"
        ];
        for str in strings {
            let name = RoswaalLocationName::from_str(str);
            assert_eq!(name, Err(RoswaalLocationNameParsingError::InvalidFormat))
        }
    }

    #[test]
    fn test_from_str_returns_success_when_valid() {
        let strings = [
            "hello world",
            "San Francisco",
            "    1 beach street"
        ];
        for str in strings {
            let name = RoswaalLocationName::from_str(str).unwrap();
            assert_eq!(name.raw_name(), str);
        }
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
