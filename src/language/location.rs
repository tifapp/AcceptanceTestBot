use std::str::FromStr;

use strum_macros::EnumString;

use crate::utils::string::UppercaseFirstAsciiCharacter;

#[derive(Debug, PartialEq, Eq, EnumString)]
pub enum RoswaalLocation {
    LosAngeles,
    SanFrancisco,
    Oakland,
    Brooklyn,
    Houston,
    Antarctica,
    London,
    Miami,
    Reno,
    SanJose,
    SantaCruz,
    Sacramento,
    Paris,
    SaltLakeCity
}

impl RoswaalLocation {
    fn parse(str: &String) -> Option<Self> {
        let comparator = str.split_whitespace()
            .map(|s| s.uppercase_first_ascii_char())
            .collect::<String>();
        RoswaalLocation::from_str(comparator.as_str()).ok()
    }
}

#[cfg(test)]
mod roswaal_location_tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let location = RoswaalLocation::parse(&String::from(""));
        assert!(location.is_none())
    }

    #[test]
    fn test_parse_random() {
        let location = RoswaalLocation::parse(&String::from("didhughduiguyd"));
        assert!(location.is_none())
    }

    #[test]
    fn test_name_for_all_locations() {
        assert_location("Los Angeles", RoswaalLocation::LosAngeles);
        assert_location("San Francisco", RoswaalLocation::SanFrancisco);
        assert_location("Oakland", RoswaalLocation::Oakland);
        assert_location("Antarctica", RoswaalLocation::Antarctica);
        assert_location("Paris", RoswaalLocation::Paris);
        assert_location("Santa Cruz", RoswaalLocation::SantaCruz);
        assert_location("Houston", RoswaalLocation::Houston);
        assert_location("London", RoswaalLocation::London);
        assert_location("Salt Lake City", RoswaalLocation::SaltLakeCity);
        assert_location("Sacramento", RoswaalLocation::Sacramento);
        assert_location("San Jose", RoswaalLocation::SanJose);
        assert_location("Reno", RoswaalLocation::Reno);
        assert_location("Brooklyn", RoswaalLocation::Brooklyn);
        assert_location("Miami", RoswaalLocation::Miami);
    }

    fn assert_location(name: &str, location: RoswaalLocation) {
        let name_str = String::from(name);
        let some_location = Some(location);
        assert_eq!(RoswaalLocation::parse(&name_str), some_location);
        assert_eq!(
            RoswaalLocation::parse(&name_str.to_lowercase()),
            some_location
        );
        let with_whitespace = format!("  \t{}  \t", name);
        assert_eq!(
            RoswaalLocation::parse(&with_whitespace),
            some_location
        );
        assert_eq!(
            RoswaalLocation::parse(&with_whitespace.to_lowercase()),
            some_location
        );
    }
}
