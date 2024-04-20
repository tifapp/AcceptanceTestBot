use super::error::RoswaalCompilationError;

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalTest {
    name: String,
    steps: Vec<RoswaalStep>
}

impl RoswaalTest {
    pub fn compile(roswaal_string: &String) -> Result<Self, RoswaalCompilationError> {
        let mut lines = roswaal_string.lines();
        let has_test_name = lines.next()
            .map(|l| l.to_lowercase().starts_with("new test:"))
            .unwrap_or(false);
        if !has_test_name {
            return Err(RoswaalCompilationError::NoTestName);
        }
        let step_line = lines.next();
        if let Some((step_name, _)) = step_line.and_then(|l| l.split_once(":")) {
            let step_name = String::from(step_name.trim());
            return Err(RoswaalCompilationError::InvalidStepName(step_name))
        }
        Err(RoswaalCompilationError::NoTestSteps)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoswaalStep {
    name: String,
    requirement: String
}

#[cfg(test)]
mod roswaal_test_tests {
    use super::*;

    #[test]
    fn test_compile_returns_no_name_for_empty_string() {
        let result = RoswaalTest::compile(&String::from(""));
        assert_eq!(result, Err(RoswaalCompilationError::NoTestName))
    }

    #[test]
    fn test_compile_returns_no_name_for_random_string() {
        let test_string = &String::from("jkashdkjashdkjahsd ehiuh3ui2geuyg23urg");
        let result = RoswaalTest::compile(test_string);
        assert_eq!(result, Err(RoswaalCompilationError::NoTestName))
    }

    #[test]
    fn test_compile_returns_no_steps_when_name_formatted_correctly_uppercase() {
        let test = "New Test: Hello world";
        let result = RoswaalTest::compile(&String::from(test));
        assert_eq!(result, Err(RoswaalCompilationError::NoTestSteps))
    }

    #[test]
    fn test_compile_returns_no_steps_when_name_formatted_correctly_lowercase() {
        let test = "new test: Hello world";
        let result = RoswaalTest::compile(&String::from(test));
        assert_eq!(result, Err(RoswaalCompilationError::NoTestSteps))
    }

    #[test]
    fn test_compile_returns_no_steps_when_step_line_is_random_string() {
        let test = "\
            new test: Hello world
            lsjkhadjkhasdfjkhasdjkfhkjsd
            ";
        let result = RoswaalTest::compile(&String::from(test));
        assert_eq!(result, Err(RoswaalCompilationError::NoTestSteps))
    }

    #[test]
    fn test_compile_returns_invalid_step_name_when_command_is_not_a_step() {
        let test = "\
            new test: Hello world
            passo 1: mamma mia
            ";
        let result = RoswaalTest::compile(&String::from(test));
        let step_name = String::from("passo 1");
        assert_eq!(
            result,
            Err(RoswaalCompilationError::InvalidStepName(step_name))
        )
    }
}
