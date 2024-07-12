/// A type for representing a user entered query for a list of test names.
///
/// Users will enter test names with each test name being on a separate line. An empty string
/// indicates that *all* tests should be covered by this query.
#[derive(Debug, PartialEq, Eq)]
pub enum RoswaalSearchTestsQuery<'a> {
    TestNames(RoswaalTestNamesString<'a>),
    AllTests
}

impl <'a> RoswaalSearchTestsQuery<'a> {
    pub fn new(string: &'a str) -> Self {
        if string.is_empty() {
            Self::AllTests
        } else {
            Self::TestNames(RoswaalTestNamesString(string))
        }
    }
}

/// A list of stringified test names.
#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalTestNamesString<'a>(pub &'a str);

impl <'a> RoswaalTestNamesString<'a> {
    /// Returns an iterator to the test names specified by this string.
    pub fn iter(&self) -> impl Iterator<Item = &'a str> {
        self.0.lines().filter_map(|line| {
            let string = line.trim();
            if string.is_empty() {
                None
            } else {
                Some(string)
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.iter().count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_denotes_all_tests() {
        let query = RoswaalSearchTestsQuery::new("");
        assert_eq!(query, RoswaalSearchTestsQuery::AllTests)
    }

    #[test]
    fn separates_test_names_on_each_line_ignoring_empty_lines() {
        let string = "\
Test 1
   Johnny in Mexico
Basic Sail Across the Atlantic with the Titanic

The 5 Hounds are too OP Plz Nerf
            ";
        let query = RoswaalSearchTestsQuery::new(string);
        let test_names = match query {
            RoswaalSearchTestsQuery::TestNames(test_names) => test_names,
            _ => panic!()
        };
        let test_names = test_names.iter().collect::<Vec<&str>>();
        let expected_names = vec![
            "Test 1",
            "Johnny in Mexico",
            "Basic Sail Across the Atlantic with the Titanic",
            "The 5 Hounds are too OP Plz Nerf"
        ];
        assert_eq!(test_names, expected_names)
    }

    #[test]
    fn is_empty() {
        let strings = vec![("", true), ("    ", true), ("\n\n  \n", true), ("h", false)];
        for (string, is_empty) in strings {
            assert_eq!(RoswaalTestNamesString(string).is_empty(), is_empty)
        }
    }
}
