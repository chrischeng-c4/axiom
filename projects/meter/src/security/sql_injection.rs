// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-security-sql-injection-rs.md#source
// CODEGEN-BEGIN
//! SQL injection testing utilities
//!
//! Provides tools to test input validators and sanitizers against SQL injection attacks.

use super::payloads::PayloadDatabase;

/// Result of an injection test
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-sql-injection-rs.md#source
pub enum InjectionResult {
    /// Input was blocked (validator returned error)
    Blocked,
    /// Input was sanitized (modified but accepted)
    Sanitized,
    /// Input was allowed as-is
    Allowed,
    /// Unexpected error occurred
    Error(String),
}

/// A single injection test case
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-sql-injection-rs.md#source
pub struct InjectionTest {
    /// Name/description of the test
    pub name: String,
    /// Payload being tested
    pub payload: String,
    /// Expected result
    pub expected: InjectionResult,
    /// Actual result (filled in after test)
    pub actual: Option<InjectionResult>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-sql-injection-rs.md#source
impl InjectionTest {
    /// Create a new injection test
    pub fn new(
        name: impl Into<String>,
        payload: impl Into<String>,
        expected: InjectionResult,
    ) -> Self {
        Self {
            name: name.into(),
            payload: payload.into(),
            expected,
            actual: None,
        }
    }

    /// Check if the test passed
    pub fn passed(&self) -> bool {
        self.actual.as_ref() == Some(&self.expected)
    }
}

/// SQL injection tester
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-sql-injection-rs.md#source
pub struct SqlInjectionTester {
    payload_db: PayloadDatabase,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-sql-injection-rs.md#source
impl SqlInjectionTester {
    /// Create a new SQL injection tester
    pub fn new() -> Self {
        Self {
            payload_db: PayloadDatabase::new(),
        }
    }

    /// Test a validator function against a set of payloads
    ///
    /// The validator should return:
    /// - Ok(sanitized_string) if input is accepted (possibly modified)
    /// - Err(error_message) if input is rejected
    pub fn test<F>(&self, validator: F, payloads: &[String]) -> Vec<InjectionTest>
    where
        F: Fn(&str) -> Result<String, String>,
    {
        payloads
            .iter()
            .map(|payload| {
                let result = match validator(payload) {
                    Ok(sanitized) => {
                        if &sanitized == payload {
                            InjectionResult::Allowed
                        } else {
                            InjectionResult::Sanitized
                        }
                    }
                    Err(_) => InjectionResult::Blocked,
                };

                let mut test = InjectionTest::new(
                    format!("Test payload: {}", Self::truncate(payload, 50)),
                    payload.clone(),
                    InjectionResult::Blocked, // We expect all malicious payloads to be blocked
                );
                test.actual = Some(result);
                test
            })
            .collect()
    }

    /// Test identifier validation (table names, column names, etc.)
    ///
    /// Uses identifier injection payloads from the database.
    pub fn test_identifiers<F>(&self, validator: F) -> Vec<InjectionTest>
    where
        F: Fn(&str) -> Result<String, String>,
    {
        let payloads = self.payload_db.identifier_injection();
        self.test(validator, payloads)
    }

    /// Test value validation (query values, user input, etc.)
    ///
    /// Uses SQL injection payloads from the database.
    pub fn test_values<F>(&self, validator: F) -> Vec<InjectionTest>
    where
        F: Fn(&str) -> Result<String, String>,
    {
        let payloads = self.payload_db.sql_injection();
        self.test(validator, payloads)
    }

    /// Test unicode tricks validation
    ///
    /// Uses unicode trick payloads from the database.
    pub fn test_unicode<F>(&self, validator: F) -> Vec<InjectionTest>
    where
        F: Fn(&str) -> Result<String, String>,
    {
        let payloads = self.payload_db.unicode_tricks();
        self.test(validator, payloads)
    }

    /// Test overflow and resource exhaustion
    ///
    /// Uses overflow payloads from the database.
    pub fn test_overflow<F>(&self, validator: F) -> Vec<InjectionTest>
    where
        F: Fn(&str) -> Result<String, String>,
    {
        let payloads = self.payload_db.overflow();
        self.test(validator, payloads)
    }

