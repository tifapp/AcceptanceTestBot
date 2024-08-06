use crate::location::name::RoswaalLocationName;

use super::{
    ast::RoswaalTestSyntax,
    compiler::{RoswaalCompilationError, RoswaalCompile, RoswaalCompileContext},
    test::RoswaalTest,
};

/// A data type constructed from compiling multiple test cases at a time.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RoswaalTestCompilationResults<'a> {
    results: Vec<(
        Result<RoswaalTest, Vec<RoswaalCompilationError>>,
        RoswaalTestSyntax<'a>,
    )>,
}

impl<'a> RoswaalTestCompilationResults<'a> {
    pub fn compile(
        syntax: &Vec<RoswaalTestSyntax<'a>>,
        location_names: &Vec<RoswaalLocationName>,
    ) -> Self {
        Self {
            results: syntax
                .iter()
                .map(|syntax| {
                    let compile_context = RoswaalCompileContext::new(&location_names);
                    let result = RoswaalTest::compile_syntax(syntax, compile_context);
                    (result, syntax.clone())
                })
                .collect(),
        }
    }
}

impl<'a> RoswaalTestCompilationResults<'a> {
    pub fn tests(&self) -> Vec<RoswaalTest> {
        self.results
            .iter()
            .filter_map(|r| r.0.clone().ok())
            .collect()
    }

    pub fn tests_with_syntax(&self) -> Vec<(RoswaalTest, RoswaalTestSyntax<'a>)> {
        self.results
            .iter()
            .filter_map(|r| match r.0.clone() {
                Ok(test) => Some((test, r.1.clone())),
                Err(_) => None,
            })
            .collect()
    }

    pub fn failures(&self) -> Vec<RoswaalTestCompilationFailure<'a>> {
        self.results
            .iter()
            .enumerate()
            .filter_map(|(i, r)| match r.0.clone() {
                Ok(_) => None,
                Err(errors) => Some(RoswaalTestCompilationFailure {
                    i,
                    errors,
                    syntax: r.1.clone(),
                }),
            })
            .collect()
    }

    pub fn has_compiling_tests(&self) -> bool {
        self.results.iter().filter(|r| r.0.is_ok()).count() > 0
    }

    pub fn has_non_compiling_tests(&self) -> bool {
        self.results.iter().filter(|r| r.0.is_err()).count() > 0
    }
}

/// A test compilation failure.
#[derive(Debug, Clone)]
pub struct RoswaalTestCompilationFailure<'a> {
    i: usize,
    errors: Vec<RoswaalCompilationError>,
    syntax: RoswaalTestSyntax<'a>,
}

impl<'a> RoswaalTestCompilationFailure<'a> {
    /// Returns the order number in which this test appeared in a user entered string of tests
    /// to compile.
    ///
    /// This value is 1 index based.
    pub fn test_number(&self) -> usize {
        self.i + 1
    }

    pub fn errors(&self) -> &[RoswaalCompilationError] {
        &self.errors
    }

    pub fn syntax(&self) -> &RoswaalTestSyntax<'a> {
        &self.syntax
    }
}

#[cfg(test)]
mod tests {
    use crate::language::ast::RoswaalTestSyntax;

    use super::RoswaalTestCompilationResults;

    #[test]
    fn compile_raw_results_mixed() {
        let syntax = vec![
            RoswaalTestSyntax::from(""),
            RoswaalTestSyntax::from("New Test: Test\nStep 1: Test\nRequirement 1: Test"),
        ];
        let results = RoswaalTestCompilationResults::compile(&syntax, &vec![]);
        assert_eq!(results.tests().len(), 1);
        assert_eq!(results.tests()[0].name(), "Test");
        assert_eq!(results.failures().len(), 1);
        assert_eq!(results.failures()[0].test_number(), 1);
        assert!(results.has_compiling_tests());
        assert!(results.has_non_compiling_tests())
    }

    #[test]
    fn compile_raw_results_all_success() {
        let syntax = vec![RoswaalTestSyntax::from(
            "New Test: Test\nStep 1: Test\nRequirement 1: Test",
        )];
        let results = RoswaalTestCompilationResults::compile(&syntax, &vec![]);
        assert!(results.has_compiling_tests());
        assert!(!results.has_non_compiling_tests())
    }

    #[test]
    fn compile_raw_results_all_failures() {
        let syntax = vec![RoswaalTestSyntax::from("")];
        let results = RoswaalTestCompilationResults::compile(&syntax, &vec![]);
        assert!(!results.has_compiling_tests());
        assert!(results.has_non_compiling_tests())
    }
}
