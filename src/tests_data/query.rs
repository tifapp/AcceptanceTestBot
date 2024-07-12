/// A type for representing a user entered query for a list of test names.
///
/// Users will enter test names with each test name being on a separate line. An empty string
/// indicates that *all* tests should be covered by this query.
pub struct RoswaalTestsQuery<'a> {
    string: &'a str
}

impl <'a> RoswaalTestsQuery<'a> {
    pub fn new(string: &'a str) -> Self {
        Self { string }
    }
}

impl <'a> RoswaalTestsQuery<'a> {
    /// Returns true if this query is for querying all tests.
    pub fn is_for_all_tests(&self) -> bool {
        self.string.is_empty()
    }

    /// Returns an iterator to the test names of this query.
    pub fn names(&self) -> impl Iterator<Item = &'a str> {
        self.string.lines().filter_map(|line| {
            let string = line.trim();
            if string.is_empty() {
                None
            } else {
                Some(string)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_denotes_all_tests() {
        let query = RoswaalTestsQuery::new("");
        assert!(query.is_for_all_tests())
    }

    #[test]
    fn string_with_names_denotes_specific_tests() {
        let string = "\
Test 1
   Johnny in Mexico
Basic Sail Across the Atlantic with the Titanic

The 5 Hounds are too OP Plz Nerf
            ";
        let query = RoswaalTestsQuery::new(string);
        assert!(!query.is_for_all_tests())
    }

    #[test]
    fn separates_test_names_on_each_line_ignoring_empty_lines() {
        let string = "\
Test 1
   Johnny in Mexico
Basic Sail Across the Atlantic with the Titanic

The 5 Hounds are too OP Plz Nerf
            ";
        let query = RoswaalTestsQuery::new(string);
        let test_names = query.names().collect::<Vec<&str>>();
        let expected_names = vec![
            "Test 1",
            "Johnny in Mexico",
            "Basic Sail Across the Atlantic with the Titanic",
            "The 5 Hounds are too OP Plz Nerf"
        ];
        assert_eq!(test_names, expected_names)
    }
}