    /// Test all categories
    pub fn test_all<F>(&self, validator: F) -> Vec<InjectionTest>
    where
        F: Fn(&str) -> Result<String, String>,
    {
        let all_payloads = self.payload_db.all();
        let payloads: Vec<String> = all_payloads.iter().map(|s| (*s).clone()).collect();
        self.test(validator, &payloads)
    }

    /// Summarize test results
    ///
    /// Returns (blocked_count, allowed_count, error_count)
    pub fn summarize(results: &[InjectionTest]) -> (usize, usize, usize) {
        let mut blocked = 0;
        let mut allowed = 0;
        let mut errors = 0;

        for test in results {
            match &test.actual {
                Some(InjectionResult::Blocked) => blocked += 1,
                Some(InjectionResult::Allowed) | Some(InjectionResult::Sanitized) => allowed += 1,
                Some(InjectionResult::Error(_)) | None => errors += 1,
            }
        }

        (blocked, allowed, errors)
    }

    /// Generate a report of failed tests
    pub fn failed_tests(results: &[InjectionTest]) -> Vec<&InjectionTest> {
        results.iter().filter(|t| !t.passed()).collect()
    }

    /// Truncate string for display
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-sql-injection-rs.md#source
impl Default for SqlInjectionTester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_result_equality() {
        assert_eq!(InjectionResult::Blocked, InjectionResult::Blocked);
        assert_eq!(InjectionResult::Allowed, InjectionResult::Allowed);
        assert_ne!(InjectionResult::Blocked, InjectionResult::Allowed);
    }

    #[test]
    fn test_injection_test_creation() {
        let test = InjectionTest::new("Test 1", "' OR 1=1--", InjectionResult::Blocked);
        assert_eq!(test.name, "Test 1");
        assert_eq!(test.payload, "' OR 1=1--");
        assert_eq!(test.expected, InjectionResult::Blocked);
        assert!(test.actual.is_none());
    }

    #[test]
    fn test_injection_test_passed() {
        let mut test = InjectionTest::new("Test", "payload", InjectionResult::Blocked);
        assert!(!test.passed()); // No actual result yet

        test.actual = Some(InjectionResult::Blocked);
        assert!(test.passed());

        test.actual = Some(InjectionResult::Allowed);
        assert!(!test.passed());
    }

    #[test]
    fn test_sql_injection_tester_creation() {
        let tester = SqlInjectionTester::new();
        assert!(!tester.payload_db.sql_injection().is_empty());
    }

    #[test]
    fn test_validator_that_blocks_all() {
        let tester = SqlInjectionTester::new();

        // Validator that blocks everything
        let validator = |_input: &str| Err("Blocked".to_string());

        let payloads = vec!["' OR 1=1--".to_string(), "admin'--".to_string()];
        let results = tester.test(validator, &payloads);

        assert_eq!(results.len(), 2);
        for result in results {
            assert_eq!(result.actual, Some(InjectionResult::Blocked));
            assert!(result.passed());
        }
    }

    #[test]
    fn test_validator_that_allows_all() {
        let tester = SqlInjectionTester::new();

        // Validator that allows everything
        let validator = |input: &str| Ok(input.to_string());

        let payloads = vec!["' OR 1=1--".to_string()];
        let results = tester.test(validator, &payloads);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].actual, Some(InjectionResult::Allowed));
        assert!(!results[0].passed()); // Should fail because we expect it to be blocked
    }

