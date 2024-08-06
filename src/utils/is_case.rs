/// A macro which returns true if an enum matches a particular case.
#[macro_export]
macro_rules! is_case {
    ($val:ident, $var:path) => {
        match $val {
            $var { .. } => true,
            _ => false,
        }
    };
}

#[cfg(test)]
mod is_case_tests {
    enum TestEnum {
        One,
        Two { _hello: u8 },
        Three(u8),
    }

    #[test]
    fn test_is_case_one() {
        let mut value = TestEnum::One;
        assert!(is_case!(value, TestEnum::One));

        value = TestEnum::Three(1);
        assert!(!is_case!(value, TestEnum::One));
    }

    #[test]
    fn test_is_case_two() {
        let mut value = TestEnum::Two { _hello: 8 };
        assert!(is_case!(value, TestEnum::Two));

        value = TestEnum::Three(1);
        assert!(!is_case!(value, TestEnum::Two));
    }

    #[test]
    fn test_is_case_three() {
        let mut value = TestEnum::Three(1);
        assert!(is_case!(value, TestEnum::Three));

        value = TestEnum::One;
        assert!(!is_case!(value, TestEnum::Two));
    }
}