    #[test]
    fn test_validator_that_sanitizes() {
        let tester = SqlInjectionTester::new();

        // Validator that removes quotes
        let validator = |input: &str| Ok(input.replace('\'', ""));

        let payloads = vec!["' OR 1=1--".to_string()];
        let results = tester.test(validator, &payloads);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].actual, Some(InjectionResult::Sanitized));
        assert!(!results[0].passed()); // Should fail because we expect it to be blocked
    }

    #[test]
    fn test_test_identifiers() {
        let tester = SqlInjectionTester::new();

        let validator = |input: &str| {
            // Simple validator: block anything with special chars
            if input.contains(['$', '.', '\'', '"', ';', '-', '/', '\\']) {
                Err("Invalid identifier".to_string())
            } else {
                Ok(input.to_string())
            }
        };

        let results = tester.test_identifiers(validator);
        assert!(!results.is_empty());

        // Most malicious identifiers should be blocked
        let (blocked, allowed, _) = SqlInjectionTester::summarize(&results);
        assert!(
            blocked > allowed,
            "More payloads should be blocked than allowed"
        );
    }

    #[test]
    fn test_test_values() {
        let tester = SqlInjectionTester::new();

        let validator = |input: &str| {
            // Block common SQL injection patterns
            if input.contains("OR") || input.contains("--") || input.contains("UNION") {
                Err("SQL injection detected".to_string())
            } else {
                Ok(input.to_string())
            }
        };

        let results = tester.test_values(validator);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_summarize() {
        let tests = vec![
            {
                let mut t = InjectionTest::new("T1", "p1", InjectionResult::Blocked);
                t.actual = Some(InjectionResult::Blocked);
                t
            },
            {
                let mut t = InjectionTest::new("T2", "p2", InjectionResult::Blocked);
                t.actual = Some(InjectionResult::Allowed);
                t
            },
            {
                let mut t = InjectionTest::new("T3", "p3", InjectionResult::Blocked);
                t.actual = Some(InjectionResult::Error("Error".to_string()));
                t
            },
        ];

        let (blocked, allowed, errors) = SqlInjectionTester::summarize(&tests);
        assert_eq!(blocked, 1);
        assert_eq!(allowed, 1);
        assert_eq!(errors, 1);
    }

    #[test]
    fn test_failed_tests() {
        let tests = vec![
            {
                let mut t = InjectionTest::new("T1", "p1", InjectionResult::Blocked);
                t.actual = Some(InjectionResult::Blocked);
                t
            },
            {
                let mut t = InjectionTest::new("T2", "p2", InjectionResult::Blocked);
                t.actual = Some(InjectionResult::Allowed);
                t
            },
        ];

        let failed = SqlInjectionTester::failed_tests(&tests);
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].name, "T2");
    }

    #[test]
    fn test_truncate() {
        let short = "short";
        assert_eq!(SqlInjectionTester::truncate(short, 10), "short");

        let long = "a".repeat(100);
        let truncated = SqlInjectionTester::truncate(&long, 50);
        assert_eq!(truncated.len(), 53); // 50 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_test_unicode() {
        let tester = SqlInjectionTester::new();

        let validator = |input: &str| {
            // Block zero-width characters
            if input.contains(['\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}']) {
                Err("Zero-width character detected".to_string())
            } else {
                Ok(input.to_string())
            }
        };

        let results = tester.test_unicode(validator);
        assert!(!results.is_empty());

        let (blocked, _, _) = SqlInjectionTester::summarize(&results);
        assert!(blocked > 0, "Should block some unicode tricks");
    }

    #[test]
    fn test_test_overflow() {
        let tester = SqlInjectionTester::new();

        let validator = |input: &str| {
            // Block strings longer than 1000 chars
            if input.len() > 1000 {
                Err("Input too long".to_string())
            } else {
                Ok(input.to_string())
            }
        };

        let results = tester.test_overflow(validator);
        assert!(!results.is_empty());

        let (blocked, _, _) = SqlInjectionTester::summarize(&results);
        assert!(blocked > 0, "Should block some overflow payloads");
    }

    #[test]
    fn test_test_all() {
        let tester = SqlInjectionTester::new();

        let validator = |_input: &str| Err("Blocked".to_string());

        let results = tester.test_all(validator);
        assert!(!results.is_empty());
        assert!(results.len() >= 100, "Should test all payload categories");
    }

    #[test]
    fn test_default_implementation() {
        let tester1 = SqlInjectionTester::default();
        let tester2 = SqlInjectionTester::new();

        assert_eq!(
            tester1.payload_db.sql_injection().len(),
            tester2.payload_db.sql_injection().len()
        );
    }
}
// CODEGEN-END
